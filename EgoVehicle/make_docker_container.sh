#!/usr/bin/env bash
set -euo pipefail
sudo docker build -t sdvlab_egovehicle:0.2 \
  --build-arg USER_UID="$(id -u)" \
  --build-arg USER_GID="$(id -g)" \
  .
