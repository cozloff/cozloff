FROM rust:1-slim

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        libdrm2 \
        libegl1 \
        libgl1 \
        libgl1-mesa-dri \
        libglx-mesa0 \
        libvulkan1 \
        libx11-6 \
        libxcursor1 \
        libxi6 \
        libxkbcommon0 \
        libxrandr2 \
        libxcb1 \
        mesa-vulkan-drivers \
        mesa-utils \
        pkg-config \
        vulkan-tools \
        xauth; \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

CMD ["bash"]
