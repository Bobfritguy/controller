use std::net::SocketAddr;
use std::net::{UdpSocket, IpAddr};
use std::sync::{Arc, Mutex};
use mdns::{Record, RecordKind};
use std::time::Duration;
use futures_util::{pin_mut, stream::StreamExt};
use std::str::FromStr;
use crate::SharedState;

pub struct Network {
    // Include fields relevant to network operations if needed
    // For example, UdpSocket if it's used across multiple functions
}

impl Network {
    // Constructor for Network
    pub fn new() -> Self {
        Network {
            // Initialize fields
        }
    }

    pub async fn discover_devices(shared_state: Arc<Mutex<SharedState>>) {
        let stream = mdns::discover::all("_controller._udp.local", Duration::from_secs(15))
            .expect("Failed to start mDNS discovery")
            .listen();
        pin_mut!(stream);

        let mut recent_port: Option<u16> = None;
        let mut recent_target: Option<String> = None;

        while let Some(Ok(response)) = stream.next().await {
            for record in response.records() {
                match &record.kind {
                    RecordKind::SRV { port, target, .. } => {
                        recent_port = Some(*port);
                        recent_target = Some(target.to_string());
                    },
                    RecordKind::A(ipv4_addr) => {
                        if let Some(port) = recent_port {
                            if Some(record.name.clone()) == recent_target {
                                let addr = SocketAddr::new(IpAddr::V4(*ipv4_addr), port);
                                let mut state = shared_state.lock().unwrap();
                                state.discovered_ips.push(addr);
                                recent_port = None;
                                recent_target = None;
                            }
                        }
                    },
                    RecordKind::AAAA(ipv6_addr) => {
                        if let Some(port) = recent_port {
                            if Some(record.name.clone()) == recent_target {
                                let addr = SocketAddr::new(IpAddr::V6(*ipv6_addr), port);
                                let mut state = shared_state.lock().unwrap();
                                state.discovered_ips.push(addr);
                                recent_port = None;
                                recent_target = None;
                            }
                        }
                    },
                    _ => (),
                }
            }
        }
    }



    fn extract_port(record: &Record) -> Option<u16> {
        if let RecordKind::SRV { port, .. } = record.kind {
            Some(port)
        } else {
            None
        }
    }

    // Helper function to convert mDNS record to IP address
    fn to_ip_addr(record: &Record) -> Option<IpAddr> {
        match record.kind {
            RecordKind::A(addr) => Some(IpAddr::V4(addr)),
            RecordKind::AAAA(addr) => Some(IpAddr::V6(addr)),
            _ => None,
        }
    }


    pub fn check_ip_string(ip_string: &str) -> bool {
        ip_string.parse::<SocketAddr>().is_ok()
    }

}

