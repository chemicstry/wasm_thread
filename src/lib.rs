use async_channel::Receiver;
use futures::executor::block_on;
use std::any::Any;
use std::fmt;
use std::mem;

pub use std::thread::{current, sleep, Result, Thread, ThreadId};

use wasm_bindgen::prelude::*;
use wasm_bindgen::*;
use web_sys::{Blob, DedicatedWorkerGlobalScope, Url, Worker, WorkerOptions, WorkerType};

struct WebWorkerContext {
    func: Box<dyn FnOnce() + Send>,
}

#[cfg(feature = "es_modules")]
#[wasm_bindgen(module = "/src/module_workers_polyfill.min.js")]
extern "C" {
    fn load_module_workers_polyfill();
}

/// Extracts path of the `wasm_bindgen` generated .js shim script
pub fn get_wasm_bindgen_shim_script_path() -> String {
    js_sys::eval(include_str!("script_path.js"))
        .unwrap()
        .as_string()
        .unwrap()
}

/// Generates worker entry script as URL encoded blob
pub fn get_worker_script(wasm_bindgen_shim_url: Option<String>) -> String {
    unsafe {
        static mut SCRIPT_URL: Option<String> = None;

        if let Some(url) = SCRIPT_URL.as_ref() {
            url.clone()
        } else {
            // If wasm bindgen shim url is not provided, try to obtain one automatically
            let wasm_bindgen_shim_url =
                wasm_bindgen_shim_url.unwrap_or_else(get_wasm_bindgen_shim_script_path);

            // Generate script from template
            let template;
            #[cfg(feature = "es_modules")]
            {
                template = include_str!("web_worker_module.js");
            }
            #[cfg(not(feature = "es_modules"))]
            {
                template = include_str!("web_worker.js");
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
            SCRIPT_URL = Some(url.clone());

            url
        }
    }
}

/// Entry point for web workers
#[wasm_bindgen]
pub fn wasm_thread_entry_point(ptr: u32) {
    let ctx = unsafe { Box::from_raw(ptr as *mut WebWorkerContext) };
    (ctx.func)();
    WorkerMessage::ThreadComplete.post();
}

static mut MAIN_THREAD_ID: Option<ThreadId> = None;

struct BuilderRequest {
    builder: Builder,
    context: WebWorkerContext,
}

impl BuilderRequest {
    pub unsafe fn spawn(self) {
        self.builder.spawn_for_context(self.context);
    }
}

enum WorkerMessage {
    SpawnThread(BuilderRequest),
    ThreadComplete,
}

impl WorkerMessage {
    pub fn post(self) {
        let req = Box::new(self);
        unsafe {
            js_sys::eval("self")
                .unwrap()
                .dyn_into::<DedicatedWorkerGlobalScope>()
                .unwrap()
                .post_message(&JsValue::from(std::mem::transmute::<_, f64>(
                    Box::into_raw(req) as u64,
                )))
                .unwrap();
        }
    }
}

/// Thread factory, which can be used in order to configure the properties of a new thread.
#[derive(Debug, Default)]
pub struct Builder {
    // A name for the thread-to-be, for identification in panic messages
    name: Option<String>,
    // The size of the stack for the spawned thread in bytes
    stack_size: Option<usize>,
    // Url of the `wasm_bindgen` generated shim `.js` script to use as web worker entry point
    wasm_bindgen_shim_url: Option<String>,
}

impl Builder {
    /// Generates the base configuration for spawning a thread, from which
    /// configuration methods can be chained.
    pub fn new() -> Builder {
        Builder::default()
    }

    /// Names the thread-to-be.
    pub fn name(mut self, name: String) -> Builder {
        self.name = Some(name);
        self
    }

    /// Sets the size of the stack (in bytes) for the new thread.
    pub fn stack_size(mut self, size: usize) -> Builder {
        self.stack_size = Some(size);
        self
    }

    /// Sets the URL of wasm_bindgen generated shim script.
    pub fn wasm_bindgen_shim_url(mut self, url: String) -> Builder {
        self.wasm_bindgen_shim_url = Some(url);
        self
    }

    /// Spawns a new thread by taking ownership of the `Builder`, and returns an
    /// [`io::Result`] to its [`JoinHandle`].
    pub fn spawn<F, T>(self, f: F) -> std::io::Result<JoinHandle<T>>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        unsafe { self.spawn_unchecked(f) }
    }

    /// Spawns a new thread without any lifetime restrictions by taking ownership
    /// of the `Builder`, and returns an [`io::Result`] to its [`JoinHandle`].
    ///
    /// # Safety
    ///
    /// The caller has to ensure that no references in the supplied thread closure
    /// or its return type can outlive the spawned thread's lifetime. This can be
    /// guaranteed in two ways:
    ///
    /// - ensure that [`join`][`JoinHandle::join`] is called before any referenced
    /// data is dropped
    /// - use only types with `'static` lifetime bounds, i.e., those with no or only
    /// `'static` references (both [`Builder::spawn`]
    /// and [`spawn`] enforce this property statically)
    pub unsafe fn spawn_unchecked<'a, F, T>(self, f: F) -> std::io::Result<JoinHandle<T>>
    where
        F: FnOnce() -> T,
        F: Send + 'a,
        T: Send + 'a,
    {
        let (sender, receiver) = async_channel::bounded(1);

        let main = Box::new(move || {
            let res = f();
            sender.try_send(res).ok();
        });
        let context = WebWorkerContext {
            func: mem::transmute::<Box<dyn FnOnce() + Send + 'a>, Box<dyn FnOnce() + Send + 'static>>(
                main,
            ),
        };

        if MAIN_THREAD_ID.is_none() {
            MAIN_THREAD_ID = Some(current().id());
        }

        if MAIN_THREAD_ID.unwrap_unchecked() == current().id() {
            self.spawn_for_context(context);
        } else {
            WorkerMessage::SpawnThread(BuilderRequest {
                builder: self,
                context,
            })
            .post();
        }

        Ok(JoinHandle(JoinInner { receiver }))
    }

    unsafe fn spawn_for_context(self, ctx: WebWorkerContext) {
        let Builder {
            name,
            wasm_bindgen_shim_url,
            ..
        } = self;

        // Get worker script as URL encoded blob
        let script = get_worker_script(wasm_bindgen_shim_url);

        // Todo: figure out how to set stack size
        let mut options = WorkerOptions::new();
        if let Some(name) = name {
            options.name(&name);
        }
        #[cfg(feature = "es_modules")]
        {
            load_module_workers_polyfill();
            options.type_(WorkerType::Module);
        }
        #[cfg(not(feature = "es_modules"))]
        {
            options.type_(WorkerType::Classic);
        }

        // Spawn the worker
        let worker = Worker::new_with_options(script.as_str(), &options).unwrap();
        let worker_reference = std::rc::Rc::new(std::cell::Cell::new(None));
        let worker_reference_callback = worker_reference.clone();
        let callback = Closure::wrap(Box::new(move |x: &web_sys::MessageEvent| {
            let req = Box::from_raw(
                std::mem::transmute::<_, u64>(x.data().as_f64().unwrap()) as *mut WorkerMessage
            );
            match *req {
                WorkerMessage::SpawnThread(builder) => {
                    builder.spawn();
                }
                WorkerMessage::ThreadComplete => {
                    worker_reference.replace(None);
                }
            };
        }) as Box<dyn FnMut(&web_sys::MessageEvent)>);
        worker.set_onmessage(Some(callback.as_ref().unchecked_ref()));
        callback.forget();

        let ctx_ptr = Box::into_raw(Box::new(ctx));

        // Pack shared wasm (module and memory) and work as a single JS array
        let init = js_sys::Array::new();
        init.push(&wasm_bindgen::module());
        init.push(&wasm_bindgen::memory());
        init.push(&JsValue::from(ctx_ptr as u32));

        // Send initialization message
        worker_reference_callback.set(Some(
            match worker.post_message(&init) {
                Ok(()) => Ok(worker),
                Err(e) => {
                    drop(Box::from_raw(ctx_ptr));
                    Err(e)
                }
            }
            .unwrap(),
        ));
    }
}

/// Inner representation for JoinHandle
struct JoinInner<T> {
    // thread: Thread,
    receiver: Receiver<T>,
}

impl<T> JoinInner<T> {
    fn join(&mut self) -> Result<T> {
        let res = block_on(self.receiver.recv());
        res.map_err(|e| Box::new(e) as Box<(dyn Any + Send + 'static)>)
    }

    async fn join_async(&mut self) -> Result<T> {
        let res = self.receiver.recv().await;
        res.map_err(|e| Box::new(e) as Box<(dyn Any + Send + 'static)>)
    }
}

/// An owned permission to join on a thread (block on its termination).
pub struct JoinHandle<T>(JoinInner<T>);

impl<T> JoinHandle<T> {
    /// Extracts a handle to the underlying thread.
    pub fn thread(&self) -> &Thread {
        unimplemented!();
        //&self.0.thread
    }

    /// Waits for the associated thread to finish.
    pub fn join(mut self) -> Result<T> {
        self.0.join()
    }

    /// Waits for the associated thread to finish asynchronously.
    pub async fn join_async(mut self) -> Result<T> {
        self.0.join_async().await
    }
}

impl<T> fmt::Debug for JoinHandle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("JoinHandle { .. }")
    }
}

/// Spawns a new thread, returning a JoinHandle for it.
pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    Builder::new().spawn(f).expect("failed to spawn thread")
}
