use crossbeam_channel::Receiver;
use std::thread::{self, JoinHandle};

use crate::message::Message;

#[derive(Debug)]
pub struct Worker {
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    /// Creates a new [`Worker`].
    ///
    /// ## Panic
    ///
    /// May panic when the OS cannot create thread
    pub fn new(receiver: Receiver<Message>) -> Worker {
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

    /// Returns the take thread of this [`Worker`].
    pub fn take_thread(&mut self) -> Option<JoinHandle<()>> {
        self.thread.take()
    }
}
