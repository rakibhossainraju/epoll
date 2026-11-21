use crate::ffi;
use std::{
    io::{Error, Result},
    net::TcpStream,
    os::fd::AsRawFd,
    sync::atomic::{AtomicUsize, Ordering},
};

pub type RegisterId = usize;

pub struct Registry {
    pub raw_fd: i32,
    current_id: AtomicUsize,
}

impl Registry {
    pub fn new(raw_fd: i32) -> Self {
        Self {
            raw_fd,
            current_id: AtomicUsize::new(0),
        }
    }
    pub fn register(&mut self, source: &TcpStream, interests: i32) -> Result<RegisterId> {
        let token = self.next_id();
        let mut event = ffi::Event::new(interests as u32, token);
        let op = ffi::EPOLL_CTL_ADD;

        let res = unsafe { ffi::epoll_ctl(self.raw_fd, op, source.as_raw_fd(), &mut event) };

        if res < 0 {
            return Err(Error::last_os_error());
        }
        Ok(token)
    }

    fn next_id(&self) -> RegisterId {
        self.current_id.fetch_add(1, Ordering::Relaxed) + 1
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
