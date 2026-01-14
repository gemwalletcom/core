# syntax=docker/dockerfile:1
FROM rust:1.92.0-bookworm AS builder
WORKDIR /app

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked,id=rust-target \
    cargo build --release --bin api --bin daemon --bin dynode && \
    cp /app/target/release/api /app/api && \
    cp /app/target/release/daemon /app/daemon && \
    cp /app/target/release/dynode /app/dynode

# Core runtime image
FROM debian:bookworm AS core
WORKDIR /app
RUN apt-get update && apt-get install -y \
    openssl ca-certificates libpq-dev postgresql curl \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/api /app/
COPY --from=builder /app/daemon /app/
COPY --from=builder /app/Settings.yaml /app/
CMD ["sh", "-c", "/app/${BINARY}"]

# Dynode runtime image
FROM debian:bookworm AS dynode
WORKDIR /app
RUN apt-get update && apt-get install -y \
    openssl ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/dynode /app/
COPY --from=builder /app/apps/dynode/config.yml /app/
CMD ["/app/dynode"]
