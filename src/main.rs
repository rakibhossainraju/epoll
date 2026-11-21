mod client;
mod connection;
mod connections;
mod ffi;
mod poll;
mod registry;
mod utils;

use std::io::Result;

use poll::Poll;

const HOST: &str = "localhost";
const PORT: u16 = 8080;
const N_REQUESTS: usize = 5;

fn main() -> Result<()> {
    // Delay server runs in the background on its own thread.
    let _server = delayserver::spawn(HOST, PORT)?;

    let mut poll = Poll::new()?;
    let mut connections = client::open_requests(&mut poll, &format!("{HOST}:{PORT}"), N_REQUESTS)?;

    let mut finished = 0;
    while finished < N_REQUESTS {
        let mut events = Vec::with_capacity(10);
        poll.poll(&mut events, None)?;
        finished += connections.handle(&events)?;
    }

    println!("\n------\nFINISHED");
    Ok(())
}
