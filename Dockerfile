FROM rust:1-bookworm

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        pkg-config \
        libxext6 \
        libvulkan1 \
        vulkan-tools; \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

CMD ["bash"]
