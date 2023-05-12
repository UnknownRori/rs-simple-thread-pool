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

use error::{FailedToSendJob, FailedToSpawnThread};
use message::Message;
use worker::Worker;

type Job = Box<dyn FnOnce() + Send + 'static>;

/// This is where the thread will be pooled
///
/// It depend on how you add this package on your project
/// you can either using Rust standard library
/// or you can use `crossbeam-channel`, the API is the same even on different feature flag.
///
/// ## Examples
///
/// ```rust,no_run
/// use std::{
///     io::Write,
///     net::{TcpListener, TcpStream},
///     thread,
///     time::Duration,
/// };
///
/// use unknownrori_simple_thread_pool::{error::FailedToSendJob, ThreadPool};
///
/// fn handle_connection(mut stream: TcpStream) {
///     thread::sleep(Duration::from_secs(2));
///
///     let response = "HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\nHi!";
///
///     stream.write_all(response.as_bytes()).unwrap();
///
///     thread::sleep(Duration::from_secs(2));
/// }
///
/// fn main() -> Result<(), FailedToSendJob> {
///     let pool = ThreadPool::new(2).unwrap();
///
///     let socket = TcpListener::bind("127.0.0.1:8000").unwrap();
///     println!("server started at http://127.0.0.1:8000");
///
///     for stream in socket.incoming() {
///         println!("Got stream!");
///         match stream {
///             Ok(stream) => pool.execute(|| handle_connection(stream))?,
///             Err(_) => eprintln!("Something is wrong!"),
///         }
///     }
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct ThreadPool {
    sender: Sender<Message>,
    workers: Vec<Worker>,
}

impl ThreadPool {
    /// Creates a new [`ThreadPool`], with passed worker args for how many worker thread to be created
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// use std::{thread, time::Duration};
    ///
    /// use unknownrori_simple_thread_pool::{
    ///     crossbeam_channel::unbounded,
    ///     error::FailedToSendJob,
    ///     ThreadPool,
    /// };
    ///
    /// fn main() -> Result<(), FailedToSendJob> {
    ///     let pool = ThreadPool::new(2).unwrap();
    ///     let (send, recv) = unbounded();
    ///
    ///     pool.execute(move || {
    ///         send.send(40).unwrap();
    ///     })?;
    ///
    ///     assert_eq!(recv.recv().unwrap(), 40);
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Error
    ///
    /// It will return an [`Err`] if cannot create thread worker
    #[cfg(feature = "crossbeam")]
    pub fn new(worker: usize) -> Result<ThreadPool, FailedToSpawnThread> {
        let workers = Vec::with_capacity(worker);

        let (sender, receiver) = unbounded();

        let mut threadpool = ThreadPool { workers, sender };
        for _ in 0..worker {
            let thread_builder = std::thread::Builder::new();

            let worker = Worker::new(receiver.clone(), thread_builder)
                .or_else(|_| Err(FailedToSpawnThread))?;

            threadpool.workers.push(worker);
        }

        Ok(threadpool)
    }

    /// Creates a new [`ThreadPool`], with passed worker args for how many worker thread to be created
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// use std::sync::mpsc::channel;
    /// use std::{thread, time::Duration};
    ///
    /// use unknownrori_simple_thread_pool::{error::FailedToSendJob, ThreadPool};
    ///
    /// fn main() -> Result<(), FailedToSendJob> {
    ///     let pool = ThreadPool::new(2).unwrap();
    ///     let (send, recv) = channel();
    ///
    ///     pool.execute(move || {
    ///         send.send(40).unwrap();
    ///     })?;
    ///
    ///     assert_eq!(recv.recv().unwrap(), 40);
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Error
    ///
    /// It will return an [`Err`] if cannot create thread worker
    #[cfg(feature = "mpsc")]
    pub fn new(worker: usize) -> Result<ThreadPool, FailedToSpawnThread> {
        let workers = Vec::with_capacity(worker);

        let (sender, receiver) = channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut threadpool = ThreadPool { sender, workers };
        for _ in 0..worker {
            let thread_builder = std::thread::Builder::new();

            let worker = Worker::new(Arc::clone(&receiver), thread_builder)
                .or_else(|_| Err(FailedToSpawnThread))?;

            threadpool.workers.push(worker);
        }

        Ok(threadpool)
    }

    /// Execute a job to worker thread, it's require Closure with no param and no return
    ///
    /// ## Errors
    ///
    /// This function will return an [`Err`] if the communication channel between worker thread
    /// and main thread is closed.
    pub fn execute<F>(&self, job: F) -> Result<(), FailedToSendJob>
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender
            .send(Message::NewJob(Box::new(job)))
            .or_else(|_| Err(FailedToSendJob))?;

        Ok(())
    }
}

impl Drop for ThreadPool {
    /// Make sure the [`ThreadPool`] do proper clean up with it's thread workers
    ///
    /// ## Panic
    ///
    /// May Panic if communcation between worker thread and main thread is closed
    /// or there are panic in worker thread.
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
