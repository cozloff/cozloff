FROM nvidia/cuda:13.0.2-devel-ubuntu24.04

ENV CUDA_PATH=/usr/local/cuda
ENV PATH=/root/.cargo/bin:/usr/local/cuda/bin:$PATH
ENV LD_LIBRARY_PATH=/usr/local/cuda/lib64

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        curl \
        pkg-config \
        libssl-dev; \
    rm -rf /var/lib/apt/lists/*; \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --profile minimal --default-toolchain stable

WORKDIR /app

CMD ["bash"]
