#!/bin/sh
set -eu

project="l4-proxy-openresty-check"
compose_file="tests/openresty/compose.yml"

cleanup() {
  docker compose -p "$project" -f "$compose_file" down
}
trap cleanup EXIT

docker compose -p "$project" -f "$compose_file" up -d

probe() {
  service="$1"
  port="$2"
  attempts=0
  while [ "$attempts" -lt 20 ]; do
    if result=$(docker compose -p "$project" -f "$compose_file" exec -T "$service" \
      wget -qO- --header='X-Real-IP: 203.0.113.7' "http://openresty:$port/whoami" 2>/dev/null); then
      printf '%s\n' "$result"
      return 0
    fi
    attempts=$((attempts + 1))
    sleep 1
  done
  echo "OpenResty did not become ready" >&2
  return 1
}

trusted_web=$(probe trusted-probe 8081)
untrusted_web=$(probe untrusted-probe 8081)
trusted_app=$(probe trusted-probe 8082)

printf 'trusted web:\n%s\nuntrusted web:\n%s\ntrusted app:\n%s\n' \
  "$trusted_web" "$untrusted_web" "$trusted_app"

expect_line() {
  printf '%s\n' "$1" | grep -Fqx "$2" || {
    echo "Missing expected line: $2" >&2
    exit 1
  }
}

expect_line "$trusted_web" 'route=web'
expect_line "$trusted_web" 'remote_addr=203.0.113.7'
expect_line "$trusted_web" 'realip_remote_addr=172.30.78.10'
expect_line "$untrusted_web" 'route=web'
expect_line "$untrusted_web" 'remote_addr=172.30.78.11'
expect_line "$trusted_app" 'route=app'
expect_line "$trusted_app" 'remote_addr=203.0.113.7'
expect_line "$trusted_app" 'realip_remote_addr=172.30.78.10'
