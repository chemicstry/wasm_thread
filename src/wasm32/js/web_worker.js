// synchronously, using the browser, import wasm_bindgen shim JS scripts
importScripts('WASM_BINDGEN_SHIM_URL');

// Wait for the main thread to send us the shared module/memory and work context.
// Once we've got it, initialize it all with the `wasm_bindgen` global we imported via
// `importScripts`.
self.onmessage = event => {
    let [ module, memory, work, thread_key ] = event.data;

    wasm_bindgen(module, memory).catch(err => {
        console.log(err);

        // Propagate to main `onerror`:
        setTimeout(() => {
            throw err;
        });
        // Rethrow to keep promise rejected and prevent execution of further commands:
        throw err;
    }).then(wasm => {
        // Enter rust code by calling entry point defined in `lib.rs`.
        // This executes closure defined by work context.
        wasm.wasm_thread_entry_point(work);
    });
};
  