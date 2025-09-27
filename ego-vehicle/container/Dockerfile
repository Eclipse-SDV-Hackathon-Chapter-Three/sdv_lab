# Dockerfile
FROM ubuntu:22.04 as DEV

ENV DEBIAN_FRONTEND=noninteractive

# Toolchain & essentials (+ openssh-client and xz-utils for rustup)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      clang-12 libclang-12-dev llvm-12 llvm-12-dev \
      build-essential cmake git pkg-config \
      libssl-dev \
      curl ca-certificates \
      openssh-client xz-utils && \
    rm -rf /var/lib/apt/lists/*

# Non-interactive host key acceptance for first-time git fetches
ENV GIT_SSH_COMMAND="ssh -o StrictHostKeyChecking=accept-new"

# Pre-populate known_hosts for common git forges (optional, speeds up first fetch)
RUN mkdir -p /etc/ssh && \
    ssh-keyscan -H github.com gitlab.com ssh.dev.azure.com 2>/dev/null >> /etc/ssh/ssh_known_hosts

# Default to Clang 12
ENV CC=clang-12 \
    CXX=clang++-12

# --- Create user matching host UID/GID (or keep root cleanly) ---
ARG USER_UID=1000
ARG USER_GID=1000
ARG USERNAME=dev

# If you pass USER_UID=0/USER_GID=0 you MUST also pass USERNAME=root.
# Fail fast with a clear message if not.
RUN set -eux; \
    if [ "$USER_UID" = "0" ] || [ "$USER_GID" = "0" ]; then \
        if [ "$USERNAME" != "root" ]; then \
            echo "ERROR: When USER_UID/USER_GID is 0, pass --build-arg USERNAME=root"; \
            exit 1; \
        fi; \
        mkdir -p /workspace; chown -R 0:0 /workspace; \
    else \
        # Create group if GID not taken
        getent group "$USER_GID" >/dev/null || groupadd -g "$USER_GID" "$USERNAME"; \
        # Create user if UID not taken
        if ! id -u "$USER_UID" >/dev/null 2>&1; then \
            useradd -m -u "$USER_UID" -g "$USER_GID" -s /bin/bash "$USERNAME"; \
        fi; \
        mkdir -p /workspace; chown -R "$USER_UID":"$USER_GID" /workspace; \
    fi

# Have Cargo use the git CLI (so it respects SSH/agent)
RUN mkdir -p /etc/cargo && \
    printf "[net]\ngit-fetch-with-cli = true\n" > /etc/cargo/config.toml

# Also set the same for the default user later
# (duplicated into the user's $HOME below after we switch users)

# Prefer SSH for GitHub even if a dep uses https
RUN git config --system url."ssh://git@github.com/".insteadOf https://github.com/

# Add both common cargo bin paths to PATH (works for root or $USERNAME)
ENV PATH=/home/${USERNAME}/.cargo/bin:/root/.cargo/bin:$PATH \
    LLVM_CONFIG_PATH=/usr/bin/llvm-config-12 \
    LIBCLANG_PATH=/usr/lib/llvm-12/lib \
    LIBCLANG_STATIC_PATH=/usr/lib/llvm-12/lib \
    CLANG_PATH=/usr/bin/clang-12

# Switch to chosen user (root if you built with USERNAME=root)
USER ${USERNAME}
WORKDIR /workspace

# Ensure per-user cargo config mirrors system setting
RUN mkdir -p ~/.cargo && printf "[net]\ngit-fetch-with-cli = true\n" > ~/.cargo/config.toml && \
    mkdir -p ~/.ssh && chmod 700 ~/.ssh

# Install rustup (full default profile) and extras
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile default --default-toolchain stable && \
    ~/.cargo/bin/rustup component add clippy rustfmt
    
# (optional) sanity checks
# RUN rustc --version && cargo --version && clang-12 --version

FROM DEV as COMPILE

COPY . $WORKDIR
RUN --mount=type=ssh,mode=0666 \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo fetch

RUN --mount=type=cache,target=${WORKDIR}/target/release \
    cargo build --release

FROM COMPILE as PROD
ENV RUST_LOG=info
ENTRYPOINT ["target/release/ego-vehicle"]

