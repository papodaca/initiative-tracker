#!/usr/bin/env bash
# Local smoke: build an AppImage inside ubuntu:26.04 (mirrors CI).
# Usage: ./smoke-docker.sh
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/../.." && pwd)

docker run --rm \
  -e "IT_DOCKER_ROOT=/workspace" \
  -v "${REPO_ROOT}:/workspace" \
  -w /workspace \
  ubuntu:26.04 \
  bash -euo pipefail /workspace/packaging/appimage/smoke-docker-inner.sh
