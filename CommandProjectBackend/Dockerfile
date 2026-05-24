FROM rust:1.94.0 AS builder

WORKDIR /usr/src/app

RUN apt-get update && apt-get install -y musl-tools

RUN rustup target add x86_64-unknown-linux-musl

COPY ./logger/Cargo.toml ./logger/Cargo.toml
COPY ./logger/src ./logger/src

COPY ./auth_rpc/Cargo.toml ./auth_rpc/Cargo.toml
COPY ./auth_rpc/build.rs ./auth_rpc/build.rs
COPY ./auth_rpc/proto ./auth_rpc/proto
COPY ./auth_rpc/src ./auth_rpc/src

COPY ./CommandProjectBackend/Cargo.toml ./CommandProjectBackend/
COPY ./CommandProjectBackend/src ./CommandProjectBackend/src

WORKDIR /usr/src/app/CommandProjectBackend

RUN apt-get update && apt-get install -y --no-install-recommends protobuf-compiler libprotobuf-dev
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM alpine:3.17 AS start

COPY --from=builder /usr/src/app/CommandProjectBackend/target/x86_64-unknown-linux-musl/release/CommandProjectBackend /usr/local/bin/CommandProjectBackend

EXPOSE 8080

CMD ["CommandProjectBackend"]