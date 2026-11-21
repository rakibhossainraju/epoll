use std::{
    io::{Result, Write},
    net::TcpStream,
};

use super::http::get_req;
use crate::{
    epoll::{self, Poll},
    net::Connections,
};

/// Opens `n` non-blocking requests against the delay server and registers
/// each one with `poll`. Later requests get shorter delays.
pub fn open_requests(poll: &mut Poll, addr: &str, n: usize) -> Result<Connections> {
    let mut connections = Connections::new();

    for i in 0..n {
        let delay = (n - i) * 1000;
        let request = get_req(&format!("/{delay}/request-{i}"));

        let mut stream = TcpStream::connect(addr)?;
        stream.set_nonblocking(true)?;
        stream.write_all(request.as_bytes())?;

        let token = poll
            .registry()
            .register(&stream, epoll::EPOLLIN | epoll::EPOLLET)?;
        connections.add(token, stream);
    }

    Ok(connections)
}
