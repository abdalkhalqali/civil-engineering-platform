#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CACHE_DIR="$ROOT_DIR/.cache"
SDK_VERSION="3.47.4"
SDK_DIR="$CACHE_DIR/flutter_${SDK_VERSION}"
ARCHIVE="$CACHE_DIR/flutter_linux_${SDK_VERSION}-stable.tar.xz"
SDK_URL="https://storage.googleapis.com/flutter_infra_release/releases/stable/linux/flutter_linux_${SDK_VERSION}-stable.tar.xz"
EXPECTED_SHA256="5b45f0ceda99b9bebdc873e7e69f6450aeb4c30f454b505e2e62fc9255a907d3"

mkdir -p "$CACHE_DIR"

if [[ ! -x "$SDK_DIR/bin/flutter" ]]; then
  echo "Preparing Flutter $SDK_VERSION..."
  rm -rf "$CACHE_DIR/flutter_extract"
  mkdir -p "$CACHE_DIR/flutter_extract"

  if [[ ! -f "$ARCHIVE" ]]; then
    curl --fail --location --retry 3 --output "$ARCHIVE" "$SDK_URL"
  fi

  ACTUAL_SHA256="$(sha256sum "$ARCHIVE" | awk '{print $1}')"
  if [[ "$ACTUAL_SHA256" != "$EXPECTED_SHA256" ]]; then
    echo "Flutter SDK checksum mismatch." >&2
    echo "Expected: $EXPECTED_SHA256" >&2
    echo "Actual:   $ACTUAL_SHA256" >&2
    exit 1
  fi

  tar -xJf "$ARCHIVE" -C "$CACHE_DIR/flutter_extract"
  rm -rf "$SDK_DIR"
  mv "$CACHE_DIR/flutter_extract/flutter" "$SDK_DIR"
  rm -rf "$CACHE_DIR/flutter_extract" "$ARCHIVE"
fi

export PATH="$SDK_DIR/bin:$PATH"
export FLUTTER_ROOT="$SDK_DIR"
export PUB_CACHE="${PUB_CACHE:-$CACHE_DIR/dart_pub_cache}"

cd "$ROOT_DIR/apps/client_flutter"
flutter pub get

flutter build web --release

exec python3 -m http.server \
  "${PORT:-5000}" \
  --bind 0.0.0.0 \
  --directory "$ROOT_DIR/apps/client_flutter/build/web"