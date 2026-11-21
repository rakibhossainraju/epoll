use super::{ffi, registry::Registry};
use std::io::{Error, Result};

pub struct Poll {
    registry: Registry,
}

impl Poll {
    pub fn new() -> Result<Self> {
        let raw_fd = unsafe { ffi::epoll_create1(0) };
        if raw_fd == -1 {
            return Err(Error::last_os_error());
        }
        Ok(Self {
            registry: Registry::new(raw_fd),
        })
    }

    pub fn registry(&mut self) -> &mut Registry {
        &mut self.registry
    }

    pub fn poll(&mut self, events: &mut ffi::Events, timeout: Option<i32>) -> Result<()> {
        let fd = self.registry.raw_fd;
        let timeout = timeout.unwrap_or(-1);
        let max_events = events.capacity() as i32;
        let available_events =
            unsafe { ffi::epoll_wait(fd, events.as_mut_ptr(), max_events, timeout) };

        if available_events == -1 {
            return Err(Error::last_os_error());
        }
        unsafe { events.set_len(available_events as usize) }

        Ok(())
    }
}
