## Overview

이 저장소는 Pingdom 프로젝트의 **Load Balancer 영역**을 관리합니다.

외부 클라이언트 요청을 가장 먼저 수신하는 엣지 계층으로,
트래픽 제어와 접근 차단을 수행한 뒤 Pingdom Backend Server로 요청을 분배합니다.

HAProxy 기반의 프록시 구성과, HAProxy Runtime API를 통해 상태를 조회하고
제어하는 C++ 운영 에이전트(HAProxy Agent)로 구성됩니다.

애플리케이션 비즈니스 로직을 직접 처리하지 않으며, 요청 전달 경로와
서비스 가용성을 관리하는 역할을 담당합니다.

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
| Responsibility | 트래픽 분배, 접근 제어 및 백엔드 상태 관리 |
| Primary Output | 프록시 구성 및 운영 에이전트 |
| Target | Pingdom Backend Server |

## Scope

### Included

- 클라이언트 요청 수신 및 백엔드 분배
- 클라이언트 단위 요청 속도 제한
- 정적 IP 기반 접근 차단
- 백엔드 헬스 체크 및 장애 노드 격리
- 경로 단위 타임아웃 정책 적용
- HAProxy Runtime API 기반 상태 조회

### Not Included

- 애플리케이션 비즈니스 로직 처리
- 사용자 인증 및 권한 관리
- 데이터 저장소 운영 및 관리
- 서비스 배포 파이프라인 관리

## Key Capabilities

- **트래픽 분배**: Round Robin 방식으로 Backend Server 노드에 요청을 분배합니다.
- **요청 속도 제한**: Stick Table 기반으로 클라이언트 IP당 10초 200회를 초과하는 요청을 `429`로 차단합니다.
- **정적 접근 차단**: ACL 목록에 등록된 IP의 요청을 `403`으로 차단합니다.
- **헬스 체크**: 5초 주기로 백엔드 상태를 확인하고 장애 노드를 분배 대상에서 제외합니다.
- **경로 단위 타임아웃**: 장시간 처리 API에 한해 기본 타임아웃을 개별 확장합니다.
- **운영 에이전트**: Unix Socket으로 HAProxy Runtime API에 접속해 상태 정보를 조회합니다.

## Technology and Tools

| Category | Technology |
|---|---|
| Proxy | HAProxy 3.2 (Alpine) |
| Mode | HTTP |
| Agent | C++20 |
| Build | CMake 3.20 |
| Control | HAProxy Runtime API (Unix Socket) |
| Delivery | Docker / Docker Compose |

---

# Architecture

Pingdom Load Balancer는
요청 처리 경로와 운영 제어 경로를 분리한

**Edge Proxy + Control Agent Architecture**

구조를 사용합니다.

```text
Client
    │
    │ HTTP :80
    ▼
HAProxy Edge (frontend http_in)
    │
    ├──── Rate Limit        Stick Table / 429
    ├──── Static IP Block   ACL List / 403
    ├──── X-Forwarded-For
    └──── Path Timeout Override
    │
    ▼
Backend Pool (spring_cluster)
    │
    ├──── Balance      Round Robin
    ├──── Health Check GET / expect 200
    │
    ▼
Pingdom Backend Server


Control Plane
    │
    ▼
Runtime API (Unix Socket)
    ▲
    │ show info / show stat
    │
HAProxy Agent
    │
    ├──── HAProxyClient   Socket 통신
    ├──── HealthMonitor   상태 폴링
    ├──── StatsParser     CSV 파싱
    └──── BlockManager    차단 목록 관리

Statistics
    │
    ▼
listen stats  127.0.0.1:8404 /stats
```

## Docker

Docker 기반 실행 환경을 제공합니다. 로컬에 HAProxy가 없어도 컨테이너로 프록시를 실행할 수 있습니다.

빌드 및 실행:

```bash
docker compose up -d --build
```

프록시는 호스트의 `80` 포트로 요청을 수신하며, 통계 페이지는 루프백에 한해
`127.0.0.1:8404`로 노출됩니다.

프록시 구성 파일은 볼륨 마운트가 아니라 이미지 빌드 시 `COPY`로 포함됩니다.
따라서 `haproxy/haproxy.cfg`를 수정한 경우 재빌드가 필요합니다.

차단 IP 목록은 `haproxy/acl` 디렉터리를 읽기 전용으로 마운트합니다.

## Getting Started

이 저장소를 확인하거나 실행하기 위해 필요한 최소 절차입니다.

### Requirements

- Docker
- Docker Compose
- HAProxy 3.2 (로컬 실행 시)
- CMake 3.20 이상 (에이전트 빌드 시)
- C++20 지원 컴파일러

### Setup

```bash
git clone https://github.com/Type-Nu11/pingdom-loadbalancer
cd pingdom-loadbalancer
```

TLS 인증서 및 접근 차단 목록 등 환경별 자원을 준비합니다.

### Usage

프록시 실행:

```bash
docker compose up -d --build
```

로컬 실행:

```bash
haproxy -f haproxy/haproxy.cfg
```

에이전트 빌드 및 실행:

```bash
cmake -S haproxy-agent -B haproxy-agent/build
cmake --build haproxy-agent/build

./haproxy-agent/build/haproxy-agent
```

에이전트는 기본적으로 `/tmp/haproxy.sock` 경로의 Runtime API 소켓에 접속합니다.

### Configuration

설정에 필요한 항목은 프록시 구성 파일 및 접근 제어 자원을 기준으로 관리합니다.

- `haproxy/haproxy.cfg`
- `haproxy/acl/blocked_ip.lst`
- `compose.yml`

실제 인증서, 통계 페이지 계정, 백엔드 주소 및 운영 환경 정보는 저장소에 커밋하지 않습니다.

### Verification

저장소 변경사항은 다음 방법으로 검증합니다.

```bash
haproxy -c -f haproxy/haproxy.cfg
```

검증 방식이 여러 개인 경우 목적별로 구분합니다.

| Verification | Purpose |
|---|---|
| `haproxy -c -f haproxy/haproxy.cfg` | 프록시 구성 문법 검증 |
| `docker compose up -d --build` | 서비스 실행 환경 검증 |
| `127.0.0.1:8404/stats` | 백엔드 상태 및 트래픽 확인 |
| `haproxy-agent` 실행 | Runtime API 연동 확인 |

## Repository Structure

```text
.
├── README.md
├── Dockerfile                          # HAProxy 이미지 빌드 정의
├── compose.yml                         # 서비스 실행 구성
├── haproxy
│   ├── haproxy.cfg                     # 프록시 정책 및 백엔드 구성
│   ├── acl
│   │   └── blocked_ip.lst              # 정적 차단 IP 목록
│   ├── maps
│   │   └── blocked.map                 # 맵 기반 차단 정의
│   └── certs                           # TLS 인증서 자원
└── haproxy-agent
    ├── CMakeLists.txt                  # 에이전트 빌드 정의
    ├── include
    │   ├── haproxy                     # Runtime API 클라이언트 및 모델 정의
    │   ├── model                       # 차단 항목 모델 정의
    │   ├── monitor                     # 상태 모니터 정의
    │   └── parser                      # 통계 파서 정의
    └── src
        ├── main.cpp                    # 에이전트 실행 진입점
        ├── haproxy
        │   └── HAProxyClient.cpp       # Unix Socket 기반 명령 실행
        ├── monitor
        │   └── HealthMonitor.cpp       # 주기적 상태 폴링
        ├── parser
        │   └── StatsParser.cpp         # 통계 CSV 파싱
        └── manager
            └── BlockManager.cpp        # 차단 목록 관리
```

실제 구조를 기준으로 주요 디렉터리와 파일만 설명합니다.

`StatsParser`와 `BlockManager`는 현재 기본 빌드 대상에 포함되어 있지 않으며,
별도 구성으로 관리합니다.

## Related Repositories

| Repository | Relationship |
|---|---|
| Pingdom Backend Server | 프록시 요청 전달 대상 |
| [Pingdom MCP Server](https://github.com/Type-Nu11/pingdom-mcp) | Backend Server 연동 AI 처리 계층 |
| Pingdom Infrastructure | 서비스 배포 및 운영 환경 관리 |

공개되어 있거나 접근 가능한 저장소만 연결합니다.

## Documentation

| Document | Description |
|---|---|
| - | 별도 공개 문서 없음 |

## Release and Compatibility

현재 버전은 GA(General Availability) 단계입니다.

호환성에 영향을 주는 변경사항은 Release와 관련 문서를 통해 안내합니다.
변경사항은 저장소의 Release 또는 변경 이력을 기준으로 확인합니다.

## License

이 프로젝트는 MIT License를 따릅니다.

자세한 내용은 [LICENSE](LICENSE) 파일을 참고하세요.

---

Part of Pingdom
