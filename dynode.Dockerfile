# syntax=docker/dockerfile:1
FROM lukemathwalker/cargo-chef:latest-rust-1.91.0-bookworm AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this layer will be cached
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo chef cook --release --recipe-path recipe.json --package dynode

# Copy source and build application
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release --package dynode && \
    mkdir -p /output && \
    cp target/release/dynode /output/

FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates openssl curl && rm -rf /var/lib/apt/lists/*

COPY --from=builder /output/dynode /app/
COPY --from=builder /app/apps/dynode/config.yml /app/apps/dynode/domains.yml /app/

CMD ["/app/dynode"]