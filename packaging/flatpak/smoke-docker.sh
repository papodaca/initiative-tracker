#!/usr/bin/env bash
# Local smoke: build a .flatpak bundle inside ubuntu:latest (mirrors release.yml).
# Needs privileged for nested bubblewrap.
# Usage: ./smoke-docker.sh
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/../.." && pwd)

docker run --rm --privileged \
  -e IT_DOCKER_ROOT=/workspace \
  -v "${REPO_ROOT}:/workspace" \
  -w /workspace \
  ubuntu:latest \
  bash -euo pipefail /workspace/packaging/flatpak/smoke-docker-inner.sh
