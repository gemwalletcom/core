FROM rust:1.89.0-bookworm AS chef
# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN cargo install cargo-chef --version 0.1.72
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
# Use specific target for better caching
RUN cargo build --release --workspace --exclude gemstone

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
