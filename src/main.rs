mod ffi;
mod poll;

use std::{io::{self, Read, Result, Write}, net::TcpStream, thread, time::Duration};
use ffi::Event;
use poll::Poll;

fn main() -> Result<()> {
    // Delay server runs in the background on its own thread.
    let _server = delayserver::spawn("localhost", 8080);

    let poll = Poll::new()?;
    let n_events = 5;

    let mut streams: Vec<TcpStream> = Vec::new();
    let delay_server_addr = "localhost:8080";

    for i in 0..n_events {
        let delay = (n_events - i) * 1000;
        let url_path = format!("/{delay}/request-{i}");
        let request = get_req(&url_path);
        let mut stream = std::net::TcpStream::connect(delay_server_addr)?;
        stream.set_nonblocking(true)?;

        stream.write_all(request.as_bytes())?;
        poll.registry()
            .register(&stream, i, ffi::EPOLLIN | ffi::EPOLLET)?;

        streams.push(stream);
    }
    thread::sleep(Duration::from_secs(20));
    Ok(())
}

fn get_req(path: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\n\
             Host: localhost\r\n\
             Connection: close\r\n\
             \r\n"
    )
}
