#!/bin/sh

case $1 in
  *"-r"*) # Matches -r or --release
    CONF="--release -Z build-std-features=panic_immediate_abort"
    BINDGEN=""
    FILE_PATH="release"
    ;;
  *)
    CONF=""
    BINDGEN="--keep-debug"
    FILE_PATH="debug"
    ;;
esac

RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals' \
  cargo +nightly build \
  --target wasm32-unknown-unknown \
  -Z build-std=std,panic_abort \
  ${CONF}

wasm-bindgen \
  target/wasm32-unknown-unknown/${FILE_PATH}/wasm_thread_example.wasm \
  ${BINDGEN} \
  --target web \
  --out-dir ./pkg
