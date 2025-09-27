#!/usr/bin/env bash
# Works whether you're using sudo or not
UID=$(id -u "${SUDO_USER:-$USER}")
GID=$(id -g "${SUDO_USER:-$USER}")

TARGET=${1:-DEV}

podman build \
  --target $TARGET \
  --build-arg USER_UID=$UID \
  --build-arg USER_GID=$GID \
  --build-arg USERNAME=dev \
  --ssh default \
  -t sdv-dev .
