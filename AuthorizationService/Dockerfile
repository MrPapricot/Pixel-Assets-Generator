FROM rust:1.94.0 AS builder

WORKDIR /usr/src/app

RUN rustup target add x86_64-unknown-linux-musl

COPY ./logger/Cargo.toml ./logger/Cargo.toml
COPY ./logger/src ./logger/src

COPY ./auth_rpc/Cargo.toml ./auth_rpc/Cargo.toml
COPY ./auth_rpc/build.rs ./auth_rpc/build.rs
COPY ./auth_rpc/proto ./auth_rpc/proto
COPY ./auth_rpc/src ./auth_rpc/src

COPY ./AuthorizationService/Cargo.toml ./AuthorizationService/
COPY ./AuthorizationService/src ./AuthorizationService/src

WORKDIR /usr/src/app/AuthorizationService

RUN apt-get update && apt-get install -y --no-install-recommends protobuf-compiler libprotobuf-dev
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM alpine:3.17 AS start

RUN GRPC_HEALTH_PROBE_VERSION=v0.4.50 && TARGETOS=linux && TARGETARCH=amd64 && \
    wget -qO/bin/grpc_health_probe https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/${GRPC_HEALTH_PROBE_VERSION}/grpc_health_probe-${TARGETOS}-${TARGETARCH} && \
    chmod +x /bin/grpc_health_probe

COPY --from=builder /usr/src/app/AuthorizationService/target/x86_64-unknown-linux-musl/release/AuthorizationService /usr/local/bin/AuthorizationService

CMD ["AuthorizationService"]