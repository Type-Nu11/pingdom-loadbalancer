# Rust edge proxy

```text
Client -- HTTPS --> Rust edge :443 -- HTTP --> OpenResty :8081 or :8082
                                             -- HTTP --> Backend on the same VPS
```

The edge accepts only `www.typenull.xyz`. It terminates HTTPS and forwards each request to OpenResty on the same Ubuntu host. A request with `X-Client-Type: App` goes to port 8082. All other requests go to port 8081. The routing header is forwarded unchanged. The edge streams the request and response bodies and preserves application headers, except that it replaces any incoming `X-Real-IP` with the TCP peer address.

`X-Client-Type` is a routing hint, not an authentication mechanism. OpenResty must enforce authorization for both ports.

OpenResty currently uses `$remote_addr` for rate limits and logs. To make that value reflect the client address carried in `X-Real-IP`, configure `real_ip_header X-Real-IP` and `set_real_ip_from` for the edge source address OpenResty actually sees. Do not trust `X-Real-IP` from arbitrary clients. The Rust edge always replaces a client-supplied `X-Real-IP` before forwarding. The isolated [OpenResty trust experiment](../tests/openresty/README.md) checks trusted versus untrusted source addresses. In the planned same-host loopback setup, trusting `127.0.0.1` trusts every local process, so the host itself remains part of the trust boundary.

The listener address, domain, certificate paths, upstream addresses and header name are in `configs/edge.toml`. The default upstream addresses are `127.0.0.1:8081` and `127.0.0.1:8082` because the Rust container uses Linux host networking.

## Rate limit

The in-memory limiter uses the actual TCP peer IP, never `X-Real-IP` or another client-provided header. The initial setting allows 1,200 requests per 10-second window per IP. The 1,201st and later requests in that window receive `429` with `Retry-After`; one burst does not blacklist the IP. If the same IP exceeds the limit in three distinct windows within 60 seconds, it is blocked for 10 seconds. An expired block clears its strikes. This policy is local to one Rust process, and restarting the process clears its history. No Redis is required for the single-instance setup.

Shared public IPs (NAT, mobile carriers, corporate gateways) may represent many users, so 1,200/10s is a provisional setting, not a universally safe threshold. Monitor `429` rates before tightening it. The table is bounded to 50,000 tracked IPs; when a shard is full, previously unseen IPs are allowed through rather than risking false-positive blocks. This is a deliberate fail-open tradeoff under memory pressure.

## Existing Ubuntu certificate

`deploy/rust.compose.yml` mounts Ubuntu's `/etc/letsencrypt` directory read-only. The Rust process reads `fullchain.pem` and `privkey.pem` from the mounted directory at startup. It does not request or renew certificates. If the existing certificate lives elsewhere, set `TLS_CERT_DIR` to the directory containing the `live/` and `archive/` trees, or update the paths in `configs/edge.toml`. The whole tree is mounted because Certbot's `live/` files commonly link into `archive/`.

To build and start on Ubuntu after confirming the certificate paths and that port 443 is free:

```sh
docker compose -f deploy/rust.compose.yml config
docker compose -f deploy/rust.compose.yml up -d --build
docker compose -f deploy/rust.compose.yml logs -f proxy-server
```

To run directly on the host:

```sh
cargo run --release -p proxy-server -- configs/edge.toml
```

The current server handles ordinary HTTP/1.1 and HTTP/2 requests over TLS. WebSocket upgrades and active upstream health checks have not been added yet. HTTP parsing and forwarding can change wire formatting such as header order or capitalization; the application header values and body stream are passed through.

## CI

`.github/workflows/ci.yml` checks formatting, tests, Clippy and a release build with Rust 1.98.0 on pushes and pull requests. A separate job runs the isolated OpenResty real-IP trust test. CI does not deploy the proxy, access production certificates, or benchmark the 1,200/10s threshold.
