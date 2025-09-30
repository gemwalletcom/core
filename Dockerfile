# syntax=docker/dockerfile:1
FROM rust:1.90.0-bookworm AS builder
WORKDIR /app

# Copy source
COPY . .

# Build with cache mounts - dependencies and source will be cached separately by Cargo
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked,id=rust-target-core \
    cargo build --release --bin api --bin daemon --bin parser --bin setup

# Copy binaries from cache to layer
RUN --mount=type=cache,target=/app/target,sharing=locked,id=rust-target-core \
    mkdir -p /output && \
    cp /app/target/release/api /output/ && \
    cp /app/target/release/daemon /output/ && \
    cp /app/target/release/parser /output/ && \
    cp /app/target/release/setup /output/

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm AS runtime
WORKDIR /app

# Install dependencies first for better caching
RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    libpq-dev \
    postgresql \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy binaries
COPY --from=builder /output/api /app/
COPY --from=builder /output/daemon /app/
COPY --from=builder /output/parser /app/
COPY --from=builder /output/setup /app/
COPY --from=builder /app/Settings.yaml /app/

CMD ["sh", "-c", "/app/${BINARY}"]
