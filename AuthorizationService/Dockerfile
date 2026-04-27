FROM rust:1.94.0 AS builder

WORKDIR /usr/src/app

RUN rustup target add x86_64-unknown-linux-musl

COPY ./logger/Cargo.toml ./logger/Cargo.toml
COPY ./logger/src ./logger/src

COPY ./AuthorizationService/Cargo.toml ./AuthorizationService/
COPY ./AuthorizationService/src ./AuthorizationService/src

WORKDIR /usr/src/app/AuthorizationService

RUN cargo build --target x86_64-unknown-linux-musl --release

FROM alpine:3.17 AS start

COPY --from=builder /usr/src/app/AuthorizationService/target/x86_64-unknown-linux-musl/release/AuthorizationService /usr/local/bin/AuthorizationService

CMD ["AuthorizationService"]