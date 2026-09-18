#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CACHE_DIR="$ROOT_DIR/.cache"
SDK_VERSION="3.47.4"
SDK_DIR="$CACHE_DIR/flutter_${SDK_VERSION}"
ARCHIVE="$CACHE_DIR/flutter_linux_${SDK_VERSION}-stable.tar.xz"
SDK_URL="https://storage.googleapis.com/flutter_infra_release/releases/stable/linux/flutter_linux_${SDK_VERSION}-stable.tar.xz"
EXPECTED_SHA256="5b45f0ceda99b9bebdc873e7e69f6450aeb4c30f454b505e2e62fc9255a907d3"
WASM_BINDGEN_VERSION="0.2.128"
WASM_BINDGEN_DIR="$CACHE_DIR/wasm-bindgen-cli-$WASM_BINDGEN_VERSION"
WASM_OUTPUT="$CACHE_DIR/frb_wasm/pkg"
WASM_RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals -C link-args=--shared-memory -C link-args=--max-memory=1073741824 -C link-args=--import-memory -C link-args=--export=__heap_base -C link-args=--export=__wasm_init_tls -C link-args=--export=__tls_size -C link-args=--export=__tls_align -C link-args=--export=__tls_base"

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

if [[ ! -x "$WASM_BINDGEN_DIR/bin/wasm-bindgen" ]]; then
  cargo install wasm-bindgen-cli \
    --version "$WASM_BINDGEN_VERSION" \
    --locked \
    --root "$WASM_BINDGEN_DIR"
fi

SYSTEM_RUSTC="$(command -v rustc)"
SYSTEM_RUSTDOC="$(command -v rustdoc)"
unset RUSTUP_TOOLCHAIN
export RUSTC="$SYSTEM_RUSTC"
export RUSTDOC="$SYSTEM_RUSTDOC"
export RUSTC_BOOTSTRAP=1
export RUSTFLAGS="$WASM_RUSTFLAGS"

cd "$ROOT_DIR/apps/client_flutter"
flutter pub get

rm -rf "$WASM_OUTPUT"
mkdir -p "$WASM_OUTPUT"
cargo build \
  --manifest-path "$ROOT_DIR/core/geometry_kernel_rs/Cargo.toml" \
  --lib \
  --release \
  --target wasm32-unknown-unknown \
  -Z build-std=std,panic_abort
"$WASM_BINDGEN_DIR/bin/wasm-bindgen" \
  "$ROOT_DIR/core/geometry_kernel_rs/target/wasm32-unknown-unknown/release/geometry_kernel_rs.wasm" \
  --out-dir "$WASM_OUTPUT" \
  --no-typescript \
  --target no-modules \
  --out-name geometry_kernel_rs

flutter build web --release

mkdir -p build/web/pkg
cp "$WASM_OUTPUT/geometry_kernel_rs.js" build/web/pkg/
cp "$WASM_OUTPUT/geometry_kernel_rs_bg.wasm" build/web/pkg/

exec python3 "$ROOT_DIR/scripts/serve_flutter_web.py" \
  --port "${PORT:-5000}" \
  --directory "$ROOT_DIR/apps/client_flutter/build/web"