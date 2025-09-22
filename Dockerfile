FROM rust:1.89.0-bookworm AS builder
WORKDIR /app

COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release --bin api --bin daemon --bin parser --bin setup

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm AS runtime
WORKDIR /app

# Install dependencies first for better caching
RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    libpq-dev \
    postgresql \
    && rm -rf /var/lib/apt/lists/*

# Copy binaries
COPY --from=builder /app/target/release/api /app/
COPY --from=builder /app/target/release/daemon /app/
COPY --from=builder /app/target/release/parser /app/
COPY --from=builder /app/target/release/setup /app/
COPY --from=builder /app/Settings.yaml /app/

CMD ["sh", "-c", "/app/${BINARY}"]
