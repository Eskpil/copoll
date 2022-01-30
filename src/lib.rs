use std::{io, os::unix::io::RawFd};
use std::time::{Duration};
use std::os::unix::io::AsRawFd;

use nix::sys::epoll;

#[derive(Debug, Copy, Clone)]
pub enum Interest {
    Readable,
    Writable,
    Both
}

#[derive(Debug, Copy, Clone)]
pub enum Mode {
   Level,
   Edge,
   OneShot
}

#[derive(Debug, Copy, Clone)]
pub struct Readiness {
    pub readable: bool,
    pub writable: bool,
    pub error: bool
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Token(pub usize);

#[derive(Debug)]
pub struct Epoll {
    epoll_fd: RawFd
}

pub type Event = epoll::EpollEvent;
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
    pub fn create() -> io::Result<Epoll> {
        let epoll_fd = epoll::epoll_create1(epoll::EpollCreateFlags::EPOLL_CLOEXEC)?;
        Ok(Epoll { epoll_fd })
    }

    pub fn poll(
        &mut self, 
        events: &mut Events,
        timeout: Option<Duration>
    ) -> io::Result<()> {
        let timeout = timeout.map(|d| d.as_millis() as isize).unwrap_or(-1);

        events.clear();
        
        let n_events = epoll::epoll_wait(
            self.epoll_fd, 
            events,
            timeout,
        )?; 

        unsafe {
            events.set_len(n_events as usize)
        };

        Ok(())
    }

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

pub mod event {
        
    use crate::{Event, Token, Readiness, flags_to_readiness};

    pub fn token(event: &Event) -> Token {
       Token(event.data() as usize) 
    }

    pub fn readiness(event: &Event) -> Readiness {
        flags_to_readiness(event.events())
    }

}
