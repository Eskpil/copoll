### Copoll

Copoll or Cool Poll is a abstraction for the linux epoll api making it a tad bit
easier to poll your file descriptors. It does not provide a eventloop like mio
or calloop. Because the whole point of this create is leaving that to the user.

## Documentation

[Docs.rs](https://docs.rs/copoll/1.3.0/copoll/)


## Example

```rs
use copoll::{Interest, Mode, Token, Epoll, Event, Events};
use std::os::unix::{net::UnixListener, io::AsRawFd};
use std::time::Duration;

const LISTENER: Token = Token(0);

fn main() {
    let mut epoll = Epoll::create().unwrap();
    let mut listener = UnixListener::bind("test.sock").unwrap();
    epoll.register(listener.as_raw_fd(), LISTENER, Interest::Both, Mode::Edge).unwrap();

    loop {
        let mut events = epoll.poll(Some(Duration::from_millis(2000))).unwrap();

        for event in events.iter() {
            // Handle the event, read from the socket
            // respond to it etc
            // Here you could also use the utility functions provided in copoll::event;
            // example just breaks on first event
            break;
        }
    }
}
``` 
