use super::ffi;
use std::{
    io::{Error, Result},
    net::TcpStream,
    os::fd::AsRawFd,
};

pub type RegisterId = usize;

pub struct Registry {
    pub(super) raw_fd: i32,
    current_id: RegisterId,
}

impl Registry {
    pub fn new(raw_fd: i32) -> Self {
        Self {
            raw_fd,
            current_id: 0,
        }
    }
    /// Tells the kernel to watch `source` and returns the token that will
    /// come back to us in `Event::token()` whenever it becomes ready.
    ///
    /// This is the "write" half of the epoll round trip:
    /// `register` (epoll_ctl) puts an entry into the kernel's interest list,
    /// and `Poll::poll` (epoll_wait) later reads ready entries back out.
    pub fn register(&mut self, source: &TcpStream, interests: i32) -> Result<RegisterId> {
        // A fresh, unique id. The kernel never interprets it; it only stores
        // it next to the fd and hands it back untouched with each event.
        let token = self.next_id();

        // Build the entry on our stack: `events` is the mask of things we
        // care about (EPOLLIN | EPOLLET) and `epoll_data` carries the token.
        let mut event = ffi::Event::new(interests as u32, token);
        let op = ffi::EPOLL_CTL_ADD;

        // `epoll_ctl(ADD)` makes the kernel create an interest-list entry
        // that links three things: this epoll instance (`self.raw_fd`), the
        // stream's own fd (`source.as_raw_fd()`), and a COPY of `event`.
        // Because it is copied, our local `event` can be dropped as soon as
        // the call returns. (The `*mut` is only there to match the C
        // signature; ADD does not modify it.) From now on the kernel itself
        // watches the socket; we do nothing until `epoll_wait` reports it.
        let res = unsafe { ffi::epoll_ctl(self.raw_fd, op, source.as_raw_fd(), &mut event) };

        if res < 0 {
            return Err(Error::last_os_error());
        }
        Ok(token)
    }

    fn next_id(&mut self) -> RegisterId {
        self.current_id += 1;
        self.current_id
    }
}

impl Drop for Registry {
    fn drop(&mut self) {
        let res = unsafe { ffi::close(self.raw_fd) };
        if res < 0 {
            let err = Error::last_os_error();
            eprintln!("ERROR: {err:?}");
        }
    }
}
