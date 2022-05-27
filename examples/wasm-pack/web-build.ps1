$env:RUSTFLAGS="--cfg=web_sys_unstable_apis -C embed-bitcode=yes -C target-feature=+simd128,+atomics,+bulk-memory,+mutable-globals"
wasm-pack build --dev --out-dir ./module/target --target web --features wasm_thread/es_modules
