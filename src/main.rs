mod ffi;
mod poll;
mod registry;
mod utils;

use std::{
    collections::HashMap,
    io::{self, Read, Result, Write},
    net::TcpStream,
};

use ffi::Event;
use poll::Poll;
use utils::get_req;
use registry::RegisterId;

struct Connection {
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
}

type Connections = HashMap<RegisterId, Connection>;

fn main() -> Result<()> {
    // Delay server runs in the background on its own thread.
    let _server = delayserver::spawn("localhost", 8080)?;

    let mut poll = Poll::new()?;
    let n_events = 5;

    let delay_server_addr = "localhost:8080";
    let mut streams: Connections = HashMap::new();

    for i in 0..n_events {
        let delay = (n_events - i) * 1000;
        let url_path = format!("/{delay}/request-{i}");
        let request = get_req(&url_path);
        let mut stream = std::net::TcpStream::connect(delay_server_addr)?;
        stream.set_nonblocking(true)?;

        stream.write_all(request.as_bytes())?;

        let token = poll
            .registry()
            .register(&stream, ffi::EPOLLIN | ffi::EPOLLET)?;

        streams.insert(token, Connection::new(stream));
    }

    let mut handled_events = 0;

    while handled_events < n_events {
        let mut events = Vec::with_capacity(10);
        poll.poll(&mut events, None)?;

        if events.is_empty() {
            println!("TIMEOUT (OR SPURIOUS EVENT NOTIFICATION) ");
            continue;
        }

        handled_events += handle_events(&events, &mut streams)?;
    }
    println!("FINISHED");
    Ok(())
}

fn handle_events(events: &[Event], streams: &mut Connections) -> Result<usize> {
    let mut handled_events = 0;
    for event in events {
        let register_id = event.token();
        let mut data = vec![0u8; 4096];
        loop {
            if let Some(connection) = streams.get_mut(&register_id) {
                match connection.stream.read(&mut data) {
                    Ok(0) => {
                        if !connection.closed {
                            connection.closed = true;
                            handled_events += 1;
                        }
                        break;
                    },
                    Ok(n) => {
                        let txt = String::from_utf8_lossy(&data[..n]);
                        println!("RECEIVED: {:?}", event);
                        println!("{txt}\n------\n");
                    }
                    // Not ready to read in a non-blocking manner. This could
                    // happen even if the event was reported as ready
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => { break; }
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => { continue; }
                    Err(e) => return Err(e),
                }
            }
            break;
        }
    }
    Ok(handled_events)
}
