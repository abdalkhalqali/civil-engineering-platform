---
name: FRB WebAssembly threading
description: Constraints for running flutter_rust_bridge worker-based WASM in the Flutter web preview.
---

FRB's web worker pool requires a threaded WASM build, a wasm-bindgen CLI compatible with the wasm-bindgen crate, and a cross-origin-isolated static server.

**Why:** A normal wasm-pack build produces a module whose `WebAssembly.Memory` cannot be cloned into FRB workers, while mismatched wasm-bindgen versions fail during JS binding generation. Missing COOP/COEP headers causes the same worker startup path to fail in the browser.

**How to apply:** Keep the build's atomics/shared-memory flags and `build-std` step together, pin the wasm-bindgen CLI used by the build to the crate-compatible version, and serve the resulting `pkg` files with COOP/COEP headers.