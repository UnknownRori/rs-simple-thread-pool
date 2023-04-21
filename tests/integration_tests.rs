#[cfg(feature = "mpsc")]
#[cfg(test)]
mod mpsc {
    use std::sync::mpsc::channel;
    use std::{thread, time::Duration};

    use unknownrori_simple_thread_pool::{error::ThreadPoolError, ThreadPool};

    /// Test the crossbeam thread pooling implementation
    ///
    /// ## Panic
    ///
    /// It may panic if the OS cannot create a thread
    #[test]
    fn test_mpsc() -> Result<(), ThreadPoolError> {
        let pool = ThreadPool::new(2);

        let (send, recv) = channel();

        for _ in 0..4 {
            let send = send.clone();

            pool.execute(move || {
                for _ in 0..40 {
                    // Simulate long process
                    thread::sleep(Duration::from_millis(10));
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

#[cfg(feature = "crossbeam")]
#[cfg(test)]
mod crossbeam {
    use std::{thread, time::Duration};

    use crossbeam_channel::unbounded;

    use unknownrori_simple_thread_pool::{error::ThreadPoolError, ThreadPool};

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
                    // Simulate long process
                    thread::sleep(Duration::from_millis(10));
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
