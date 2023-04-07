//! Trivial tests to ensure that native thread API is unchanged.

use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use wasm_thread as thread;

#[test]
fn thread_join() {
    let handle = thread::spawn(|| 1234);

    assert_eq!(handle.join().unwrap(), 1234);
}

#[test]
fn thread_scope() {
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
}

#[test]
fn thread_messaging() {
    use std::sync::mpsc::{channel, Receiver};

    let (tx, rx) = channel();
    static ATOMIC_COUNT: AtomicUsize = AtomicUsize::new(0);

    fn reader_callback(rx: Receiver<String>) {
        while let Ok(_) = rx.recv() {
            let old_value = ATOMIC_COUNT.fetch_add(1, Ordering::Relaxed);
            if old_value == usize::MAX {
                break;
            }
            thread::sleep(Duration::from_millis(200));
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
        drop(tx);
        thread::sleep(Duration::from_millis(1100));

        let value = ATOMIC_COUNT.load(Ordering::Relaxed);
        std::assert_eq!(value, 6);
        ATOMIC_COUNT.store(usize::MAX, Ordering::Relaxed);
    })
    .join()
    .unwrap();

    reader_thread.join().unwrap();
}

#[test]
fn thread_no_join() {
    use std::sync::{atomic::AtomicBool, mpsc::channel};

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
        .join()
        .unwrap();

    tx.send(()).unwrap();
}
