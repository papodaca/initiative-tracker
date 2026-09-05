#!/usr/bin/env bash
# Runs inside ubuntu:latest with the repo bind-mounted.
set -euo pipefail

ROOT=${IT_DOCKER_ROOT:-/workspace}

export DEBIAN_FRONTEND=noninteractive
apt-get update
apt-get install -y --no-install-recommends \
  ca-certificates curl gnupg \
  flatpak flatpak-builder ostree elfutils \
  git sudo

git config --global --add safe.directory "${ROOT}" 2>/dev/null || true

# Host compose. libglycin still bwraps for a normal user in this nested Docker
# even with GLYCIN_DISABLE_SANDBOX; root is how the pre-build probe succeeds.
run_appstream_compose() {
  local files_root=$1
  GLYCIN_DISABLE_SANDBOX=1 appstreamcli compose \
    --prefix=/ \
    --origin=im.apodaca.InitiativeTracker \
    --result-root="${files_root}" \
    --data-dir="${files_root}/share/app-info/xmls" \
    --icons-dir="${files_root}/share/app-info/icons/flatpak" \
    --print-report=full \
    --components=im.apodaca.InitiativeTracker,im.apodaca.InitiativeTracker.desktop \
    "${files_root}"
}

compose_probe=$(mktemp -d)
bash "${ROOT}/packaging/install-data.sh" / "${compose_probe}/files"
run_appstream_compose "${compose_probe}/files"
rm -rf "${compose_probe}"

flatpak_version() {
  local tag
  tag=$(git -C "${ROOT}" describe --tags --exact-match HEAD 2>/dev/null || true)
  if [[ ${tag} =~ ^v([0-9][^[:space:]]*)$ ]]; then
    printf '%s' "${BASH_REMATCH[1]}"
  else
    printf '0.1.0+git%s.%s' \
      "$(git -C "${ROOT}" rev-list --count HEAD)" \
      "$(git -C "${ROOT}" rev-parse --short HEAD)"
  fi
}

useradd -m builder
echo "builder ALL=(ALL) NOPASSWD: ALL" >> /etc/sudoers
host_uid=$(stat -c %u "${ROOT}")
host_gid=$(stat -c %g "${ROOT}")
restore_ownership() {
  chown -R "${host_uid}:${host_gid}" "${ROOT}"
}
trap restore_ownership EXIT
chown -R builder:builder "${ROOT}"

VERSION=$(flatpak_version)
export ROOT VERSION
sudo -u builder env HOME=/home/builder ROOT="${ROOT}" VERSION="${VERSION}" \
  bash -euo pipefail <<'EOF'
flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak install -y --user flathub \
  org.gnome.Platform//50 \
  org.gnome.Sdk//50 \
  org.freedesktop.Sdk.Extension.rust-stable//25.08
cd "${ROOT}/packaging/flatpak"
flatpak-builder --user --force-clean --disable-rofiles-fuse --repo=repo build-dir \
  im.apodaca.InitiativeTracker.json
EOF

# Compose as root. Glycin still launches bwrap for uid builder inside this
# nested Docker (GLYCIN_DISABLE_SANDBOX is ignored by libglycin); root skips it.
files_root="${ROOT}/packaging/flatpak/build-dir/files"
run_appstream_compose "${files_root}"
chown -R builder:builder "${files_root}/share/app-info"

sudo -u builder env HOME=/home/builder ROOT="${ROOT}" VERSION="${VERSION}" \
  bash -euo pipefail <<'EOF'
cd "${ROOT}/packaging/flatpak"
flatpak build-export repo build-dir
flatpak build-bundle repo \
  "im.apodaca.InitiativeTracker-${VERSION}.flatpak" \
  im.apodaca.InitiativeTracker
EOF

shopt -s nullglob
bundles=("${ROOT}"/packaging/flatpak/*.flatpak)
if [[ ${#bundles[@]} -eq 0 ]]; then
  echo "No Flatpak bundle produced" >&2
  ls -la "${ROOT}/packaging/flatpak" >&2
  exit 1
fi
for b in "${bundles[@]}"; do
  echo "Built $(basename "${b}") ($(du -h "${b}" | awk '{print $1}'))"
done
