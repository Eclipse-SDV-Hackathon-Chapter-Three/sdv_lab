# Dockerfile
FROM ubuntu:22.04

# Avoid interactive apt prompts
ENV DEBIAN_FRONTEND=noninteractive

# Toolchain & essentials (includes OpenSSL dev)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      clang-12 libclang-12-dev llvm-12 llvm-12-dev \
      build-essential cmake git pkg-config \
      libssl-dev \
      curl ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Default to Clang 12
ENV CC=clang-12 \
    CXX=clang++-12

# Create a user matching your host UID/GID
ARG USER_UID=1000
ARG USER_GID=1000
ARG USERNAME=dev
RUN groupadd -g ${USER_GID} ${USERNAME} && \
    useradd -m -u ${USER_UID} -g ${USER_GID} -s /bin/bash ${USERNAME} && \
    mkdir -p /workspace && chown -R ${USER_UID}:${USER_GID} /workspace

# Your requested env vars + Rust paths
ENV USERNAME=${USERNAME} \
    CARGO_HOME=/home/${USERNAME}/.cargo \
    RUSTUP_HOME=/home/${USERNAME}/.rustup \
    PATH=/home/${USERNAME}/.cargo/bin:$PATH \
    LLVM_CONFIG_PATH=/usr/bin/llvm-config-12 \
    LIBCLANG_PATH=/usr/lib/llvm-12/lib \
    LIBCLANG_STATIC_PATH=/usr/lib/llvm-12/lib \
    CLANG_PATH=/usr/bin/clang-12

USER ${USERNAME}
WORKDIR /workspace

# Install rustup (noninteractive) and default to stable toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain stable

# (optional) sanity checks
# RUN rustc --version && cargo --version && clang-12 --version
