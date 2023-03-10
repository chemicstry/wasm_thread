use std::time::Duration;

use wasm_bindgen::prelude::*;
use wasm_thread as thread;

#[wasm_bindgen(start)]
fn main() {
    console_log::init().unwrap();
    console_error_panic_hook::set_once();

    for _ in 0..2 {
        thread::spawn(|| {
            for i in 1..3 {
                log::info!("hi number {} from the spawned thread {:?}!", i, thread::current().id());
                thread::sleep(Duration::from_millis(1));
            }
        });
    }

    for i in 1..3 {
        log::info!("hi number {} from the main thread {:?}!", i, thread::current().id());
    }
}
