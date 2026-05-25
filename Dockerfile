FROM rust:1-slim

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        pkg-config; \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

CMD ["bash"]
