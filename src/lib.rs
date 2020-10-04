#[cfg(target_arch = "wasm32")]
mod wasm_thread;

#[cfg(not(target_arch = "wasm32"))]
pub use std::thread::*;
#[cfg(target_arch = "wasm32")]
pub use wasm_thread::*;
