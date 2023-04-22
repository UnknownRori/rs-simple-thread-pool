# Rust-Simple-Thread-Pool

A Thread Pool that focused on lightweight.

## 🚀 Usage

By default `unknownrori-simple-thread-pool` uses `crossbeam-channel` not `mpsc` that standard library provided

```sh
> cargo add unknownrori-simple-thread-pool
```

### crossbeam-channel

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

    pool.execute(move || {
        send.send(40).unwrap();
    })?;

    assert_eq!(recv.recv().unwrap(), 40);

    Ok(())
}

### mpsc

To use `std::sync::mpsc` instead of `crossbeam-channel` package, run this command

```sh
> cargo add unknownrori-simple-thread-pool --no-default-features -F mpsc
```

```rust
use std::sync::mpsc::channel;
use std::{thread, time::Duration};

use unknownrori_simple_thread_pool::{error::ThreadPoolError, ThreadPool};

fn main() -> Result<(), ThreadPoolError> {
    let pool = ThreadPool::new(2);
    let (send, recv) = channel();

    pool.execute(move || {
        send.send(40).unwrap();
    })?;

    assert_eq!(recv.recv().unwrap(), 40);

    Ok(())
}
```

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

## 🌟 Contribution

Feel free to contribute, send pull request or issue and i will take a look
