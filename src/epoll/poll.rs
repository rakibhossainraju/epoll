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

    /// Blocks until at least one registered source is ready (or `timeout`
    /// ms pass; `None` waits forever), then fills `events` with what is ready.
    ///
    /// How the kernel hands events back to us:
    ///
    /// 1. The kernel keeps two lists per epoll instance. The *interest list*
    ///    holds everything `Registry::register` added (fd + event mask +
    ///    token). When a registered fd becomes ready, its entry is put on the
    ///    *ready list*.
    /// 2. `epoll_wait` sleeps until the ready list is non-empty, then COPIES
    ///    up to `max_events` entries out of it into memory we point it at.
    ///    Each copied entry is an `Event`: `events` is the bitmask of what
    ///    actually happened (e.g. EPOLLIN) and `epoll_data` is the token we
    ///    stored at registration, which is how we know which stream it was.
    /// 3. With `EPOLLET`, a reported entry leaves the ready list and only
    ///    comes back on the next edge (new data), so the caller must drain
    ///    the source until `WouldBlock`.
    pub fn poll(&mut self, events: &mut ffi::Events, timeout: Option<i32>) -> Result<()> {
        let fd = self.registry.raw_fd;
        // epoll_wait takes -1 to mean "block forever".
        let timeout = timeout.unwrap_or(-1);

        // `Vec::with_capacity(n)` reserved room for n `Event`s on the heap but
        // its `len` is still 0: the memory exists, it just holds no valid
        // events yet. The capacity (not the len) is therefore the most the
        // kernel may write, and we tell it exactly that.
        let max_events = events.capacity() as i32;

        // `as_mut_ptr()` is the address of the first slot of that heap buffer.
        // The kernel writes the ready events straight into it, one after
        // another, and returns how many it wrote (0 means the timeout hit).
        // Rust cannot see this write: from its point of view the Vec is
        // untouched, which is why the `unsafe` and the `set_len` below exist.
        let available_events =
            unsafe { ffi::epoll_wait(fd, events.as_mut_ptr(), max_events, timeout) };

        if available_events == -1 {
            return Err(Error::last_os_error());
        }

        // The Vec still says `len == 0`, so without this the events the
        // kernel just wrote would be invisible (iterating gives nothing).
        // `set_len` tells the Vec "the first N slots now hold valid values".
        //
        // SAFETY: N <= capacity because we passed `max_events = capacity`,
        // and the kernel fully initialised each of those N slots. That only
        // holds because `Event` is `#[repr(C, packed)]`, matching the layout
        // of the kernel's `struct epoll_event` byte for byte.
        unsafe { events.set_len(available_events as usize) }

        Ok(())
    }
}
