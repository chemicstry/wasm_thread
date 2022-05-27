#!/bin/sh

RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals" \
    wasm-pack build --dev --out-dir ./module/target --target web --features wasm_thread/es_modules
