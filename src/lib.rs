pub mod error;

mod message;
mod worker;

#[cfg(feature = "crossbeam")]
pub use crossbeam_channel;

#[cfg(feature = "crossbeam")]
use crossbeam_channel::{unbounded, Sender};

#[cfg(feature = "mpsc")]
use std::sync::mpsc::{channel, Sender};

#[cfg(feature = "mpsc")]
use std::sync::{Arc, Mutex};

use error::ThreadPoolError;
use message::Message;
use worker::Worker;

type Job = Box<dyn FnOnce() + Send + 'static>;

#[derive(Debug)]
pub struct ThreadPool {
    sender: Sender<Message>,
    workers: Vec<Worker>,
}

impl ThreadPool {
    /// Creates a new [`ThreadPool`].
    ///
    /// ## Panic
    ///
    /// May panic if the OS cannot create thread
    #[cfg(feature = "crossbeam")]
    pub fn new(worker: usize) -> ThreadPool {
        let mut workers = Vec::with_capacity(worker);

        let (sender, receiver) = unbounded();

        for _ in 0..worker {
            workers.push(Worker::new(receiver.clone()));
        }

        ThreadPool { workers, sender }
    }

    /// Creates a new [`ThreadPool`].
    ///
    /// ## Panic
    ///
    /// May panic if the OS cannot create thread
    #[cfg(feature = "mpsc")]
    pub fn new(worker: usize) -> ThreadPool {
        let mut workers = Vec::with_capacity(worker);

        let (sender, receiver) = channel();
        let receiver = Arc::new(Mutex::new(receiver));

        for _ in 0..worker {
            workers.push(Worker::new(Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    /// Execute a job to worker thread, it's require Closure with no param and no return
    ///
    /// ## Errors
    ///
    /// This function will return an error if the communication channel between worker thread
    /// and main thread is closed.
    pub fn execute<F>(&self, job: F) -> Result<(), ThreadPoolError>
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender
            .send(Message::NewJob(Box::new(job)))
            .or_else(|_| Err(ThreadPoolError::FailedToSendJob))?;

        Ok(())
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.take_thread() {
                thread.join().unwrap();
            }
        }
    }
}
