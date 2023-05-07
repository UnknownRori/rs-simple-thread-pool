#[derive(Debug)]
pub enum ThreadPoolError {
    FailedToSendJob,
}

impl core::fmt::Display for ThreadPoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThreadPoolError::FailedToSendJob => f.write_fmt(format_args!(
                "Thread pool failed to send a job to it's worker! the channel connection has been abruptly closed!"
            ))?,
        };

        Ok(())
    }
}

#[derive(Debug)]
pub struct FailedToSpawnThread;

impl core::fmt::Display for FailedToSpawnThread {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Thread pool failed to create worker thread!"))?;

        Ok(())
    }
}
