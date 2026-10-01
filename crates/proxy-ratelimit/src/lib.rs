use std::{
    collections::{hash_map::DefaultHasher, HashMap, VecDeque},
    hash::{Hash, Hasher},
    net::IpAddr,
    sync::Mutex,
    time::{Duration, Instant},
};

const SHARDS: usize = 64;

#[derive(Debug, Clone, Copy)]
pub struct Policy {
    pub max_requests: u64,
    pub window: Duration,
    pub strike_window: Duration,
    pub strikes_before_block: usize,
    pub block: Duration,
    pub max_tracked_ips: usize,
}

impl Policy {
    pub fn validate(self) -> Result<Self, &'static str> {
        if self.max_requests == 0
            || self.window.is_zero()
            || self.strike_window < self.window
            || self.strikes_before_block == 0
            || self.block.is_zero()
            || self.max_tracked_ips < SHARDS
        {
            Err("invalid rate-limit policy")
        } else {
            Ok(self)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Allowed,
    Limited { retry_after: Duration },
    Blacklisted { retry_after: Duration },
}

struct Entry {
    window_start: Instant,
    count: u64,
    exceeded_this_window: bool,
    strikes: VecDeque<Instant>,
    blocked_at: Option<Instant>,
    last_seen: Instant,
}

impl Entry {
    fn new(now: Instant) -> Self {
        Self {
            window_start: now,
            count: 0,
            exceeded_this_window: false,
            strikes: VecDeque::new(),
            blocked_at: None,
            last_seen: now,
        }
    }
}

/// Shared, per-process request limiter. IPs are obtained from the TCP peer,
/// never from client-provided forwarding headers.
pub struct RateLimiter {
    policy: Policy,
    shards: Vec<Mutex<HashMap<IpAddr, Entry>>>,
}

impl RateLimiter {
    pub fn new(policy: Policy) -> Result<Self, &'static str> {
        let policy = policy.validate()?;
        let shards = (0..SHARDS).map(|_| Mutex::new(HashMap::new())).collect();
        Ok(Self { policy, shards })
    }

    pub fn check(&self, ip: IpAddr) -> Decision {
        self.check_at(ip, Instant::now())
    }

    fn check_at(&self, ip: IpAddr, now: Instant) -> Decision {
        let ip = normalize_ip(ip);
        let index = shard_index(ip);
        let mut entries = self.shards[index]
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !entries.contains_key(&ip) {
            let capacity = self.policy.max_tracked_ips / SHARDS
                + usize::from(index < self.policy.max_tracked_ips % SHARDS);
            if entries.len() >= capacity {
                let idle = self.policy.strike_window.max(self.policy.block);
                entries.retain(|_, entry| {
                    now.saturating_duration_since(entry.last_seen) < idle
                        || entry.blocked_at.is_some_and(|start| {
                            now.saturating_duration_since(start) < self.policy.block
                        })
                });
            }
            // When the bounded table is full, unknown IPs pass through rather
            // than causing unrelated clients to be blocked.
            if entries.len() >= capacity {
                return Decision::Allowed;
            }
        }

        let entry = entries.entry(ip).or_insert_with(|| Entry::new(now));
        entry.last_seen = now;
        if let Some(start) = entry.blocked_at {
            let elapsed = now.saturating_duration_since(start);
            if elapsed < self.policy.block {
                return Decision::Blacklisted {
                    retry_after: self.policy.block - elapsed,
                };
            }
            *entry = Entry::new(now);
        }

        let elapsed = now.saturating_duration_since(entry.window_start);
        if elapsed >= self.policy.window {
            entry.window_start = now;
            entry.count = 0;
            entry.exceeded_this_window = false;
        }
        while entry.strikes.front().is_some_and(|strike| {
            now.saturating_duration_since(*strike) >= self.policy.strike_window
        }) {
            entry.strikes.pop_front();
        }
        if entry.count < self.policy.max_requests {
            entry.count += 1;
            return Decision::Allowed;
        }

        if !entry.exceeded_this_window {
            entry.exceeded_this_window = true;
            entry.strikes.push_back(now);
            if entry.strikes.len() >= self.policy.strikes_before_block {
                entry.blocked_at = Some(now);
                entry.strikes.clear();
                return Decision::Blacklisted {
                    retry_after: self.policy.block,
                };
            }
        }
        Decision::Limited {
            retry_after: self.policy.window - now.saturating_duration_since(entry.window_start),
        }
    }
}

fn normalize_ip(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(address) => address.to_ipv4_mapped().map(IpAddr::V4).unwrap_or(ip),
        _ => ip,
    }
}

fn shard_index(ip: IpAddr) -> usize {
    let mut hasher = DefaultHasher::new();
    ip.hash(&mut hasher);
    (hasher.finish() as usize) % SHARDS
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> Policy {
        Policy {
            max_requests: 1200,
            window: Duration::from_secs(10),
            strike_window: Duration::from_secs(60),
            strikes_before_block: 3,
            block: Duration::from_secs(10),
            max_tracked_ips: 64,
        }
    }

    #[test]
    fn exact_limit_is_allowed_then_returns_429_without_immediate_blacklist() {
        let limiter = RateLimiter::new(policy()).unwrap();
        let ip = "192.0.2.1".parse().unwrap();
        let now = Instant::now();
        for _ in 0..1200 {
            assert_eq!(limiter.check_at(ip, now), Decision::Allowed);
        }
        assert_eq!(
            limiter.check_at(ip, now),
            Decision::Limited {
                retry_after: Duration::from_secs(10)
            }
        );
        assert_eq!(
            limiter.check_at(ip, now),
            Decision::Limited {
                retry_after: Duration::from_secs(10)
            }
        );
        assert_eq!(
            limiter.check_at(ip, now + Duration::from_secs(10)),
            Decision::Allowed
        );
    }

    #[test]
    fn three_distinct_over_limit_windows_trigger_temporary_block() {
        let mut config = policy();
        config.max_requests = 1;
        let limiter = RateLimiter::new(config).unwrap();
        let ip = "192.0.2.2".parse().unwrap();
        let now = Instant::now();
        for seconds in [0, 10] {
            let at = now + Duration::from_secs(seconds);
            assert_eq!(limiter.check_at(ip, at), Decision::Allowed);
            assert!(matches!(limiter.check_at(ip, at), Decision::Limited { .. }));
        }
        let at = now + Duration::from_secs(20);
        assert_eq!(limiter.check_at(ip, at), Decision::Allowed);
        assert_eq!(
            limiter.check_at(ip, at),
            Decision::Blacklisted {
                retry_after: config.block
            }
        );
        assert_eq!(
            limiter.check_at(ip, at + Duration::from_secs(3)),
            Decision::Blacklisted {
                retry_after: Duration::from_secs(7)
            }
        );
        assert_eq!(limiter.check_at(ip, at + config.block), Decision::Allowed);
    }

    #[test]
    fn strikes_expire_and_other_ips_are_independent() {
        let mut config = policy();
        config.max_requests = 1;
        let limiter = RateLimiter::new(config).unwrap();
        let ip: IpAddr = "192.0.2.3".parse().unwrap();
        let other: IpAddr = "192.0.2.4".parse().unwrap();
        let now = Instant::now();
        assert_eq!(limiter.check_at(ip, now), Decision::Allowed);
        assert!(matches!(
            limiter.check_at(ip, now),
            Decision::Limited { .. }
        ));
        assert_eq!(limiter.check_at(other, now), Decision::Allowed);
        assert_eq!(
            limiter.check_at(ip, now + Duration::from_secs(60)),
            Decision::Allowed
        );
        assert!(matches!(
            limiter.check_at(ip, now + Duration::from_secs(60)),
            Decision::Limited { .. }
        ));
    }

    #[test]
    fn ipv4_mapped_ipv6_uses_same_bucket() {
        let mut config = policy();
        config.max_requests = 1;
        let limiter = RateLimiter::new(config).unwrap();
        let now = Instant::now();
        assert_eq!(
            limiter.check_at("192.0.2.5".parse().unwrap(), now),
            Decision::Allowed
        );
        assert!(matches!(
            limiter.check_at("::ffff:192.0.2.5".parse().unwrap(), now),
            Decision::Limited { .. }
        ));
    }

    #[test]
    fn full_shard_fails_open_for_new_ip() {
        let mut config = policy();
        config.max_requests = 1;
        let limiter = RateLimiter::new(config).unwrap();
        let now = Instant::now();
        let first: IpAddr = "192.0.2.6".parse().unwrap();
        let second = (7..=254)
            .map(|last| IpAddr::from([192, 0, 2, last]))
            .find(|ip| shard_index(*ip) == shard_index(first))
            .unwrap();
        assert_eq!(limiter.check_at(first, now), Decision::Allowed);
        assert_eq!(limiter.check_at(second, now), Decision::Allowed);
        assert_eq!(limiter.check_at(second, now), Decision::Allowed);
        assert!(matches!(
            limiter.check_at(first, now),
            Decision::Limited { .. }
        ));
    }
}
