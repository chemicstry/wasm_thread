# wasm_thread

An `std::thread` replacement for wasm32 target.

This crate tries to closely replicate `std::thread` API, however, some features are not yet implemented or are even impossible to implement.

## Running examples

### Native

- Just `cargo run --example simple`

### wasm32

- Build with `./build_wasm.sh` or copy paste commands from the script if your environment does not support shell scripts. This custom build step is required because prebuilt standard library does not have support for atomics yet. Read more about this [here](https://rustwasm.github.io/2018/10/24/multithreading-rust-and-wasm.html).
- Serve `examples` directory over HTTP and open `simple.html` in browser. Inspect console output. You can use `cargo install basic-http-server` and `basic-http-server examples`.

## Using as a library

- Add `wasm_thread` to your `Cargo.toml`.
- Replace `use std::thread` with `use wasm_thread as thread`. Note that some API might be missing.
- Adapt [build_wasm.sh](build_wasm.sh) to your project. Currently only `--no-modules` target is supported.

## Notes on wasm limitations

- Any blocking API (`thread.join()`, `futures::block_on()`, etc) on the main thread will freeze the browser for as long as lock is maintained. This also freezes any proxied functions, which means that worker spawning, network fetches and other similar asynchronous APIs will block also and can cause a deadlock. To avoid this, either run your `main()` in a worker thread or use async futures.
- Atomic locks (`i32.atomic.wait` to be specific) will panic on the main thread. This means that `mutex.lock()` will likely crash. Solution is the same as above.
- Only `no-modules` target is supported by `wasm_bindgen`. This will be lifted once browsers support modules in web workers.
- Web workers are normally spawned by providing a script URL, however, to avoid bundling scripts this library uses URL encoded blob [web_worker.js](src/web_worker.js) to avoid HTTP fetch. `wasm_bindgen` generated `.js` shim script is still needed and a [hack](src/script_path.js) is used to obtain its URL. If this for some reason does not work in your setup, please report an issue or use `Builder::wasm_bindgen_shim_url()` to specify explicit URL.
- For additional information on wasm threading look at [this](https://rustwasm.github.io/2018/10/24/multithreading-rust-and-wasm.html) blogpost or [raytrace-parallel](https://rustwasm.github.io/wasm-bindgen/examples/raytrace.html) example.

### Example output

Native:
```
hi number 1 from the spawned thread ThreadId(2)!
hi number 1 from the main thread ThreadId(1)!
hi number 1 from the spawned thread ThreadId(3)!
hi number 2 from the main thread ThreadId(1)!
hi number 2 from the spawned thread ThreadId(2)!
hi number 2 from the spawned thread ThreadId(3)!
```

Wasm:
```
hi number 1 from the main thread ThreadId(1)!
hi number 2 from the main thread ThreadId(1)!
hi number 1 from the spawned thread ThreadId(2)!
hi number 1 from the spawned thread ThreadId(3)!
hi number 2 from the spawned thread ThreadId(2)!
hi number 2 from the spawned thread ThreadId(3)!
```

As you can see wasm threads are only spawned after `main()` returns, because browser event loop cannot continue while main thread is blocked.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

#### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
