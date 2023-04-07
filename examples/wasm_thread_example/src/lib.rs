use std::sync::{atomic::AtomicUsize, mpsc::Sender};

use wasm_bindgen::prelude::*;
use wasm_thread as thread;

#[wasm_bindgen]
pub struct Encoder {
    counter_thread: Option<thread::JoinHandle<usize>>,
    queue: Option<Sender<()>>,
}

static COUNT: AtomicUsize = AtomicUsize::new(0);

#[wasm_bindgen]
impl Encoder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");

        let (queue, receiver) = std::sync::mpsc::channel();

        wasm_thread::Builder::default()
            .prefix(String::from("gif"))
            .set_default();

        log::info!("Creating counter thread");

        let counter_thread = thread::Builder::new()
            .name(String::from("counter"))
            .spawn(move || {
                log::info!("Counter thread started");
                while let Ok(_) = receiver.recv() {
                    let _old_count = COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }

                log::info!("Counter thread stopped");
                COUNT.load(std::sync::atomic::Ordering::Relaxed)
            })
            .unwrap();

        log::info!("Created counter thread");

        Self {
            counter_thread: Some(counter_thread),
            queue: Some(queue),
        }
    }

    pub fn count(&self) -> usize {
        COUNT.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn increment(&mut self) {
        self.queue
            .as_ref()
            .expect("Encoder was closed")
            .send(())
            .unwrap();
    }

    pub async fn stop(&mut self) {
        self.queue = None;

        match self.counter_thread.take().unwrap().join_async().await {
            Ok(count) => {
                log::info!("Count: {}", count);
            }
            Err(e) => {
                log::error!("Error: {:?}", e);
            }
        }
    }
}
