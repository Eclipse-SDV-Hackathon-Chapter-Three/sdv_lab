#!/usr/bin/env bash
set -euo pipefail
sudo docker run --rm -it \
  --mount type=bind,source="$PWD",target="$PWD" \
  -w "$PWD" \
  sdvlab_egovehicle:0.2 bash
