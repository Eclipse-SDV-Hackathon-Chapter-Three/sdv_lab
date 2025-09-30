#!/usr/bin/env bash
# Works whether you're using sudo or not
UID=$(id -u "${SUDO_USER:-$USER}")
GID=$(id -g "${SUDO_USER:-$USER}")

TARGET=${1:-DEV}

sudo podman build \
  --target $TARGET \
  --build-arg USER_UID=$UID \
  --build-arg USER_GID=$GID \
  --build-arg USERNAME=dev \
  -t uprotocol-sensors .