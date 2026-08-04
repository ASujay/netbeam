use std::{sync::{Arc, atomic::{AtomicBool, Ordering}, mpsc::Sender}, thread::{self, JoinHandle}};
use crate::{errors::NBResult, event::Events};

#[derive(Clone)]
pub struct ShutdownSignal {
    flag: Arc<AtomicBool>,
}

impl ShutdownSignal {
    pub fn new() -> ShutdownSignal {
        ShutdownSignal { flag: Arc::new(AtomicBool::new(false)) }
    }
    pub fn shutdown(&self) {
        self.flag.store(true, Ordering::Release);
    }
    pub fn is_shutdown(&self)-> bool {
        self.flag.load(Ordering::Acquire)
    }
}

pub struct ThreadGroup {
    thread_handles: Vec<JoinHandle<NBResult<()>>>,
}

impl ThreadGroup {
    pub fn new() -> Self {
        ThreadGroup { thread_handles: Vec::new() }
    }
    pub fn spawn_thread<F>(&mut self, f: F) where F: FnOnce() -> NBResult<()> + Send + 'static {
        self.thread_handles.push(thread::spawn(f));
    }
    pub fn join_all(self) -> NBResult<()> {
        for handle in self.thread_handles {
            _ = handle.join();
        }
        Ok(())
    }
}

// this will contain handles for the data coming and out of threads

#[derive(Clone)]
pub struct ThreadContext {
    shutdown: ShutdownSignal,
    event_sender: Sender<Events>,
}

impl ThreadContext {
    pub fn new(shutdown: ShutdownSignal, event_sender: Sender<Events>) -> ThreadContext {
        ThreadContext { shutdown, event_sender }
    }

    pub fn is_shutdown(&self) -> bool {
        self.shutdown.is_shutdown()
    }

    pub fn shutdown(&self) {
        self.shutdown.shutdown();
    }
}