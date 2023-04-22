# Rust-Simple-ThreadPool

A very simple thread pool for networking and other stuff.

## 🛠️ Development

Make sure you have installed cargo and git

```bash
# clone repository
> git clone https://github.com/UnknownRori/rs-simple-thread-pool

# enter cloned repository
> cd simple-rust-thread-pool

# build the library
> cargo build
```

## 🚀 Usage

add this to your `Cargo.toml`

```toml
[dependencies]
unknownrori-simple-thread-pool = 0.1.2
```

```rust
use std::{thread, time::Duration};

use unknownrori_simple_thread_pool::{
    crossbeam_channel::unbounded,
    error::ThreadPoolError,
    ThreadPool,
};

fn main() -> Result<(), ThreadPoolError> {
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
```

if you want to use stdlib this

```toml
[dependencies]
unknownrori-simple-thread-pool = { version = "0.1.2", default-features = false, features = ["mpsc"] }
```

```rust
use std::sync::mpsc::channel;
use std::{thread, time::Duration};

use unknownrori_simple_thread_pool::{error::ThreadPoolError, ThreadPool};

fn main() -> Result<(), ThreadPoolError> {
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
```

## 🌟 Contribution

Feel free to contribute, send pull request or issue and i will take a look
