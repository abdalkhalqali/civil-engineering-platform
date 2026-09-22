#!/usr/bin/env bash
# Compiles the Rust engineering kernel to WebAssembly and generates the JavaScript
# glue the browser workbench loads.
#
# The kernel is the single source of truth; this script only changes the shape of
# its boundary (native for Flutter, WebAssembly for the browser).
#
# Requirements: Rust with the `wasm32-unknown-unknown` target, and a wasm-bindgen
# CLI whose version matches the `wasm-bindgen` crate in Cargo.lock.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CRATE_DIR="$ROOT_DIR/core/geometry_kernel_rs"
OUT_DIR="$ROOT_DIR/web/pkg"
WASM_BINDGEN_VERSION="0.2.128"
TARGET_DIR="${CARGO_TARGET_DIR:-$CRATE_DIR/target}"
WASM_BINDGEN_BIN="${WASM_BINDGEN_BIN:-wasm-bindgen}"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required to build the kernel." >&2
  exit 1
fi

if ! command -v "$WASM_BINDGEN_BIN" >/dev/null 2>&1; then
  echo "wasm-bindgen $WASM_BINDGEN_VERSION is required. Install it with:" >&2
  echo "  cargo install wasm-bindgen-cli --version $WASM_BINDGEN_VERSION --locked" >&2
  exit 1
fi

export CARGO_TARGET_DIR="$TARGET_DIR"

rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true

cargo build \
  --manifest-path "$CRATE_DIR/Cargo.toml" \
  --lib \
  --release \
  --no-default-features \
  --features wasm \
  --target wasm32-unknown-unknown

WASM_FILE="$TARGET_DIR/wasm32-unknown-unknown/release/geometry_kernel_rs.wasm"

rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

"$WASM_BINDGEN_BIN" "$WASM_FILE" \
  --target web \
  --no-typescript \
  --out-dir "$OUT_DIR" \
  --out-name geometry_kernel

echo "Kernel built to $OUT_DIR"
