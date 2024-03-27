#![cfg(target_arch = "wasm32")]

use core::{
    sync::atomic::{AtomicBool, Ordering},
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
async fn thread_async_channel() {
    // Exchange a series of messages over async channel.
    let (thread_tx, main_rx) = async_channel::unbounded::<String>();
    let (main_tx, thread_rx) = async_channel::unbounded::<String>();

    thread::spawn(|| {
        futures::executor::block_on(async move {
            thread::sleep(Duration::from_millis(100));
            thread_tx.send("Hello".to_string()).await.unwrap();
            let mut msg = thread_rx.recv().await.unwrap();
            msg.push_str("!");
            thread_tx.send(msg).await.unwrap();
        })
    });

    let mut msg = main_rx.recv().await.unwrap();
    msg.push_str(" world");
    main_tx.send(msg).await.unwrap();

    let result = main_rx.recv().await.unwrap();
    assert_eq!(result, "Hello world!");
}

// This test should fail if "keep_worker_alive" enabled
#[wasm_bindgen_test]
async fn keep_worker_alive(){
    thread::spawn(|| {
        wasm_bindgen_futures::spawn_local(async move {
            let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::from(42));
            let x = wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
            //additional wait to simulate a js future that takes more time
            async_std::task::sleep(std::time::Duration::from_secs(1)).await;
            // This should only run if "keep_worker_alive" is enabled. If disabled, 
            // the thread will close before it can run
            assert_eq!(1, 2);
        });
    });
}

#[wasm_bindgen_test]
async fn spawn_async(){
    let (thread_tx, main_rx) = async_channel::unbounded::<String>();
    //since spawn_async closes the thread once the provided closure is complete,
    //"keep_worker_alive" is not necessary
    thread::spawn_async(|| async move{
        let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::from(42));
        let x = wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
        //additional wait to simulate a js future that takes more time
        async_std::task::sleep(std::time::Duration::from_secs(1)).await;
        thread_tx.send("After js future".to_string()).await.unwrap();
    });
    let mut msg = main_rx.recv().await.unwrap();

    assert_eq!(msg, "After js future");
}

