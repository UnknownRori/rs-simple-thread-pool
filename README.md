# UnknownRori-Simple-Thread-Pool

A Thread Pool that focused on lightweight.

## 🚀 Usage

By default `unknownrori-simple-thread-pool` uses `crossbeam-channel` not `mpsc` that standard library provided

```sh
# If you want to use crossbeam-channel package
> cargo add unknownrori-simple-thread-pool

# If you want to use mpsc from rust standard library
> cargo add unknownrori-simple-thread-pool --no-default-features -F mpsc
```

```rust
use std::{
    io::Write,
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use unknownrori_simple_thread_pool::{error::FailedToSendJob, ThreadPool};

fn handle_connection(mut stream: TcpStream) {
    thread::sleep(Duration::from_secs(2));

    let response = "HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\nHi!";

    stream.write_all(response.as_bytes()).unwrap();

    thread::sleep(Duration::from_secs(2));
}

fn main() -> Result<(), ThreadPoolError> {
    let pool = ThreadPool::new(2).unwrap();

    let socket = TcpListener::bind("127.0.0.1:8000").unwrap();
    println!("server started at http://127.0.0.1:8000");

    for stream in socket.incoming() {
        println!("Got stream!");

        match stream {
            Ok(stream) => pool.execute(|| handle_connection(stream))?,
            Err(_) => eprintln!("Something is wrong!"),
        }
    }

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
