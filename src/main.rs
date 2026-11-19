mod ffi;
mod poll;

use ffi::Event;
use poll::Poll;
use std::{
    io::{self, Read, Result, Write},
    net::TcpStream,
};

fn main() -> Result<()> {
    // Delay server runs in the background on its own thread.
    let _server = delayserver::spawn("localhost", 8080)?;

    let mut poll = Poll::new()?;
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
    let mut handled_events = 0;
    // Tracks streams that already hit EOF, so a second EPOLLIN edge for the
    // same stream (e.g. from the FIN after the data) isn't counted twice.
    let mut done = vec![false; n_events];
    while handled_events < n_events {
        let mut events = Vec::with_capacity(10);
        poll.poll(&mut events, None)?;

        if events.is_empty() {
            println!("TIMEOUT (OR SPURIOUS EVENT NOTIFICATION) ");
            continue;
        }

        handled_events += handle_events(&events, &mut streams, &mut done)?;
    }
    println!("FINISHED");
    Ok(())
}

fn handle_events(
    events: &[Event],
    streams: &mut [TcpStream],
    done: &mut [bool],
) -> Result<usize> {
    let mut handled_events = 0;
    for event in events {
        let index = event.token();
        let mut data = vec![0u8; 4096];
        loop {
            match streams[index].read(&mut data) {
                Ok(n) if n == 0 => {
                    if !done[index] {
                        done[index] = true;
                        handled_events += 1;
                    }
                    break;
                }
                Ok(n) => {
                    let txt = String::from_utf8_lossy(&data[..n]);
                    println!("RECEIVED: {:?}", event);
                    println!("{txt}\n------\n");
                }
                // Not ready to read in a non-blocking manner. This could
                // happen even if the event was reported as ready
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            }
        }
    }
    Ok(handled_events)
}

fn get_req(path: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\n\
             Host: localhost\r\n\
             Connection: close\r\n\
             \r\n"
    )
}
