#[cfg(not(target_arch = "wasm32"))]
use std::thread;
use std::time::Duration;

#[cfg(target_arch = "wasm32")]
use wasm_thread as thread;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;

    use crate::main;

    // Prevent `wasm_bindgen` from autostarting main on all spawned threads
    #[wasm_bindgen(start)]
    pub fn dummy_main() {}

    // Export explicit run function to start main
    #[wasm_bindgen]
    pub fn run() {
        console_log::init().unwrap();
        console_error_panic_hook::set_once();
        main();
    }
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init_from_env(env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"));

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

    // It's not possible to do a scope on the main thread, because blocking waits are not supported, but we can use
    // scope inside web workers.
    thread::spawn(|| {
        log::info!("Start scope test on thread {:?}", thread::current().id());

        let mut a = vec![1, 2, 3];
        let mut x = 0;

        thread::scope(|s| {
            s.spawn(|| {
                log::info!("hello from the first scoped thread {:?}", thread::current().id());
                // We can borrow `a` here.
                log::debug!("a = {:?}", &a);
            });
            s.spawn(|| {
                log::info!("hello from the second scoped thread {:?}", thread::current().id());
                // We can even mutably borrow `x` here,
                // because no other threads are using it.
                x += a[0] + a[2];
            });

            log::info!(
                "Hello from scope \"main\" thread {:?} inside scope.",
                thread::current().id()
            );
        });

        // After the scope, we can modify and access our variables again:
        a.push(4);
        assert_eq!(x, a.len());
        log::info!("Scope done x={} a.len()={}", x, a.len());
    });
}
