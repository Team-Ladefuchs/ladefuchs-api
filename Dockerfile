FROM rust:alpine as builder
WORKDIR /build

COPY ./src ./src
COPY ./sql ./sql
COPY ./migrations ./migrations
COPY ./sqlx-data.json ./sqlx-data.json
COPY ./build.rs /.build.rs
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

ENV SQLX_OFFLINE true

RUN apk update && \
    apk add musl musl-dev
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine as runtime
WORKDIR /deploy

COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/ladefuchs-api /deploy/ladefuchs-api

EXPOSE 3000

ENV DOMAIN="http://localhost:3000"
ENV LISTEN=0.0.0.0

CMD ["/deploy/ladefuchs-api"]
