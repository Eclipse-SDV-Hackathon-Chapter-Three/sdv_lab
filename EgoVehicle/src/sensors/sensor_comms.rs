use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::thread::ThreadId;

/// Crate-local abstraction for “something you can listen to”.
/// Each implementor picks a concrete payload type via an associated type.
pub trait Listen {
    type Data: Send + 'static;

    /// Hook a listener. The callback may mutate internal state, so it’s FnMut.
    fn listen<F>(&self, f: F)
    where
        F: FnMut(Self::Data) + Send + 'static;
}

pub struct SensorComms {
    _name: String,
    tx: Sender<Message>,
    handle: Option<JoinHandle<()>>,
    worker_thread_id: ThreadId,
}

enum Message {
    Job(Box<dyn FnOnce() + Send>),
    Shutdown,
}

impl SensorComms {
    /// Spawn a single worker thread to run heavy jobs.
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let (tx, rx) = mpsc::channel::<Message>();

        let worker_name = name.clone();
        let handle = thread::Builder::new()
            .name(format!("sensor-worker-{}", worker_name))
            .spawn(move || worker_loop(rx))
            .expect("failed to spawn SensorComms worker");

        // Capture the worker thread id for Drop-time deadlock avoidance.
        let worker_thread_id = handle.thread().id();

        Self {
            _name: name,
            tx,
            handle: Some(handle),
            worker_thread_id,
        }
    }

    /// Wire a typed sensor to this worker. You supply the handler for `S::Data`.
    ///
    /// We keep the CARLA callback cheap; heavy work is pushed to the worker thread.
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
                // Enqueue heavy work onto the worker thread.
                let handler = Arc::clone(&handler);
                let _ = tx.send(Message::Job(Box::new(move || {
                    if let Ok(mut h) = handler.lock() {
                        h(data);
                    }
                })));
            }
        });
    }
}

impl Drop for SensorComms {
    fn drop(&mut self) {
        // Try to signal the worker to exit. Ignore errors if it already died.
        let _ = self.tx.send(Message::Shutdown);

        // Avoid deadlocking by joining from the same thread as the worker.
        if std::thread::current().id() != self.worker_thread_id {
            if let Some(handle) = self.handle.take() {
                let _ = handle.join(); // ignore poison/err on shutdown
            }
        } else {
            // If we are the worker thread, we cannot join ourselves—just exit.
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
