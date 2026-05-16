use std::net::UdpSocket;
use std::io;

pub const DEFAULT_SOCKET: u16 = 20069;
pub const BROADCAST_ADDR: &str = "255.255.255.255:20069";

pub struct UdpNode {
    pub socket: UdpSocket,
}

impl UdpNode {
    pub fn new(port: Option<u16>) -> io::Result<UdpNode> {
        let mut address = String::from("0.0.0.0:");
        if let Some(port) = port {
            address.push_str(&port.to_string());
        } else {
            address.push_str(&DEFAULT_SOCKET.to_string());
        }
        let socket = UdpSocket::bind(address)?;
        Ok(UdpNode { socket })
    }
}