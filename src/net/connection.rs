use std::{
    io::{self, Read, Result},
    net::TcpStream,
};

pub struct Connection {
    stream: TcpStream,
    closed: bool,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            closed: false,
        }
    }

    /// Reads until `WouldBlock` or EOF (required for edge-triggered epoll).
    /// Returns `true` only the first time EOF is seen.
    pub fn read_available(&mut self) -> Result<bool> {
        let mut buf = [0u8; 4096];
        loop {
            match self.stream.read(&mut buf) {
                Ok(0) => {
                    let first = !self.closed;
                    self.closed = true;
                    return Ok(first);
                }
                Ok(n) => println!("\n------\n{}", String::from_utf8_lossy(&buf[..n])),
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(false),
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            }
        }
    }
}
