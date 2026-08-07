use std::{net::{IpAddr, SocketAddr, UdpSocket}, thread::{self}, time::Duration};
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use crate::{common::{DEFAULT_TCP_PORT, DEFAULT_UDP_PORT}, device::Device, errors::NBResult, event::Events, packet::DiscoveryPacket, thread::ThreadContext};

fn get_broadcast_addrs() -> NBResult<Vec<IpAddr>> {
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
    let broadcast_addrs: Vec<IpAddr> = get_broadcast_addrs()?;
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
            )?;
            // send the acknowledgement to the receiver
            let ackn_packet = DiscoveryPacket::Ackn { request_id }.encode();
            _ = socket.send_to(ackn_packet.as_slice(), socket_addr)?;
        }
    }
    Ok(())
}
pub fn reply_to_sender(context: ThreadContext, socket: UdpSocket) -> NBResult<()> {
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
pub fn retransmit_to_sender(context: ThreadContext, socket: UdpSocket) -> NBResult<()> {
    Ok(())
}

#[cfg(test)]
mod test {
    use std::sync::mpsc::{self, Receiver, Sender};
    use crate::thread::ShutdownSignal;
    use super::*;

    // #[test]
    // fn protocol_packet_exchange() {
    //     let sender_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    //     let receiver_socket = UdpSocket::bind("127.0.0.1:0").unwrap();

    //     // set timeout so that test does not get stuck
    //     receiver_socket.set_read_timeout(Some(Duration::from_millis(500))).unwrap();
    //     let receiver_addr = receiver_socket.local_addr().unwrap();
    //     let (event_tx, event_rx): (Sender<Events>, Receiver<Events>) = mpsc::channel();
    //     let shutdown = ShutdownSignal::new();
    //     let context = ThreadContext::new()
    //     let receiver_addr
    // }
}
