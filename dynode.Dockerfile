# syntax=docker/dockerfile:1
FROM rust:1.90.0-bookworm AS builder
WORKDIR /app

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked,id=rust-target-dynode \
    cargo build --release --package dynode && \
    cp /app/target/release/dynode /app/dynode

FROM debian:bookworm AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/dynode /app/
COPY --from=builder /app/apps/dynode/config.yml /app/

CMD ["/app/dynode"]