use std::net::{SocketAddr};

pub struct SharedState {
    // Fields go here
    pub discovered_ips: Vec<SocketAddr>,
    pub service: String,
}

impl SharedState {
    // Define a new method to create an instance of SharedState
    pub fn new(service: String) -> Self {
        SharedState {
            // Initialize the fields
            discovered_ips: Vec::new(),
            service,
        }
    }
    pub fn set_service(&mut self, service: String) {
        self.service = service;
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    Sending,
    Receiving,
    Stopped,
    Settings,
}