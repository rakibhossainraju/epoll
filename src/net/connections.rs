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

    #[allow(dead_code)]
    pub fn remove(&mut self, id: RegisterId) {
        self.state.remove(&id);
    }

    /// Dispatches events to their connections and returns how many
    /// connections reached EOF for the first time.
    pub fn handle(&mut self, events: &[Event]) -> Result<usize> {
        let mut finished = 0;
        for event in events {
            if let Some(conn) = self.state.get_mut(&event.token()) {
                finished += conn.read_available()? as usize;
            }
        }
        Ok(finished)
    }
}
