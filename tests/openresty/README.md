# OpenResty real-IP trust experiment

This isolated fixture tests the OpenResty side of `X-Real-IP` trust. It does **not** expose ports to the host or modify the production OpenResty configuration. The trusted probe (`172.30.78.10`) stands in for the Rust edge; the untrusted probe (`172.30.78.11`) stands in for a direct client. Both can send the same forged header, but only the trusted probe should affect `$remote_addr`.

Run from the repository root:

```sh
sh tests/openresty/check.sh
```

This automated check starts only its own isolated Compose project and removes it afterward. For manual inspection, use:

```sh
docker compose -f tests/openresty/compose.yml up -d
docker compose -f tests/openresty/compose.yml exec -T trusted-probe \
  wget -qO- --header='X-Real-IP: 203.0.113.7' http://openresty:8081/whoami
docker compose -f tests/openresty/compose.yml exec -T untrusted-probe \
  wget -qO- --header='X-Real-IP: 203.0.113.7' http://openresty:8081/whoami
docker compose -f tests/openresty/compose.yml exec -T trusted-probe \
  wget -qO- --header='X-Real-IP: 203.0.113.7' http://openresty:8082/whoami
docker compose -f tests/openresty/compose.yml down
```

Expected: the trusted probe gets `remote_addr=203.0.113.7` and `realip_remote_addr=172.30.78.10`; the untrusted probe keeps `remote_addr=172.30.78.11`. Port 8081 reports `route=web` and 8082 reports `route=app`.

In production, replace the test source address with the actual source IP of the Rust edge as seen by OpenResty, and restrict the OpenResty listener so clients cannot bypass the edge. With both services on the same host via loopback, `set_real_ip_from 127.0.0.1` trusts **every local process**, not only the Rust process; treat that as a host-level trust boundary.

The 1,200-requests-per-10-seconds rate limit is not measured by this fixture. Test it separately against the Rust edge during the deployment exercise, recording 429 counts and `Retry-After` responses for a shared-IP-like workload.
