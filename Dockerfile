# syntax=docker/dockerfile:1
FROM rust:1.90.0-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this layer will be cached
RUN cargo chef cook --release --recipe-path recipe.json --bin api --bin daemon

# Copy source and build application
COPY . .
RUN cargo build --release --bin api --bin daemon && \
    mkdir -p /output && \
    cp /app/target/release/api /output/ && \
    cp /app/target/release/daemon /output/

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
COPY --from=builder /app/Settings.yaml /app/

CMD ["sh", "-c", "/app/${BINARY}"]
