use std::sync::MutexGuard;

use wasm_bindgen::prelude::*;
use web_sys::{Blob, Url};

/// Extracts path of the `wasm_bindgen` generated .js shim script
pub fn get_wasm_bindgen_shim_script_path() -> String {
    js_sys::eval(include_str!("js/script_path.js"))
        .unwrap()
        .as_string()
        .unwrap()
}

/// Generates worker entry script as URL encoded blob
pub fn get_worker_script(wasm_bindgen_shim_url: Option<String>) -> String {
    static mut SCRIPT_URL: Option<String> = None;

    if let Some(url) = unsafe { SCRIPT_URL.as_ref() } {
        url.clone()
    } else {
        // If wasm bindgen shim url is not provided, try to obtain one automatically
        let wasm_bindgen_shim_url =
            wasm_bindgen_shim_url.unwrap_or_else(get_wasm_bindgen_shim_script_path);

        // Generate script from template
        let template;
        #[cfg(feature = "es_modules")]
        {
            template = include_str!("js/web_worker_module.js");
        }
        #[cfg(not(feature = "es_modules"))]
        {
            template = include_str!("js/web_worker.js");
        }
        let script = template.replace("WASM_BINDGEN_SHIM_URL", &wasm_bindgen_shim_url);

        // Create url encoded blob
        let arr = js_sys::Array::new();
        arr.set(0, JsValue::from_str(&script));
        let blob = Blob::new_with_str_sequence(&arr).unwrap();
        let url = Url::create_object_url_with_blob(
            &blob
                .slice_with_f64_and_f64_and_content_type(0.0, blob.size(), "text/javascript")
                .unwrap(),
        )
        .unwrap();
        unsafe { SCRIPT_URL = Some(url.clone()) };

        url
    }
}
