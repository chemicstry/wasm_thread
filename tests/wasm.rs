#![cfg(target_arch = "wasm32")]

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
