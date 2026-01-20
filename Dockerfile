# syntax=docker/dockerfile:1
FROM lukemathwalker/cargo-chef:latest-rust-1.92.0-bookworm AS chef
WORKDIR /app
ENV CARGO_INCREMENTAL=0

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json \
    --package api --package daemon --package dynode
COPY . .
RUN cargo build --release --package api --package daemon --package dynode && cp target/release/api target/release/daemon target/release/dynode /app/

# Shared runtime base
FROM debian:bookworm-slim AS runtime-base
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends openssl ca-certificates && rm -rf /var/lib/apt/lists/*

# Core runtime image
FROM runtime-base AS core
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/api /app/
COPY --from=builder /app/daemon /app/
COPY --from=builder /app/Settings.yaml /app/
CMD ["sh", "-c", "/app/${BINARY}"]

# Dynode runtime image
FROM runtime-base AS dynode
WORKDIR /app
COPY --from=builder /app/dynode /app/
COPY --from=builder /app/apps/dynode/config.yml /app/
CMD ["/app/dynode"]
