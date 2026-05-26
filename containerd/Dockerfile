FROM rust:1.95-bookworm AS builder

ARG NERDCTL_VERSION=2.0.0

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        pkg-config \
        libssl-dev \
        protobuf-compiler \
    && curl -fsSL "https://github.com/containerd/nerdctl/releases/download/v${NERDCTL_VERSION}/nerdctl-${NERDCTL_VERSION}-linux-amd64.tar.gz" \
        | tar -xz -C /usr/local/bin nerdctl \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

CMD ["nerdctl", "--address", "/run/containerd/containerd.sock", "info"]
