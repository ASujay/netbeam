use std::net::{Ipv4Addr, UdpSocket};

use local_ip_address::local_ip;

use crate::{error::{NetbeamResult}, packet::Packet};

pub const DEFAULT_UDP_SOCKET: u16 = 20069;

pub struct UdpNode {
    pub socket: UdpSocket,
    pub broadcast_addr: String,
    pub ip: Ipv4Addr,
}

impl UdpNode {
    pub fn new(port: Option<u16>) -> NetbeamResult<UdpNode> {
        let mut address = String::from("0.0.0.0:");
        if let Some(port) = port {
            address.push_str(&port.to_string());
        } else {
            address.push_str(&DEFAULT_UDP_SOCKET.to_string());
        }
        let socket = UdpSocket::bind(address)?;
        let local_ip_str: String = local_ip()?.to_string();
        let local_ip: Ipv4Addr = local_ip_str.parse()?;
        let [a, b, c, _] = local_ip.octets();
        let mut broadcast_addr = Ipv4Addr::new(a, b, c, 255).to_string();
        broadcast_addr.push(':');
        broadcast_addr.push_str(&DEFAULT_UDP_SOCKET.to_string());
        Ok(UdpNode { socket, broadcast_addr, ip: local_ip})
    }

    pub fn set_broadcast(&mut self) -> NetbeamResult<()> {
        self.socket.set_broadcast(true)?;
        Ok(())
    }

    pub fn send_packet(&self, packet: Packet, addr: &String) -> NetbeamResult<()> {
        let data = packet.serialize();
        self.socket.send_to(&data, addr)?;
        Ok(())
    }
}