FROM rust:1.94.0 AS builder

WORKDIR /usr/src/app

RUN rustup target add x86_64-unknown-linux-musl

COPY Cargo.toml Cargo.lock ./
COPY src src
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM alpine:3.17 AS start

COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/CommandProjectBackend /usr/local/bin/CommandProjectBackend

EXPOSE 8080

CMD ["CommandProjectBackend"]