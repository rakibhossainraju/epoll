use crate::registry::RegisterId;

pub const EPOLL_CTL_ADD: i32 = 1;
pub const EPOLLIN: i32 = 0x1;
pub const EPOLLET: i32 = 1 << 31;

#[link(name = "c")]
unsafe extern "C" {
    pub unsafe fn epoll_create1(flag: i32) -> i32;
    pub unsafe fn close(fd: i32) -> i32;
    pub unsafe fn epoll_ctl(epfd: i32, op: i32, fd: i32, event: *mut Event) -> i32;
    pub unsafe fn epoll_wait(epfd: i32, events: *mut Event, max_events: i32, timeout: i32) -> i32;
}

pub type Events = Vec<Event>;

#[derive(Debug)]
#[repr(C, packed)]
pub struct Event {
    pub(crate) events: u32,
    pub(crate) epoll_data: usize,
}

impl Event {
    pub fn new(events: u32, epoll_data: usize) -> Self {
        Self { events, epoll_data }
    }

    pub fn token(&self) -> RegisterId {
        self.epoll_data
    }
}
