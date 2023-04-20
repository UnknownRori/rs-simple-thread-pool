pub mod error;

mod message;
mod worker;

use crossbeam_channel::{unbounded, Sender};
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
    /// ## Example
    ///
    /// ```rust
    /// use std::{thread, time::Duration};
    /// use simple_thread_pool::{error::ThreadPoolError, ThreadPool};
    ///
    /// let pool = ThreadPool::new(2);
    /// let (send, recv) = unbounded();
    ///
    /// for _ in 0..4 {
    ///     let send = send.clone();
    ///
    ///     pool.execute(move || {
    ///         for _ in 0..40 {
    ///             // Simulate long process
    ///             thread::sleep(Duration::from_millis(10));
    ///         }
    ///
    ///         send.send(40).unwrap();
    ///     })?;
    /// }
    ///
    /// assert_eq!(recv.recv().unwrap(), 40);
    /// assert_eq!(recv.recv().unwrap(), 40);
    /// assert_eq!(recv.recv().unwrap(), 40);
    /// assert_eq!(recv.recv().unwrap(), 40);
    /// assert_eq!(recv.try_recv().is_err(), true);
    /// ```
    ///
    /// ## Panic
    ///
    /// May panic if the OS cannot create thread
    pub fn new(worker: usize) -> ThreadPool {
        let mut workers = Vec::with_capacity(worker);

        let (sender, receiver) = unbounded();

        for _ in 0..worker {
            workers.push(Worker::new(receiver.clone()));
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
