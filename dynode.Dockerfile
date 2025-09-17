FROM rust:1.89.0-bookworm AS builder
WORKDIR /app

COPY . .

RUN cargo build --release --package dynode

FROM debian:bookworm AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/dynode /app/

COPY --from=builder /app/apps/dynode/config.yml /app/
COPY --from=builder /app/apps/dynode/domains.yml /app/

CMD ["/app/dynode"]