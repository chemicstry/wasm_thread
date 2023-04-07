/// From https://github.com/rustwasm/wasm-bindgen/tree/main/examples/raytrace-parallel

// synchronously, using the browser, import out shim JS scripts
import init, { Encoder } from "./wasm_thread_example.js";

// Wait for the main thread to send us the shared module/memory. Once we've got
// it, initialize it all with the `wasm_bindgen` global we imported via
// `importScripts`.
//
// After our first message all subsequent messages are an entry point to run,
// so we just do that.
let encoder;
let lastCount;
let diffCount = 0;
self.onmessage = async (event) => {
  switch (event.data.type) {
    case "init": {
      await init();
      encoder = new Encoder();
      self.postMessage({ type: "initResponse" });
      break;
    }
    case "encode": {
      encoder.increment();
      const count = encoder.count();
      if (count !== lastCount) {
        lastCount = count;
        console.log(count);
        diffCount++;
      }
      break;
    }
    case "finish": {
      const result = await encoder.stop();
      console.log("diffCount", diffCount);
      self.postMessage({ type: "finishResponse", result });
      break;
    }
  }
};
