use std::{
    marker::PhantomData,
    panic::{catch_unwind, resume_unwind, AssertUnwindSafe},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    thread::{current, sleep, Thread},
    time::Duration,
};

use crate::Builder;

pub struct ScopeData {
    num_running_threads: AtomicUsize,
    a_thread_panicked: AtomicBool,
    main_thread: Thread,
}

pub struct Scope<'scope, 'env: 'scope> {
    data: Arc<ScopeData>,
    /// Invariance over 'scope, to make sure 'scope cannot shrink,
    /// which is necessary for soundness.
    ///
    /// Without invariance, this would compile fine but be unsound:
    ///
    /// ```compile_fail,E0373
    /// std::thread::scope(|s| {
    ///     s.spawn(|| {
    ///         let a = String::from("abcd");
    ///         s.spawn(|| println!("{a:?}")); // might run after `a` is dropped
    ///     });
    /// });
    /// ```
    scope: PhantomData<&'scope mut &'scope ()>,
    env: PhantomData<&'env mut &'env ()>,
}

impl Builder {
    /// Spawns a new scoped thread using the settings set through this `Builder`.
    ///
    /// Unlike [Scope::spawn], this method yields an [`io::Result`] to
    /// capture any failure to create the thread at the OS level.
    pub fn spawn_scoped<'scope, 'env, F, T>(
        self,
        _scope: &'scope Scope<'scope, 'env>,
        f: F,
    ) -> std::io::Result<ScopedJoinHandle<'scope, T>>
    where
        F: FnOnce() -> T + Send + 'scope,
        T: Send + 'scope,
    {
        Ok(ScopedJoinHandle(
            unsafe { self.spawn_unchecked(f) }?,
            PhantomData,
        ))
    }
}

pub fn scope<'env, F, T>(f: F) -> T
where
    F: for<'scope> FnOnce(&'scope Scope<'scope, 'env>) -> T,
{
    // We put the `ScopeData` into an `Arc` so that other threads can finish their
    // `decrement_num_running_threads` even after this function returns.
    let scope = Scope {
        data: Arc::new(ScopeData {
            num_running_threads: AtomicUsize::new(0),
            main_thread: current(),
            a_thread_panicked: AtomicBool::new(false),
        }),
        env: PhantomData,
        scope: PhantomData,
    };

    // Run `f`, but catch panics so we can make sure to wait for all the threads to join.
    let result = catch_unwind(AssertUnwindSafe(|| f(&scope)));

    // Wait until all the threads are finished.
    while scope.data.num_running_threads.load(Ordering::Acquire) != 0 {
        // park();
        // TODO: Replaced by a wasm-friendly version of park()
        sleep(Duration::from_millis(1));
    }

    // Throw any panic from `f`, or the return value of `f` if no thread panicked.
    match result {
        Err(e) => resume_unwind(e),
        Ok(_) if scope.data.a_thread_panicked.load(Ordering::Relaxed) => {
            panic!("a scoped thread panicked")
        }
        Ok(result) => result,
    }
}

pub struct ScopedJoinHandle<'scope, T>(
    pub(crate) crate::JoinHandle<T>,
    pub(crate) PhantomData<&'scope ()>,
);

impl<'scope, T> ScopedJoinHandle<'scope, T> {
    pub fn join(self) -> std::io::Result<T> {
        self.0
            .join()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, ""))
    }
}

pub fn spawn_scoped<'scope, 'env, F, T>(
    builder: crate::Builder,
    scope: &'scope Scope<'scope, 'env>,
    f: F,
) -> std::io::Result<ScopedJoinHandle<'scope, T>>
where
    F: FnOnce() -> T + Send + 'scope,
    T: Send + 'scope,
{
    Ok(ScopedJoinHandle(
        unsafe { builder.spawn_unchecked(f) }?,
        PhantomData,
    ))
}
