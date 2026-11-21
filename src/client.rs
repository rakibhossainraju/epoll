use std::{
    io::{Result, Write},
    net::TcpStream,
};

use crate::{connections::Connections, ffi, poll::Poll, utils::get_req};

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
            .register(&stream, ffi::EPOLLIN | ffi::EPOLLET)?;
        connections.add(token, stream);
    }

    Ok(connections)
}
