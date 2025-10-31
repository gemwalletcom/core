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
RUN cargo chef cook --release --recipe-path recipe.json --package dynode

# Copy source and build application
COPY . .
RUN cargo build --release --package dynode && \
    mkdir -p /output && \
    cp /app/target/release/dynode /output/

FROM debian:bookworm AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /output/dynode /app/

COPY --from=builder /app/apps/dynode/config.yml /app/
COPY --from=builder /app/apps/dynode/domains.yml /app/

CMD ["/app/dynode"]