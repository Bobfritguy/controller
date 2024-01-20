use std::net::{UdpSocket, IpAddr};
use std::sync::{Arc, Mutex};
use mdns::{Record, RecordKind};
use std::time::Duration;
use futures_util::{pin_mut, stream::StreamExt};
use crate::SharedState;

pub struct Network {
    // Include fields relevant to network operations if needed
    // For example, UdpSocket if it's used across multiple functions
}

impl Network {
    // Constructor for Network
    pub fn new(/* params if needed */) -> Self {
        Network {
            // Initialize fields
        }
    }

    // Asynchronous function to discover devices via mDNS
    pub async fn discover_devices(shared_state: Arc<Mutex<SharedState>>) {
        let stream = mdns::discover::all("_controller._udp.local", Duration::from_secs(15))
            .expect("Failed to start mDNS discovery")
            .listen();

        let stream = stream.filter_map(|response| async {
            response.ok().and_then(|r| r.records().filter_map(to_ip_addr).next())
        });

        // Pin the stream before using it
        pin_mut!(stream);

        while let Some(addr) = stream.next().await {
            let mut state = shared_state.lock().unwrap();
            state.discovered_ips.push(addr);
        }
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

pub fn check_ip_string(ip_string: &String) -> bool{
    let default: String = "0.0.0.0:1234".to_string();
    let ip_max_length = "000.000.000.000:0000".to_string().len();
    let ip_min_length = default.len();
    if (ip_string.len() > ip_max_length) || (ip_string.len() < ip_min_length) {
        return false;
    }

    // Split into IP and Port
    let ip_port: Vec<&str> = ip_string.split(":").collect();
    if ip_port.len() != 2 {
        return false;
    }
    let ip = ip_port[0];
    let port = ip_port[1];

    // Convert the port to an int
    if port.len() != 4 {
        return false;
    }

    let _: u16 = match port.parse() {
        Ok(n) => {
            if n < 1{
                return false;
            }
            n
        },
        Err(_) => return false,
    };

    // Split the IP into octets
    let octets: Vec<&str> = ip.split(".").collect();
    if octets.len() != 4 {
        return false;
    }

    // Convert the octets to u8
    for octet in octets {
        match octet.parse::<u8>() {
            Ok(_) =>  continue,
            Err(_) => return false,
        };
    }

    true
}
