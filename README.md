# Pingdom Edge Proxy

Pingdom 웹 인스턴스의 진입점에서 TLS 트래픽을 받아 세 도메인으로 전달하는 HAProxy 기반 엣지 프록시입니다.

- `www.typenull.xyz`
- `api.typenull.xyz`
- `origin.typenull.xyz`

HAProxy는 TCP L4 프록시로 동작하며 TLS ClientHello의 SNI를 사용해 backend를 선택합니다. TLS를 복호화하지 않는 Passthrough 구조이므로 실제 TLS 인증서와 TLS handshake는 각 backend가 처리합니다.

## Responsibilities

- 웹 인스턴스의 `443` 포트에서 외부 TCP 연결 수신
- SNI 기반 도메인별 backend 라우팅
- 클라이언트 IP 기준 연결 Rate Limit
- TCP health check 및 장애 backend 제외
- PROXY protocol v2를 통한 원본 IP 전달
- HAProxy Runtime API를 사용하는 Rust 상태 에이전트 제공

현재 Rate Limit은 HAProxy 인스턴스의 모든 도메인에 공유되는 stick table 기준입니다. 여러 HAProxy 인스턴스 전체의 글로벌 제한이 필요하면 이후 `peers` 동기화 또는 외부 Rate Limit 저장소를 추가해야 합니다.

## Architecture

```text
Client
  │ TCP 3-way handshake
  ▼
HAProxy :443
  │ SNI routing + connection rate limit
  ├── www.typenull.xyz   ── PROXY v2 ──> www-openresty:443
  ├── api.typenull.xyz   ── PROXY v2 ──> api-openresty:443
  └── origin.typenull.xyz ─ PROXY v2 ──> origin-openresty:443

Rust Agent ── Unix Socket ──> HAProxy Runtime API
```

각 요청은 Client와 HAProxy 사이, HAProxy와 backend 사이에서 별도의 TCP 연결을 가집니다. backend는 PROXY protocol v2를 지원하고 원본 주소를 신뢰하도록 구성해야 합니다.

## Docker

```bash
docker compose up -d --build
```

HAProxy는 호스트의 `443` 포트를 사용하고, 통계 페이지는 `127.0.0.1:8404/stats`로만 노출합니다. Runtime API 소켓은 `haproxy-runtime` Docker volume으로 Rust Agent와 공유합니다.

Backend 서비스는 같은 Docker network(`pingdom-edge`)에 다음 DNS 이름으로 연결되어야 합니다.

| Hostname | Domain | Port |
|---|---|---:|
| `www-openresty` | `www.typenull.xyz` | 443 |
| `api-openresty` | `api.typenull.xyz` | 443 |
| `origin-openresty` | `origin.typenull.xyz` | 443 |

실제 OpenResty 또는 Origin 배포 환경에서 이 이름을 사용할 수 없다면 [haproxy/haproxy.cfg](haproxy/haproxy.cfg)의 backend 주소를 해당 DNS 이름으로 변경해야 합니다.

## TLS and Real IP

HAProxy는 현재 TLS Passthrough를 사용합니다. 따라서 인증서는 HAProxy가 아니라 backend에서 관리합니다. HAProxy가 전달하는 PROXY protocol v2를 OpenResty에서 활성화해야 합니다.

OpenResty 예시:

```nginx
listen 443 ssl proxy_protocol;

set_real_ip_from <haproxy-network-cidr>;
real_ip_header proxy_protocol;
```

HAProxy에서 TLS를 종료하는 구성이 필요해지면 `mode tcp`를 HTTP/TLS frontend로 분리하고 인증서와 SNI 정책을 별도로 추가해야 합니다.

## Rate Limit

현재 설정은 모든 도메인에 공유되는 IP별 연결 Rate Limit을 사용합니다.

```text
10초 동안 IP당 100개 초과 연결 거부
```

L4 Passthrough 구조에서는 HTTP 요청 수가 아니라 TCP 연결 수를 제한합니다. 여러 HAProxy 노드 전체에 동일한 한도를 적용하려면 노드 간 stick table 동기화가 필요합니다.

## Rust Agent

Rust Agent는 HAProxy Runtime API Unix Socket에 연결해 `show info`와 `show stat`을 5초마다 조회합니다.

로컬 빌드:

```bash
cd rust-agent
cargo run -- --socket /var/run/haproxy/admin.sock --interval 5
```

Docker 실행 시에는 Compose가 Runtime API socket을 자동으로 공유합니다.

## Verification

HAProxy 설정 문법을 확인합니다.

```bash
docker run --rm \
  -v "$PWD/haproxy:/usr/local/etc/haproxy:ro" \
  haproxy:3.2-alpine \
  haproxy -c -f /usr/local/etc/haproxy/haproxy.cfg
```

Rust Agent를 확인합니다.

```bash
cargo check --manifest-path rust-agent/Cargo.toml
```

실행 후 통계를 확인합니다.

```bash
curl http://127.0.0.1:8404/stats
```

SNI 라우팅은 실제 인증서와 backend가 준비된 환경에서 확인합니다.

```bash
openssl s_client -connect 127.0.0.1:443 -servername www.typenull.xyz
openssl s_client -connect 127.0.0.1:443 -servername api.typenull.xyz
openssl s_client -connect 127.0.0.1:443 -servername origin.typenull.xyz
```

## Repository Structure

```text
.
├── README.md
├── Dockerfile
├── compose.yml
├── haproxy
│   └── haproxy.cfg
└── rust-agent
    ├── Cargo.toml
    ├── Dockerfile
    └── src
        └── main.rs
```

## License

MIT License. 자세한 내용은 [LICENSE](LICENSE)를 참고하세요.

Part of Pingdom.
