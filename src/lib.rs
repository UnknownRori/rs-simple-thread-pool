use crossbeam_channel::{unbounded, Receiver, Sender};
use std::thread::{self, JoinHandle};

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
    Idle,
}

#[derive(Debug)]
pub enum ThreadPoolError {
    FailedToSendJob,
}

impl core::fmt::Display for ThreadPoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThreadPoolError::FailedToSendJob => f.write_fmt(format_args!(
                "Thread pool failed to send a job to it's worker!"
            ))?,
        };

        Ok(())
    }
}

#[derive(Debug)]
struct Worker {
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    /// Creates a new [`Worker`].
    ///
    /// ## Panic
    ///
    /// May panic when the OS cannot create thread
    fn new(receiver: Receiver<Message>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = match receiver.recv() {
                Ok(message) => message,
                Err(_) => Message::Idle,
            };

            let _ = match message {
                Message::NewJob(job) => job(),
                Message::Terminate => break,
                Message::Idle => (),
            };
        });

        Worker {
            thread: Some(thread),
        }
    }
}

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
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crossbeam_channel::unbounded;

    use crate::{ThreadPool, ThreadPoolError};

    /// Test the crossbeam thread pooling implementation
    ///
    /// ## Panic
    ///
    /// It may panic if the OS cannot create a thread
    #[test]
    fn test_crossbeam() -> Result<(), ThreadPoolError> {
        let pool = ThreadPool::new(2);
        let (send, recv) = unbounded();

        for _ in 0..4 {
            let send = send.clone();

            pool.execute(move || {
                for _ in 0..40 {
                    // Do some long proccess
                }

                send.send(40).unwrap();
            })?;
        }

        assert_eq!(recv.recv().unwrap(), 40);
        assert_eq!(recv.recv().unwrap(), 40);
        assert_eq!(recv.recv().unwrap(), 40);
        assert_eq!(recv.recv().unwrap(), 40);
        assert_eq!(recv.try_recv().is_err(), true);

        Ok(())
    }
}
