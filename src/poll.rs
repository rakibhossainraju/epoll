use crate::ffi;
use std::{
    io::{self, Result},
    net::TcpStream,
    os::fd::AsRawFd,
};

type Events = Vec<ffi::Event>;

struct Registry {
    raw_fd: i32,
}

pub struct Poll {
    registry: Registry,
}

impl Poll {
    pub fn new() -> Result<Self> {
        todo!()
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    pub fn pull(&mut self, events: &mut Events, timeout: Option<i32>) -> Result<()> {
        todo!()
    }
}

impl Registry {
    pub fn register(&self, source: &TcpStream, token: usize, interests: i32) -> Result<()> {
        todo!()
    }
}
impl Drop for Registry {
    fn drop(&mut self) {
        todo!()
    }
}
