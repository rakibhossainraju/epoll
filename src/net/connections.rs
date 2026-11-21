use std::{collections::HashMap, io::Result, net::TcpStream};

use super::connection::Connection;
use crate::epoll::{Event, RegisterId};

#[derive(Default)]
pub struct Connections {
    state: HashMap<RegisterId, Connection>,
}

impl Connections {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, id: RegisterId, stream: TcpStream) {
        self.state.insert(id, Connection::new(stream));
    }

    pub fn remove(&mut self, id: RegisterId) {
        self.state.remove(&id);
    }

    /// Dispatches events to their connections. A connection that reaches EOF
    /// is removed (dropping it closes the fd, which also removes it from the
    /// epoll set), so a later event for the same token is ignored.
    /// Returns how many connections finished.
    pub fn handle(&mut self, events: &[Event]) -> Result<usize> {
        let mut finished = 0;
        for event in events {
            let id = event.token();
            let Some(conn) = self.state.get_mut(&id) else {
                continue;
            };
            println!("RECEIVED: {event:?}");
            if conn.read_available()? {
                self.remove(id);
                finished += 1;
            }
        }
        Ok(finished)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::epoll::EPOLLIN;
    use std::{
        net::TcpListener,
        thread::sleep,
        time::{Duration, Instant},
    };

    fn add_closed_peer(connections: &mut Connections, id: RegisterId) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        client.set_nonblocking(true).unwrap();
        drop(listener.accept().unwrap()); // peer closes immediately
        connections.add(id, client);
    }

    #[test]
    fn finished_connection_is_removed_and_counted_once() {
        let mut connections = Connections::new();
        add_closed_peer(&mut connections, 1);
        let event = Event::new(EPOLLIN as u32, 1);

        let deadline = Instant::now() + Duration::from_secs(5);
        let mut finished = 0;
        while finished == 0 {
            assert!(Instant::now() < deadline, "never finished");
            finished = connections.handle(&[Event::new(EPOLLIN as u32, 1)]).unwrap();
            sleep(Duration::from_millis(1));
        }

        assert_eq!(finished, 1);
        assert!(connections.state.is_empty());
        // A stale event for the removed token is ignored, not double-counted.
        assert_eq!(connections.handle(&[event]).unwrap(), 0);
    }

    #[test]
    fn unfinished_connection_stays_in_the_map() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        client.set_nonblocking(true).unwrap();
        let (_server, _) = listener.accept().unwrap(); // peer stays open

        let mut connections = Connections::new();
        connections.add(1, client);

        let finished = connections.handle(&[Event::new(EPOLLIN as u32, 1)]).unwrap();
        assert_eq!(finished, 0);
        assert_eq!(connections.state.len(), 1);
    }
}
