use std::future::Future;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::ThreadId;
use std::thread::{self, JoinHandle};

pub trait Listen {
    type Data: Send + 'static;

    fn listen<F>(&self, f: F)
    where
        F: FnMut(Self::Data) + Send + 'static;
}

pub struct SensorComms {
    _name: String,
    tx: Sender<Message>,
    handle: Option<JoinHandle<()>>,
    worker_thread_id: ThreadId,
    rt: tokio::runtime::Handle,
}

enum Message {
    Job(Box<dyn FnOnce() + Send>),
    Shutdown,
}

impl SensorComms {
    /// Requires being called on a Tokio runtime (so we can grab Handle::current()).
    pub fn new(name: impl Into<String>) -> Self {
        let rt = tokio::runtime::Handle::try_current()
            .expect("SensorComms::new must be called on a Tokio runtime");
        Self::new_with_runtime(name, rt)
    }

    pub fn new_with_runtime(name: impl Into<String>, rt: tokio::runtime::Handle) -> Self {
        let name = name.into();
        let (tx, rx) = mpsc::channel::<Message>();

        let worker_name = name.clone();
        let handle = thread::Builder::new()
            .name(format!("sensor-worker-{}", worker_name))
            .spawn(move || worker_loop(rx))
            .expect("failed to spawn SensorComms worker");

        let worker_thread_id = handle.thread().id();

        Self {
            _name: name,
            tx,
            handle: Some(handle),
            worker_thread_id,
            rt,
        }
    }

    /// Synchronous handler variant (kept for convenience).
    pub fn listen_on<S, H>(&self, sensor: &S, handler: H)
    where
        S: Listen,
        H: FnMut(S::Data) + Send + 'static,
    {
        let tx = self.tx.clone();
        let handler = Arc::new(Mutex::new(handler));
        sensor.listen({
            let handler = Arc::clone(&handler);
            move |data: S::Data| {
                let handler = Arc::clone(&handler);
                let _ = tx.send(Message::Job(Box::new(move || {
                    if let Ok(mut h) = handler.lock() {
                        h(data);
                    }
                })));
            }
        });
    }

    /// Async handler variant. Your handler returns a Future; we spawn it on the stored Tokio runtime.
    pub fn listen_on_async<S, H, Fut>(&self, sensor: &S, handler: H)
    where
        S: Listen,
        H: Fn(S::Data) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let tx = self.tx.clone();
        let rt = self.rt.clone();
        let handler = Arc::new(handler);

        sensor.listen({
            let handler = Arc::clone(&handler);
            let rt = rt.clone();
            move |data: S::Data| {
                // Keep CARLA’s callback super light: just enqueue a job.
                let handler = Arc::clone(&handler);
                let rt = rt.clone();
                let _ = tx.send(Message::Job(Box::new(move || {
                    // Build the future and spawn it on Tokio.
                    let fut = (handler)(data);
                    rt.spawn(async move { fut.await });
                })));
            }
        });
    }
}

impl Drop for SensorComms {
    fn drop(&mut self) {
        let _ = self.tx.send(Message::Shutdown);
        if std::thread::current().id() != self.worker_thread_id {
            if let Some(handle) = self.handle.take() {
                let _ = handle.join();
            }
        }
    }
}

fn worker_loop(rx: Receiver<Message>) {
    while let Ok(msg) = rx.recv() {
        match msg {
            Message::Job(job) => job(),
            Message::Shutdown => break,
        }
    }
}
