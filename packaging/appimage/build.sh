#!/usr/bin/env bash
# Build an Initiative Tracker AppImage from a git checkout.
#
# Usage:
#   cd packaging/appimage
#   ./build.sh
#
# Produces: InitiativeTracker-$VERSION-$ARCH.AppImage in this directory
# (ARCH is uname -m: x86_64 or aarch64).
# Requires Ubuntu 26.04-class deps: rustc, cargo, pkg-config,
# GTK4/libadwaita, GStreamer plugins (base, good, libav) for scene video,
# curl, file, patchelf, python3.
# Video list thumbnails are not bundled (gst-video-thumbnailer pulls glycin/bwrap).
# At runtime the app uses the host XDG thumbnailers, same as Files.
#
# AppImages built on Ubuntu 26.04 target that glibc floor (GTK 4.22 / libadwaita 1.9).
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/../.." && pwd)
HOST_ARCH=$(uname -m)

git config --global --add safe.directory "${REPO_ROOT}" 2>/dev/null || true
git -C "${REPO_ROOT}" config --global --add safe.directory '*' 2>/dev/null || true

case "${HOST_ARCH}" in
  x86_64|aarch64) ;;
  *)
    echo "AppImage packaging supports x86_64 and aarch64 (got: ${HOST_ARCH})" >&2
    exit 1
    ;;
esac

LINUXDEPLOY_VERSION=${LINUXDEPLOY_VERSION:-1-alpha-20251107-1}
LINUXDEPLOY_BIN="linuxdeploy-${HOST_ARCH}.AppImage"
LINUXDEPLOY_URL="https://github.com/linuxdeploy/linuxdeploy/releases/download/${LINUXDEPLOY_VERSION}/${LINUXDEPLOY_BIN}"
GTK_PLUGIN_URL="https://raw.githubusercontent.com/linuxdeploy/linuxdeploy-plugin-gtk/7a3fbc31a9e5/linuxdeploy-plugin-gtk.sh"

CACHE_DIR=${LINUXDEPLOY_CACHE_DIR:-"${SCRIPT_DIR}/.linuxdeploy-cache"}
APPDIR="${SCRIPT_DIR}/AppDir"
BUILDDIR="${SCRIPT_DIR}/builddir"

appimage_version() {
  local tag
  tag=$(git -C "${REPO_ROOT}" describe --tags --exact-match HEAD 2>/dev/null || true)
  if [[ ${tag} =~ ^v([0-9][^[:space:]]*)$ ]]; then
    printf '%s' "${BASH_REMATCH[1]}"
  else
    printf '0.1.0+git%s.%s' \
      "$(git -C "${REPO_ROOT}" rev-list --count HEAD)" \
      "$(git -C "${REPO_ROOT}" rev-parse --short HEAD)"
  fi
}

patch_gtk_plugin() {
  # Ubuntu 26.04 / GTK 4.22+: no /usr/lib/.../gtk-4.0 modules tree, and
  # gdk-pixbuf no longer ships a 2.10.0/loaders dir (built-in loaders).
  # Upstream linuxdeploy-plugin-gtk still assumes both exist.
  local gtk=$1
  local tmp

  if ! grep -q 'SKIP_MISSING_GTK_MODULES' "${gtk}"; then
    tmp=$(mktemp)
    awk '
      BEGIN { patched = 0 }
      {
        if (!patched && $0 ~ /for elem in "\$\{src\[@\]\}"; do/) {
          print
          getline nextline
          if (nextline ~ /LD_GTK_LIBRARY_PATH/) {
            print "        # SKIP_MISSING_GTK_MODULES"
            print "        if [ ! -e \"$elem\" ]; then"
            print "            echo \"Skipping missing path: $elem\""
            print "            continue"
            print "        fi"
            print nextline
            patched = 1
            next
          }
          print nextline
          next
        }
        print
      }
      END {
        if (!patched) {
          print "patch_gtk_plugin: failed to locate copy_lib_tree loop" > "/dev/stderr"
          exit 1
        }
      }
    ' "${gtk}" >"${tmp}"
    mv "${tmp}" "${gtk}"
  fi

  if ! grep -q 'SKIP_MISSING_PIXBUF_LOADERS' "${gtk}"; then
    tmp=$(mktemp)
    python3 - "${gtk}" "${tmp}" <<'PY'
import sys
from pathlib import Path
src, dst = Path(sys.argv[1]), Path(sys.argv[2])
text = src.read_text()
old = '''if [ -x "$gdk_pixbuf_query" ]; then
    echo "Updating pixbuf cache in $APPDIR/${gdk_pixbuf_cache_file/$LD_GTK_LIBRARY_PATH//usr/lib}"
    "$gdk_pixbuf_query" > "$APPDIR/${gdk_pixbuf_cache_file/$LD_GTK_LIBRARY_PATH//usr/lib}"
else
    echo "WARNING: gdk-pixbuf-query-loaders not found"
fi
if [ ! -f "$APPDIR/${gdk_pixbuf_cache_file/$LD_GTK_LIBRARY_PATH//usr/lib}" ]; then
    echo "WARNING: loaders.cache file is missing"
fi
sed -i "s|$gdk_pixbuf_moduledir/||g" "$APPDIR/${gdk_pixbuf_cache_file/$LD_GTK_LIBRARY_PATH//usr/lib}"'''
new = '''# SKIP_MISSING_PIXBUF_LOADERS
if [ -d "$gdk_pixbuf_binarydir" ] && [ -x "$gdk_pixbuf_query" ]; then
    echo "Updating pixbuf cache in $APPDIR/${gdk_pixbuf_cache_file/$LD_GTK_LIBRARY_PATH//usr/lib}"
    mkdir -p "$(dirname "$APPDIR/${gdk_pixbuf_cache_file/$LD_GTK_LIBRARY_PATH//usr/lib}")"
    "$gdk_pixbuf_query" > "$APPDIR/${gdk_pixbuf_cache_file/$LD_GTK_LIBRARY_PATH//usr/lib}"
    sed -i "s|$gdk_pixbuf_moduledir/||g" "$APPDIR/${gdk_pixbuf_cache_file/$LD_GTK_LIBRARY_PATH//usr/lib}"
elif [ ! -d "$gdk_pixbuf_binarydir" ]; then
    echo "WARNING: gdk-pixbuf loaders dir missing (built-in loaders); not setting GDK_PIXBUF_MODULE_FILE"
    sed -i "/GDK_PIXBUF_MODULE_FILE/d" "$HOOKFILE"
else
    echo "WARNING: gdk-pixbuf-query-loaders not found"
fi'''
if old not in text:
    raise SystemExit('patch_gtk_plugin: gdk-pixbuf cache block not found')
dst.write_text(text.replace(old, new, 1))
PY
    mv "${tmp}" "${gtk}"
  fi

  if ! grep -q 'SKIP_MISSING_RPATH_DIRS' "${gtk}"; then
    tmp=$(mktemp)
    python3 - "${gtk}" "${tmp}" <<'PY'
import sys
from pathlib import Path
src, dst = Path(sys.argv[1]), Path(sys.argv[2])
text = src.read_text()
old = '''for directory in "${PATCH_ARRAY[@]}"; do
    while IFS= read -r -d '' file; do
        ln $verbose -sf "${file/$LD_GTK_LIBRARY_PATH\\//}" "$APPDIR/usr/lib"
    done < <(find "$directory" -name '*.so' -print0)
done'''
new = '''# SKIP_MISSING_RPATH_DIRS
for directory in "${PATCH_ARRAY[@]}"; do
    if [ -z "$directory" ] || [ ! -d "$directory" ]; then
        continue
    fi
    while IFS= read -r -d '' file; do
        ln $verbose -sf "${file/$LD_GTK_LIBRARY_PATH\\//}" "$APPDIR/usr/lib"
    done < <(find "$directory" -name '*.so' -print0)
done'''
if old not in text:
    raise SystemExit('patch_gtk_plugin: rpath loop not found')
dst.write_text(text.replace(old, new, 1))
PY
    mv "${tmp}" "${gtk}"
  fi

  chmod +x "${gtk}"
}

fetch_tooling() {
  mkdir -p "${CACHE_DIR}"
  local ld="${CACHE_DIR}/${LINUXDEPLOY_BIN}"
  local gtk="${CACHE_DIR}/linuxdeploy-plugin-gtk.sh"

  if [[ ! -f ${ld} ]]; then
    echo "Downloading linuxdeploy ${LINUXDEPLOY_VERSION} (${HOST_ARCH})…"
    curl -fL --retry 3 -o "${ld}.partial" "${LINUXDEPLOY_URL}"
    mv "${ld}.partial" "${ld}"
  fi
  if [[ ! -f ${gtk} ]]; then
    echo "Downloading linuxdeploy-plugin-gtk…"
    curl -fL --retry 3 -o "${gtk}.partial" "${GTK_PLUGIN_URL}"
    mv "${gtk}.partial" "${gtk}"
  fi
  patch_gtk_plugin "${gtk}"
  chmod +x "${ld}" "${gtk}"

  if [[ ! -x ${CACHE_DIR}/squashfs-root/plugins/linuxdeploy-plugin-appimage/usr/bin/appimagetool ]]; then
    echo "Extracting linuxdeploy (for appimagetool)…"
    rm -rf "${CACHE_DIR}/squashfs-root"
    (
      cd "${CACHE_DIR}"
      APPIMAGE_EXTRACT_AND_RUN=1 "./${LINUXDEPLOY_BIN}" --appimage-extract >/dev/null
    )
  fi
}

strip_graphics_driver_libs() {
  local patterns=(
    'libvulkan.so*'
    'libvulkan_*.so*'
    'libVkLayer*.so*'
    'libGLX_mesa.so*'
    'libEGL_mesa.so*'
    'libgallium*.so*'
    'libdrm_amdgpu.so*'
    'libdrm_radeon.so*'
    'libdrm_intel.so*'
    'libdrm_nouveau.so*'
    'libnvidia-*.so*'
    'libcuda.so*'
  )
  local pat
  for pat in "${patterns[@]}"; do
    find "${APPDIR}" -type f -name "${pat}" -delete 2>/dev/null || true
    find "${APPDIR}" -type l -name "${pat}" -delete 2>/dev/null || true
  done
  rm -rf \
    "${APPDIR}/usr/lib/dri" \
    "${APPDIR}/usr/share/vulkan" \
    "${APPDIR}/usr/lib/vulkan" \
    2>/dev/null || true
  find "${APPDIR}" -type d -name dri -exec rm -rf {} + 2>/dev/null || true
}

patch_apprun_hooks() {
  local hook="${APPDIR}/apprun-hooks/linuxdeploy-plugin-gtk.sh"
  if [[ ! -f ${hook} ]]; then
    return
  fi
  # Force-X11 crashes native Wayland; GTK_THEME hides libadwaita's stylesheet.
  sed -i '/^export GDK_BACKEND=x11/d' "${hook}"
  sed -i '/^export GTK_THEME=/d' "${hook}"
}

# GtkMediaFile loads plugins at runtime (not ELF NEEDED). Copy codec plugins
# from the Ubuntu packages we install, then deploy their non-glibc deps.
# Do not copy all of /usr/lib/*/gstreamer-1.0: gst-plugins-bad pulls
# tensorflow, SRT, VA-API, and glibc extras. linuxdeploy's bundled strip also
# cannot handle Ubuntu 26.04 glibc RELR (`.relr.dyn`) if libresolv sneaks in.
bundle_gstreamer() {
  local multiarch src dst lib base
  if command -v dpkg-architecture >/dev/null 2>&1; then
    multiarch=$(dpkg-architecture -qDEB_HOST_MULTIARCH)
  else
    case "${HOST_ARCH}" in
      x86_64) multiarch=x86_64-linux-gnu ;;
      aarch64) multiarch=aarch64-linux-gnu ;;
      *)
        echo "Cannot resolve multiarch for GStreamer plugins (${HOST_ARCH})" >&2
        exit 1
        ;;
    esac
  fi

  src="/usr/lib/${multiarch}/gstreamer-1.0"
  dst="${APPDIR}/usr/lib/${multiarch}/gstreamer-1.0"
  mkdir -p "${dst}"

  copy_gst_plugins_from_pkg() {
    local pkg=$1 file
    if ! dpkg -s "${pkg}" >/dev/null 2>&1; then
      echo "initiative-tracker: skipping missing ${pkg}" >&2
      return 1
    fi
    while IFS= read -r file; do
      case "${file}" in
        */gstreamer-1.0/*)
          if [[ -f ${file} || -L ${file} ]]; then
            cp -a "${file}" "${dst}/"
          fi
          ;;
      esac
    done < <(dpkg -L "${pkg}")
  }

  copy_gst_plugins_from_pkg gstreamer1.0-plugins-base || true
  copy_gst_plugins_from_pkg gstreamer1.0-plugins-good || true
  copy_gst_plugins_from_pkg gstreamer1.0-libav || true
  copy_gst_plugins_from_pkg gstreamer1.0-gl || true
  copy_gst_plugins_from_pkg gstreamer1.0-gtk4 || true

  if [[ ! -d ${src} ]]; then
    echo "GStreamer plugin dir missing: ${src}" >&2
    echo "Install gstreamer1.0-plugins-base, gstreamer1.0-plugins-good, gstreamer1.0-libav" >&2
    exit 1
  fi
  if ! find "${dst}" -name '*.so' | grep -q .; then
    echo "No GStreamer plugins copied into ${dst}" >&2
    exit 1
  fi

  skip_system_lib() {
    local name
    name=$(basename "$1")
    case "${name}" in
      ld-linux*|ld-*.so*|libc.so*|libm.so*|libpthread.so*|libdl.so*|librt.so*| \
      libresolv.so*|libnss_*|libnsl.so*|libutil.so*|libanl.so*|libcrypt.so*| \
      libthread_db.so*|libBrokenLocale.so*|libGL.so*|libGLdispatch.so*| \
      libGLX*.so*|libOpenGL.so*|libEGL.so*|libvulkan.so*|libva.so*|libva-*.so*| \
      libvdpau.so*|libnvidia-*|libcuda.so*|libdrm.so*|libdrm_*)
        return 0
        ;;
    esac
    return 1
  }

  local libs=()
  while IFS= read -r lib; do
    [[ -n ${lib} && -f ${lib} ]] || continue
    if skip_system_lib "${lib}"; then
      continue
    fi
    libs+=(--library "${lib}")
  done < <(
    find "${dst}" -name '*.so' -print0 |
      xargs -0 ldd 2>/dev/null |
      awk '/=> \// { print $3 }' |
      sort -u
  )
  if ((${#libs[@]} > 0)); then
    (
      cd "${SCRIPT_DIR}"
      # Host glibc uses RELR; linuxdeploy's strip cannot process those objects.
      env APPIMAGE_EXTRACT_AND_RUN=1 NO_STRIP=1 \
        "${LINUXDEPLOY}" --appdir "${APPDIR}" \
        --exclude-library='libc.so*' \
        --exclude-library='libm.so*' \
        --exclude-library='libpthread.so*' \
        --exclude-library='libdl.so*' \
        --exclude-library='librt.so*' \
        --exclude-library='libresolv.so*' \
        --exclude-library='ld-linux*.so*' \
        --exclude-library='libnss_*.so*' \
        --exclude-library='libvulkan.so*' \
        --exclude-library='libGLX_mesa.so*' \
        --exclude-library='libEGL_mesa.so*' \
        --exclude-library='libgallium*.so*' \
        --exclude-library='libnvidia-*.so*' \
        "${libs[@]}"
    )
  fi

  mkdir -p "${APPDIR}/apprun-hooks"
  cat > "${APPDIR}/apprun-hooks/gstreamer.sh" <<EOF
export GST_PLUGIN_SYSTEM_PATH="\${APPDIR}/usr/lib/${multiarch}/gstreamer-1.0"
export GST_PLUGIN_PATH="\${GST_PLUGIN_SYSTEM_PATH}"
EOF
}

# --- main -------------------------------------------------------------------

VERSION=$(appimage_version)
OUTPUT_NAME="InitiativeTracker-${VERSION}-${HOST_ARCH}.AppImage"
OUTPUT_PATH="${SCRIPT_DIR}/${OUTPUT_NAME}"

echo "Building ${OUTPUT_NAME} from ${REPO_ROOT}"

rm -rf "${APPDIR}" "${BUILDDIR}"
rm -f "${SCRIPT_DIR}/InitiativeTracker-"*-"${HOST_ARCH}.AppImage"
mkdir -p "${APPDIR}"

fetch_tooling

export CARGO_HOME="${BUILDDIR}/cargo-home"
export CARGO_TARGET_DIR="${BUILDDIR}/cargo-target"
cargo build --manifest-path "${REPO_ROOT}/Cargo.toml" --release --locked
install -Dm755 "${CARGO_TARGET_DIR}/release/initiative-tracker-gtk" \
  "${APPDIR}/usr/bin/initiative-tracker-gtk"
bash "${REPO_ROOT}/packaging/install-data.sh" /usr "${APPDIR}"

DESKTOP_FILE="${APPDIR}/usr/share/applications/im.apodaca.InitiativeTracker.desktop"
ICON_FILE="${APPDIR}/usr/share/icons/hicolor/512x512/apps/im.apodaca.InitiativeTracker.png"
if [[ ! -f ${DESKTOP_FILE} ]]; then
  echo "Missing desktop file after install: ${DESKTOP_FILE}" >&2
  exit 1
fi
if [[ ! -f ${ICON_FILE} ]]; then
  echo "Missing app icon after install: ${ICON_FILE}" >&2
  exit 1
fi

export APPIMAGE_EXTRACT_AND_RUN=1
export DEPLOY_GTK_VERSION=4

LINUXDEPLOY="${CACHE_DIR}/squashfs-root/usr/bin/linuxdeploy"
if [[ ! -x ${LINUXDEPLOY} ]]; then
  LINUXDEPLOY="${CACHE_DIR}/${LINUXDEPLOY_BIN}"
fi
plugin_dest="$(dirname "${LINUXDEPLOY}")/linuxdeploy-plugin-gtk.sh"
if [[ ${plugin_dest} != "${CACHE_DIR}/linuxdeploy-plugin-gtk.sh" ]]; then
  cp -f "${CACHE_DIR}/linuxdeploy-plugin-gtk.sh" "${plugin_dest}"
fi
chmod +x "${plugin_dest}"

(
  cd "${SCRIPT_DIR}"
  env APPIMAGE_EXTRACT_AND_RUN=1 \
    DEPLOY_GTK_VERSION=4 \
    "${LINUXDEPLOY}" \
    --appdir "${APPDIR}" \
    --executable "${APPDIR}/usr/bin/initiative-tracker-gtk" \
    --desktop-file "${DESKTOP_FILE}" \
    --icon-file "${ICON_FILE}" \
    --plugin gtk \
    --exclude-library='libvulkan.so*' \
    --exclude-library='libGLX_mesa.so*' \
    --exclude-library='libEGL_mesa.so*' \
    --exclude-library='libgallium*.so*' \
    --exclude-library='libnvidia-*.so*'
)

bundle_gstreamer
patch_apprun_hooks
strip_graphics_driver_libs

APPIMAGETOOL="${CACHE_DIR}/squashfs-root/plugins/linuxdeploy-plugin-appimage/usr/bin/appimagetool"
if [[ ! -x ${APPIMAGETOOL} ]]; then
  echo "appimagetool missing after linuxdeploy extract: ${APPIMAGETOOL}" >&2
  exit 1
fi
(
  cd "${SCRIPT_DIR}"
  env ARCH="${HOST_ARCH}" \
    VERSION="${VERSION}" \
    APPIMAGE_EXTRACT_AND_RUN=1 \
    "${APPIMAGETOOL}" "${APPDIR}" "${OUTPUT_PATH}"
)

if [[ ! -f ${OUTPUT_PATH} ]]; then
  echo "AppImage not produced: expected ${OUTPUT_PATH}" >&2
  ls -la "${SCRIPT_DIR}" >&2 || true
  exit 1
fi
chmod +x "${OUTPUT_PATH}"

echo "AppImage written to ${OUTPUT_PATH}"
ls -lh "${OUTPUT_PATH}"
