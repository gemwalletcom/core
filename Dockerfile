FROM lukemathwalker/cargo-chef:latest-rust-1.71.0 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release

FROM debian:bullseye AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/api /app
COPY --from=builder /app/target/release/deamon /app
COPY --from=builder /app/target/release/parser /app
COPY --from=builder /app/target/release/setup /app
COPY --from=builder /app/Settings.toml /app
RUN apt-get update && apt-get install -y openssl ca-certificates libpq-dev postgresql

CMD ["sh", "-c", "/app/${BINARY}"]
