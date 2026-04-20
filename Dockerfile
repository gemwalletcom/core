# syntax=docker/dockerfile:1
FROM lukemathwalker/cargo-chef:latest-rust-1.94.1-bookworm AS chef
WORKDIR /app
ENV CARGO_INCREMENTAL=0

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder-core
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json \
    --package api --package daemon
COPY . .
RUN cargo build --release --package api --package daemon && cp target/release/api target/release/daemon /app/

FROM chef AS builder-dynode
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json \
    --package dynode
COPY . .
RUN cargo build --release --package dynode && cp target/release/dynode /app/

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
COPY --from=builder-core /app/api /app/
COPY --from=builder-core /app/daemon /app/
COPY --from=builder-core /app/Settings.yaml /app/
CMD ["sh", "-c", "/app/${BINARY}"]

# Dynode runtime image
FROM runtime-base AS dynode
WORKDIR /app
COPY --from=builder-dynode /app/dynode /app/
COPY --from=builder-dynode /app/apps/dynode/config.yml /app/
CMD ["/app/dynode"]
