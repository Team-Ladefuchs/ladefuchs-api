FROM rust:1.60-slim-buster as builder
WORKDIR /api

COPY ./src ./src
COPY ./sql ./sql
COPY ./migrations ./migrations
COPY ./sqlx-data.json ./sqlx-data.json
COPY ./build.rs /.build.rs
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

ENV SQLX_OFFLINE true
RUN cargo build --release

FROM rust:1.60-slim-buster as runtime
WORKDIR /api

EXPOSE 3000

COPY --from=builder /api/target/release/ladefuchs-api /api/ladefuchs-api
CMD ["/api/ladefuchs-api"]
