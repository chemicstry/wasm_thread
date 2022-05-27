$env:RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals"
wasm-pack build --dev --out-dir ./no-module/target --target no-modules
