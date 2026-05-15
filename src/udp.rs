use std::{io, net::UdpSocket};

const DEFUALT_PORT: u16 = 22069;

pub struct UdpListener {
    pub port: u16,
    pub socket: UdpSocket,
    pub recv_buf: [u8;1024],
}

impl UdpListener {
    pub fn new() -> io::Result<UdpListener> {
        let mut bind_address = String::from("0.0.0.0:");
        bind_address.push_str(&DEFUALT_PORT.to_string());
        Ok(UdpListener {
            port: DEFUALT_PORT,
            socket: UdpSocket::bind(bind_address)?,
            recv_buf: [0;1024],
        })
    }
}