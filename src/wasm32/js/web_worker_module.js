// synchronously, using the browser, import wasm_bindgen shim JS scripts
import init, { wasm_thread_entry_point, keep_worker_alive } from "WASM_BINDGEN_SHIM_URL";

// Wait for the main thread to send us the shared module/memory and work context.
// Once we've got it, initialize it all with the `wasm_bindgen` global we imported via
// `importScripts`.
self.onmessage = event => {
    let [module, memory, work] = event.data;

    init(module, memory).catch(err => {
        console.log(err);

        // Propagate to main `onerror`:
        setTimeout(() => {
            throw err;
        });
        // Rethrow to keep promise rejected and prevent execution of further commands:
        throw err;
    }).then(() => {
        // Enter rust code by calling entry point defined in `lib.rs`.
        // This executes closure defined by work context.
        wasm_thread_entry_point(work);

        if (!keep_worker_alive()) {
            // Once done, terminate web worker
            close();
        }
    });
};