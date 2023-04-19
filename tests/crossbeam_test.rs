use crossbeam_channel::unbounded;

use simple_thread_pool::{error::ThreadPoolError, ThreadPool};

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
