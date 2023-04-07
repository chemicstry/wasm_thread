#![cfg(target_arch = "wasm32")]

use core::{
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::Duration,
};

use wasm_bindgen_test::*;
use wasm_thread as thread;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn thread_join_async() {
    let handle = thread::spawn(|| 1234);

    assert_eq!(handle.join_async().await.unwrap(), 1234);
}

#[wasm_bindgen_test]
async fn thread_join_sync() {
    // synchronous join only allowed inside threads
    thread::spawn(|| {
        let handle = thread::spawn(|| 1234);

        assert_eq!(handle.join().unwrap(), 1234);
    })
    .join_async()
    .await
    .unwrap();
}

#[wasm_bindgen_test]
async fn thread_scope_sync() {
    // synchronous scope only allowed inside threads
    thread::spawn(|| {
        let mut a = vec![1, 2, 3];
        let mut x = 0;

        thread::scope(|s| {
            s.spawn(|| {
                println!("hello from the first scoped thread {:?}", thread::current().id());
                // We can borrow `a` here.
                dbg!(&a)
            });

            s.spawn(|| {
                println!("hello from the second scoped thread {:?}", thread::current().id());
                // We can even mutably borrow `x` here,
                // because no other threads are using it.
                x += a[0] + a[2];
            });

            println!(
                "Hello from scope \"main\" thread {:?} inside scope.",
                thread::current().id()
            );
        });

        // After the scope, we can modify and access our variables again:
        a.push(4);
        assert_eq!(x, a.len());
    })
    .join_async()
    .await
    .unwrap();
}

#[wasm_bindgen_test]
async fn thread_scope_sync_block() {
    // synchronous scope only allowed inside threads
    thread::spawn(|| {
        let t1_done = AtomicBool::new(false);
        let t2_done = AtomicBool::new(false);

        thread::scope(|s| {
            s.spawn(|| {
                thread::sleep(Duration::from_millis(100));
                t1_done.store(true, Ordering::Relaxed);
            });

            s.spawn(|| {
                thread::sleep(Duration::from_millis(100));
                t2_done.store(true, Ordering::Relaxed);
            });

            // Threads should be in sleep and not yet done
            assert_eq!(t1_done.load(Ordering::Relaxed), false);
            assert_eq!(t2_done.load(Ordering::Relaxed), false);
        });

        // Scope should block until both threads terminate
        assert_eq!(t1_done.load(Ordering::Relaxed), true);
        assert_eq!(t2_done.load(Ordering::Relaxed), true);
    })
    .join_async()
    .await
    .unwrap();
}

#[wasm_bindgen_test]
async fn thread_messaging() {
    use std::{
        sync::mpsc::{channel, Receiver},
        thread as std_thread,
    };

    let (tx, rx) = channel();
    static ATOMIC_COUNT: AtomicUsize = AtomicUsize::new(0);

    fn reader_callback(rx: Receiver<String>) {
        while let Ok(_) = rx.recv() {
            let old_value = ATOMIC_COUNT.fetch_add(1, Ordering::Relaxed);
            if old_value == usize::MAX {
                break;
            }

            std_thread::sleep(Duration::from_millis(200));
        }
    }

    let reader_thread = thread::Builder::new()
        .name(String::from("reader"))
        .spawn(|| reader_callback(rx))
        .unwrap();

    for i in 0..1000 {
        tx.send(format!("message {}", i)).unwrap();
    }

    let _ = thread::spawn(move || {
        std_thread::sleep(Duration::from_millis(1100));

        let value = ATOMIC_COUNT.load(Ordering::Relaxed);
        std::assert_eq!(value, 6);
        ATOMIC_COUNT.store(usize::MAX, Ordering::Relaxed);
    })
    .join_async()
    .await
    .unwrap();

    reader_thread.join_async().await.unwrap();
}

#[wasm_bindgen_test]
async fn thread_no_join() {
    use std::sync::mpsc::channel;

    let (tx, rx) = channel();
    static ATOMIC_STARTED: AtomicBool = AtomicBool::new(false);

    let _ = thread::Builder::new()
        .name(String::from("polled"))
        .spawn(move || {
            ATOMIC_STARTED.store(true, Ordering::Relaxed);
            rx.recv().unwrap();
        })
        .unwrap();

    let _ = thread::Builder::new()
        .name(String::from("awaiter"))
        .spawn(move || {
            thread::sleep(Duration::from_millis(1000));
            let started = ATOMIC_STARTED.load(Ordering::Relaxed);
            std::assert_eq!(started, true);
        })
        .unwrap()
        .join_async()
        .await
        .unwrap();

    tx.send(()).unwrap();
}
