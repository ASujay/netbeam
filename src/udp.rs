use std::net::UdpSocket;
use std::io;

const DEFAULT_SOCKET: u16 = 20069;

pub struct UdpListener {
    pub socket: UdpSocket,
}

impl UdpListener {
    pub fn new() -> io::Result<UdpListener> {
        let mut address = String::from("0.0.0.0:");
        address.push_str(&DEFAULT_SOCKET.to_string());
        let socket = UdpSocket::bind(address)?;
        Ok(UdpListener { socket })
    }
}