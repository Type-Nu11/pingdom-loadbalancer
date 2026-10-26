<img width="7680" height="4320" alt="image" src="https://github.com/user-attachments/assets/4fa26896-83d1-4045-8534-9d29c599f8c7" />

## Overview

이 저장소는 Pingdom 서비스의 **Rust HTTPS 엣지 프록시**와 기존 HAProxy 구성을 관리합니다.

Rust 프록시는 `www.typenull.xyz`의 TLS를 종료하고 같은 호스트의 OpenResty로 요청을 전달합니다.
`X-Client-Type: App` 요청은 `8082`, 그 외 요청은 `8081`로 전달합니다.
구성과 실행 조건은 [Rust edge proxy](docs/rust-edge.md)를 기준으로 확인합니다.

## Project Status

현재 **GA(General Availability)** 단계입니다.

안정화된 서비스를 제공하며, 구성, 정책 및 제공 결과의 변경은 Release와
변경 이력을 통해 관리합니다.

| Item | Status |
|---|---|
| Development | `Generally Available` |
| Release | `GA` |
| Stability | `Stable` |

## Repository Role

| Item | Description |
|---|---|
| Type | `Infrastructure` |
| Responsibility | HTTPS 요청 수신, Web/App 라우팅 및 IP별 요청 제한 |
| Primary Output | Rust 프록시 실행 파일, 컨테이너 및 기존 HAProxy 구성 |
| Target | Pingdom OpenResty Gateway |

## Scope

### Included

- Rust 기반 TLS 종료와 HTTP/1.1·HTTP/2 요청 전달
- `X-Client-Type`에 따른 Web/App upstream 선택
- 실제 TCP 연결 IP에 기반한 프로세스 단위 요청 제한
- 전달 시 `X-Real-IP`를 실제 TCP peer 주소로 교체
- 설정 파일 검증, 컨테이너 실행 및 OpenResty 신뢰 경계 실험
- 기존 HAProxy 설정과 배포 파일 보관

### Not Included

- TLS 인증서 발급 및 갱신
- OpenResty의 사용자 인증·권한 검증
- Pingdom 도메인 비즈니스 로직과 데이터 저장
- WebSocket 업그레이드와 능동 upstream 헬스 체크
- 여러 인스턴스 사이의 요청 제한 상태 동기화

## Key Capabilities

- **HTTPS Entry**: 호스트에 준비된 인증서를 읽어 TLS 요청을 처리합니다.
- **Web/App Routing**: 라우팅 헤더에 따라 OpenResty upstream을 선택합니다. 인증은 OpenResty에서 수행합니다.
- **Streaming Proxy**: 요청·응답 본문을 스트리밍하며 애플리케이션 헤더를 전달합니다.
- **Rate Limit**: IP별 시간 구간 요청 수와 반복 초과 이력에 따라 `429` 및 임시 차단을 적용합니다.
- **Real IP**: 클라이언트가 보낸 `X-Real-IP`를 TCP peer 주소로 교체합니다.

## Technology and Tools

| Category | Technology |
|---|---|
| Primary | Rust, Cargo workspace |
| Protocol | TLS, HTTP/1.1, HTTP/2 |
| Gateway Integration | OpenResty |
| Legacy | HAProxy 3.2 |
| Delivery | Docker, Docker Compose, GitHub Actions |

## Getting Started

### Requirements

- Git, Rust 및 Cargo
- 컨테이너 실행 시 Linux 호스트와 Docker Compose
- 기존 TLS 인증서 및 Web/App OpenResty upstream

### Setup

```bash
git clone https://github.com/Type-Nu11/pingdom-loadbalancer.git
cd pingdom-loadbalancer
```

`configs/edge.toml`의 도메인, 인증서 경로와 upstream을 실행 환경에 맞게 확인합니다.

### Usage

Ubuntu에서 인증서 경로와 `443` 포트 사용 여부를 확인한 뒤 실행합니다.

```bash
docker compose -f deploy/rust.compose.yml config
docker compose -f deploy/rust.compose.yml up -d --build
```

호스트에서 직접 실행하려면 `cargo run --release -p proxy-server -- configs/edge.toml`을 사용합니다.

## Configuration

주요 설정은 [configs/edge.toml](configs/edge.toml)과
[deploy/rust.compose.yml](deploy/rust.compose.yml)에서 관리합니다.

- 기본 도메인: `www.typenull.xyz`
- Web upstream: `127.0.0.1:8081`, App upstream: `127.0.0.1:8082`
- 기본 제한: IP당 10초 동안 1,200개 요청, 60초 안에 세 구간에서 초과하면 10초 차단
- 상태는 단일 프로세스 메모리에 보관하며 재시작 시 초기화
- 인증서는 `/etc/letsencrypt`를 읽기 전용으로 마운트하며 다른 경로는 `TLS_CERT_DIR`로 지정

인증서와 private key는 저장소에 커밋하지 않습니다. 인증서 발급·갱신은 호스트에서 관리합니다.
OpenResty의 원본 IP 복원에는 실제 edge 주소에 대한 `set_real_ip_from`과
`real_ip_header X-Real-IP` 설정이 필요합니다. 상세 조건은 [운영 안내](docs/rust-edge.md)를 참고하세요.

## Verification

변경 범위에 맞는 검사를 선택합니다. CI의 Rust 도구체인 기준은 `1.98.0`입니다.

| Verification | Purpose |
|---|---|
| `cargo +1.98.0 fmt --all -- --check` | Rust 포맷 검사 |
| `cargo +1.98.0 test --workspace --locked` | workspace 테스트 |
| `cargo +1.98.0 clippy --workspace --all-targets --locked -- -D warnings` | 정적 분석 |
| `cargo +1.98.0 build --release --workspace --locked` | Release 빌드 |
| `sh tests/openresty/check.sh` | 격리된 Docker 환경의 원본 IP 신뢰 경계 실험 |

CI는 운영 인증서 접근이나 배포를 수행하지 않습니다. 실제 트래픽 임계값과 운영 TLS·라우팅 검증은 별도로 기록합니다.

## Repository Structure

```text
.
├── Cargo.toml                 # Rust workspace
├── crates
│   ├── proxy-config           # 설정 로딩과 검증
│   ├── proxy-ratelimit        # IP별 요청 제한
│   └── proxy-server           # HTTPS 프록시 실행
├── configs/edge.toml          # 도메인, 인증서, upstream 및 제한 정책
├── deploy
│   ├── rust.Dockerfile        # Rust 이미지
│   ├── rust.compose.yml       # Rust 실행 구성
│   ├── Dockerfile             # 기존 HAProxy 이미지
│   └── compose.yml            # 기존 HAProxy 실행 구성
├── haproxy/haproxy.cfg        # 기존 HAProxy 설정
├── docs/rust-edge.md          # Rust 운영 안내
├── tests/openresty            # 원본 IP 신뢰 경계 실험
└── README.md
```

기존 HAProxy 파일은 Rust 구성과 별도로 남아 있습니다. 이전 README가 참조하던
`rust-agent/`와 루트 `compose.yml`·`Dockerfile`은 현재 트리에 없습니다.
기존 HAProxy를 사용할 경우 해당 구성과 인증서·backend 조건을 별도로 검토해야 합니다.

## Related Repositories

| Repository | Relationship |
|---|---|
| [pingdom-api](https://github.com/Type-Nu11/pingdom-api) | 프록시를 거쳐 도달하는 핵심 API |
| [pingdom-infra](https://github.com/Type-Nu11/pingdom-infra) | Rust edge가 요청을 전달하는 OpenResty Gateway |
| [pingdom-mcp](https://github.com/Type-Nu11/pingdom-mcp) | Gateway와 Backend가 연동하는 MCP 서비스 |

## Documentation

| Document | Description |
|---|---|
| [Rust Edge](docs/rust-edge.md) | 구성, TLS, 요청 제한과 운영 조건 |
| [OpenResty Trust Test](tests/openresty/README.md) | 원본 IP 헤더의 신뢰 경계 검증 |

## Release and Compatibility

변경사항은 저장소의 Release 또는 Git 변경 이력을 기준으로 확인합니다.
호환성에 영향을 주는 변경사항은 관련 문서와 변경 이력을 통해 안내합니다.

Rust edge와 기존 HAProxy는 실행 파일과 설정이 다릅니다. 배포 전 선택한 구성의 도메인,
포트, 인증서 및 upstream 조건을 확인합니다.

## License

이 프로젝트의 사용 및 배포 조건은 [MIT License](LICENSE)를 따릅니다.

---

<div align="center">

Part of **Pingdom**

</div>
