use std::{net::{IpAddr, SocketAddr, UdpSocket}, thread::{self}, time::Duration};
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use crate::{common::{DEFAULT_TCP_PORT, DEFAULT_UDP_PORT}, device::Device, errors::NBResult, event::Events, packet::DiscoveryPacket, thread::ThreadContext};

pub struct Protocol;

impl Protocol {
    pub fn get_broadcast_addrs() -> NBResult<Vec<IpAddr>> {
        let mut broadcast_addrs = Vec::<IpAddr>::new();
        let interfaces = NetworkInterface::show()?;
        for interface in interfaces {
            for addr in interface.addr {
                if let Some(broadcast_addr) = addr.broadcast() && !broadcast_addr.is_loopback() {
                    broadcast_addrs.push(broadcast_addr);
                } 
            }
        }
        Ok(broadcast_addrs)
    }

    pub fn broadcast(packet: Vec<u8>, context: ThreadContext, socket: UdpSocket) -> NBResult<()> {
        let broadcast_addrs: Vec<IpAddr> = Self::get_broadcast_addrs()?;
        while !context.is_shutdown() {
            for ip in &broadcast_addrs {
                socket.send_to(packet.as_slice(), SocketAddr::new(*ip, DEFAULT_UDP_PORT))?;
            }
            thread::sleep(Duration::from_secs(5));
        }
        Ok(())
    }

    pub fn reply_to_info(context: ThreadContext, socket: UdpSocket) -> NBResult<()> {
        let mut buf = [0u8; 1024];
        while !context.is_shutdown() {
            let (bytes_read, socket_addr) = socket.recv_from(&mut buf)?;
            if let Some(DiscoveryPacket::Info { port, request_id }) = DiscoveryPacket::decode(&buf[0..bytes_read]) {
                context.register_event(
                    Events::DeviceFound { 
                        request_id, 
                        device: Device::new(
                            String::from("Temp"), 
                            socket_addr.ip(), 
                            port
                        )     
                    }
                )?
            }
        }
        Ok(())
    }

    pub fn reply_to_conn(context: ThreadContext, socket: UdpSocket) -> NBResult<()> {
        while !context.is_shutdown() {
            let mut buf = [0u8; 1024];
            let (bytes_read, socket_address) = socket.recv_from(&mut buf)?;
            let packet = DiscoveryPacket::decode(&buf[0..bytes_read]);
            if let Some(packet) = packet {
                match packet {
                    DiscoveryPacket::Conn => {
                        // we need to send the reply to the sender
                        let packet = DiscoveryPacket::Info {
                            port: DEFAULT_TCP_PORT,
                            request_id: 0xFF,
                        }
                        .encode();
                        println!("{:?}", packet);
                        if let Err(e) = socket.send_to(packet.as_slice(), socket_address) {
                            eprint!("Error replying to broadcaster' {}", e);
                        }
                    }
                    DiscoveryPacket::Ackn { request_id } => {
                        _ = request_id;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}