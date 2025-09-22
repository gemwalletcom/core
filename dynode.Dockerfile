# syntax=docker/dockerfile:1
FROM rust:1.89.0-bookworm AS builder
WORKDIR /app

# Copy source
COPY --link . .

# Build with full caching using BuildKit cache mounts
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked,id=rust-target-dynode \
    cargo build --release --package dynode

# Copy binary from cache to layer
RUN --mount=type=cache,target=/app/target,sharing=locked,id=rust-target-dynode \
    mkdir -p /output && \
    cp /app/target/release/dynode /output/

FROM debian:bookworm AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /output/dynode /app/

COPY --from=builder /app/apps/dynode/config.yml /app/
COPY --from=builder /app/apps/dynode/domains.yml /app/

CMD ["/app/dynode"]