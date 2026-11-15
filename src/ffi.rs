pub struct Event {}

pub const EPOLL_CTL_ADD: i32 = 1;
pub const EPOLLIN: i32 = 0x1;
pub const EPOLLET: i32 = 1 << 31;


#[link(name = "c")]
unsafe  extern "C" {
    pub unsafe fn epoll_create(size: i32) -> i32;
    pub unsafe fn close(fd: i32) -> i32;
    pub unsafe fn epoll_ctl(epfd: i32, op: i32, fd: i32, event: *mut Event) -> i32;
    pub unsafe fn epoll_wait(epfd: i32, events: *mut Event, maxevents: i32, timeout: i32) -> i32;
}
