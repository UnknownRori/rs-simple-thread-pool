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
/// use unknownrori_simple_thread_pool::{error::ThreadPoolError, ThreadPool};
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
/// fn main() -> Result<(), ThreadPoolError> {
///     let pool = ThreadPool::new(2);
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
    ///     error::ThreadPoolError,
    ///     ThreadPool,
    /// };
    ///
    /// fn main() -> Result<(), ThreadPoolError> {
    ///     let pool = ThreadPool::new(2);
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

    /// Creates a new [`ThreadPool`], with passed worker args for how many worker thread to be created
    ///
    /// ## Examples
    ///
    /// ```rust,no_run
    /// use std::sync::mpsc::channel;
    /// use std::{thread, time::Duration};
    ///
    /// use unknownrori_simple_thread_pool::{error::ThreadPoolError, ThreadPool};
    ///
    /// fn main() -> Result<(), ThreadPoolError> {
    ///     let pool = ThreadPool::new(2);
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
