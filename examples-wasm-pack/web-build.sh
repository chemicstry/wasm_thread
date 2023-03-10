#!/bin/sh

wasm-pack build --dev --out-dir ./module/target --target web --features wasm_thread/es_modules
