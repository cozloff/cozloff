FROM rust:1.95-bookworm AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        iputils-ping \
        iputils-tracepath \
        traceroute \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/network /usr/local/bin/network

ENV API_BIND_ADDRESS=0.0.0.0:8080

EXPOSE 8080

CMD ["network"]
