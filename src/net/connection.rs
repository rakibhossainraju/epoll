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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::Write,
        net::TcpListener,
        thread::sleep,
        time::{Duration, Instant},
    };

    /// Returns (server side, non-blocking client side wrapped in a Connection).
    fn pair() -> (TcpStream, Connection) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        client.set_nonblocking(true).unwrap();
        let (server, _) = listener.accept().unwrap();
        (server, Connection::new(client))
    }

    #[test]
    fn open_idle_connection_would_block_without_reporting_eof() {
        let (_server, mut conn) = pair();
        assert!(!conn.read_available().unwrap());
        assert!(!conn.closed);
    }

    #[test]
    fn drains_more_than_one_buffer_then_reports_eof_exactly_once() {
        let (mut server, mut conn) = pair();
        // Larger than the 4096-byte read buffer, so one read cannot drain it.
        server.write_all(&vec![b'x'; 10_000]).unwrap();
        drop(server);

        // EOF is only reachable once every byte has been read.
        let deadline = Instant::now() + Duration::from_secs(5);
        while !conn.read_available().unwrap() {
            assert!(Instant::now() < deadline, "never saw EOF");
            sleep(Duration::from_millis(1));
        }

        assert!(conn.closed);
        assert!(!conn.read_available().unwrap(), "EOF must be reported once");
    }
}
