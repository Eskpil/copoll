use std::{io, os::unix::io::RawFd};
use std::time::{Duration};
use std::os::unix::io::AsRawFd;

use nix::sys::epoll;


/// Describe what you are interested in polling
/// Readable means you are interested in the readable events
/// Writable means you are itnerested in the writable event
#[derive(Debug, Copy, Clone)]
pub enum Interest {
    Readable,
    Writable,
    Both
}


/// Describe what mode you want to poll the fd with
/// Level is the default linux behaviour
/// Edge is for edge-triggered notifications on the fd
/// OneShot is for one-shot notifications on the fd
#[derive(Debug, Copy, Clone)]
pub enum Mode {
   Level,
   Edge,
   OneShot
}

/// Readiness
/// readable marks the event as readable
/// writable marks the event as writable
/// error means that your event is an error
#[derive(Debug, Copy, Clone)]
pub struct Readiness {
    pub readable: bool,
    pub writable: bool,
    pub error: bool
}

/// A unique token indentifying a file descripting in the
/// Epoll instance
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Token(pub usize);

/// Epoll structure
#[derive(Debug)]
pub struct Epoll {
    epoll_fd: RawFd
}

/// Get the readiness and token of the event
#[derive(Debug)]
pub struct Event {
    pub readiness: Readiness,
    pub token: Token,
}

/// Shorthand for Vec<<epoll::EpollEvent>>
pub type Events = Vec<Event>;

fn make_flags(interest: Interest, mode: Mode) -> epoll::EpollFlags {
    let mut flags = epoll::EpollFlags::empty();
    
    match interest {
        Interest::Readable => flags |= epoll::EpollFlags::EPOLLIN,  
        Interest::Writable => flags |= epoll::EpollFlags::EPOLLOUT,
        Interest::Both => {
            flags |= epoll::EpollFlags::EPOLLIN;
            flags |= epoll::EpollFlags::EPOLLOUT;
        }
    }
    
    match mode {
        Mode::Level => { /* This is the default */ }
        Mode::Edge => flags |= epoll::EpollFlags::EPOLLET,
        Mode::OneShot => flags |= epoll::EpollFlags::EPOLLONESHOT,
    }

    flags 
}

fn flags_to_readiness(flags: epoll::EpollFlags) -> Readiness {
    Readiness {
        readable: flags.contains(epoll::EpollFlags::EPOLLIN),
        writable: flags.contains(epoll::EpollFlags::EPOLLOUT),
        error: flags.contains(epoll::EpollFlags::EPOLLERR),
    }
}


impl From<Token> for usize {
    fn from(val: Token) -> usize {
        val.0
    }
}

impl Epoll {
    /// Create a new epoll instance
    pub fn create() -> io::Result<Epoll> {
        let epoll_fd = epoll::epoll_create()?;
        Ok(Epoll { epoll_fd })
    }

    /// Poll the epoll instance for new events.
    /// Call this one on each iteration of your event loop
    pub fn poll(
        &mut self, 
        timeout: Option<Duration>
    ) -> io::Result<Events> {

        let mut buffer = [epoll::EpollEvent::empty(); 32];

        let timeout = timeout.map(|d| d.as_millis() as isize).unwrap_or(-1);

        let n_events = epoll::epoll_wait(
            self.epoll_fd, 
            &mut buffer,
            timeout,
        )?; 

        let mut events = buffer.iter().take(n_events).map(|event| Event {
            readiness: flags_to_readiness(event.events()),
            token: Token(event.data() as usize)
        }).collect(); 

        Ok(events)
    }

    /// Register a new file descriptor in the epoll instance
    pub fn register(
        &mut self,
        fd: RawFd,
        token: Token,
        interest: Interest,
        mode: Mode
    ) -> io::Result<()> {
         let mut event = epoll::EpollEvent::new(make_flags(interest, mode), usize::from(token) as u64); 
         epoll::epoll_ctl(self.epoll_fd, epoll::EpollOp::EpollCtlAdd, fd, &mut event)
            .map_err(Into::into)
    }

    /// Reregister a file descriptor in the epoll instance
    /// often used when wanting to change say the mode or interest
    pub fn reregister(
        &mut self,
        fd: RawFd,
        token: Token,
        interest: Interest,
        mode: Mode
    ) -> io::Result<()> {
         let mut event = epoll::EpollEvent::new(make_flags(interest, mode), usize::from(token) as u64); 
         epoll::epoll_ctl(self.epoll_fd, epoll::EpollOp::EpollCtlMod, fd, &mut event)
            .map_err(Into::into)
    }

    /// Stop polling events a file descriptor
    pub fn unregister(
        &mut self,
        fd: RawFd
    ) -> io::Result<()> {
        epoll::epoll_ctl(self.epoll_fd, epoll::EpollOp::EpollCtlDel, fd, None).map_err(Into::into)
    }

}

impl AsRawFd for Epoll {
    fn as_raw_fd(&self) -> RawFd {
       self.epoll_fd 
    }    
}

impl Drop for Epoll {
    fn drop(&mut self) {
        let _ = nix::unistd::close(self.epoll_fd);
    }
}

