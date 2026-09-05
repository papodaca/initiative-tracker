#!/usr/bin/env bash
# Install desktop entry, AppStream metainfo, and icons under PREFIX.
# Usage: install-data.sh PREFIX [DESTDIR]
set -euo pipefail

PREFIX=${1:?prefix required}
DESTDIR=${2:-}
APP_ID=im.apodaca.InitiativeTracker
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
SHARE="${DESTDIR}${PREFIX}/share"

install -Dm644 "${ROOT}/data/${APP_ID}.desktop" \
  "${SHARE}/applications/${APP_ID}.desktop"
install -Dm644 "${ROOT}/data/${APP_ID}.metainfo.xml" \
  "${SHARE}/metainfo/${APP_ID}.metainfo.xml"

for size in 32x32 128x128 512x512; do
  install -Dm644 \
    "${ROOT}/data/icons/hicolor/${size}/apps/${APP_ID}.png" \
    "${SHARE}/icons/hicolor/${size}/apps/${APP_ID}.png"
done

install -Dm644 \
  "${ROOT}/data/icons/hicolor/scalable/apps/${APP_ID}.svg" \
  "${SHARE}/icons/hicolor/scalable/apps/${APP_ID}.svg"
install -Dm644 \
  "${ROOT}/data/icons/hicolor/symbolic/apps/${APP_ID}-symbolic.svg" \
  "${SHARE}/icons/hicolor/symbolic/apps/${APP_ID}-symbolic.svg"
