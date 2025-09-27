#!/usr/bin/env bash
set -euo pipefail
podman run --rm -it \
  -v "$PWD":/workspace \
  -v "$SSH_AUTH_SOCK":/ssh-agent \
  -e SSH_AUTH_SOCK=/ssh-agent \
  sdv-dev bash
