use crossbeam_channel::{unbounded, Receiver, Sender};
use std::{
    error::Error,
    thread::{self, JoinHandle},
};

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
