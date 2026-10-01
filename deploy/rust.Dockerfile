FROM rust:1.98-bookworm AS builder
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
RUN cargo build --locked --release -p proxy-server

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /src/target/release/proxy-server /usr/local/bin/proxy-server
COPY configs/edge.toml /app/configs/edge.toml
EXPOSE 443
CMD ["proxy-server", "/app/configs/edge.toml"]
