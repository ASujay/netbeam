use crate::{
    common::{DEFAULT_RETRANSMIT_PERIOD, DEFAULT_TCP_PORT, DEFAULT_UDP_PORT, Request},
    device::Device,
    errors::NBResult,
    event::Events,
    packet::DiscoveryPacket,
    thread::ThreadContext,
};
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use std::{
    net::{IpAddr, SocketAddr, UdpSocket},
    thread::{self},
    time::Duration,
};

fn get_broadcast_addrs() -> NBResult<Vec<IpAddr>> {
    let mut broadcast_addrs = Vec::<IpAddr>::new();
    let interfaces = NetworkInterface::show()?;
    for interface in interfaces {
        for addr in interface.addr {
            if let Some(broadcast_addr) = addr.broadcast()
                && !broadcast_addr.is_loopback()
            {
                broadcast_addrs.push(broadcast_addr);
            }
        }
    }
    Ok(broadcast_addrs)
}
pub fn broadcast(context: ThreadContext, socket: UdpSocket) -> NBResult<()> {
    // we can let the thread crash here since if we cannot get the network interfaces we cannot continue
    let packet = DiscoveryPacket::Conn.encode();
    let broadcast_addrs: Vec<IpAddr> = get_broadcast_addrs()?;
    while !context.is_shutdown() {
        println!("Broadcasting on {} interfaces", broadcast_addrs.len());
        for ip in &broadcast_addrs {
            let destination = SocketAddr::new(*ip, DEFAULT_UDP_PORT);

            // we dont want the thread to crash here when an error occurs we want to continue broadcasting on other interfaces
            match socket.send_to(packet.as_slice(), destination) {
                Ok(_) => {}
                Err(error) => {
                    _ = context.register_event(Events::BroadcastError { destination, error });
                }
            };
        }
        thread::sleep(Duration::from_secs(5));
    }
    Ok(())
}
pub fn reply_to_info(context: ThreadContext, socket: UdpSocket) -> NBResult<()> {
    let mut buf = [0u8; 1024];
    while !context.is_shutdown() {
        let (bytes_read, socket_addr) = socket.recv_from(&mut buf)?;
        if let Some(DiscoveryPacket::Info { port, request_id }) =
            DiscoveryPacket::decode(&buf[0..bytes_read])
        {
            context.register_event(Events::DeviceFound {
                request_id,
                device: Device::new(String::from("Temp"), socket_addr.ip(), port),
            })?;
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
                    let request_id: u64 = rand::random();
                    let port = DEFAULT_TCP_PORT;
                    // we need to send the reply to the sender
                    let packet = DiscoveryPacket::Info { port, request_id }.encode();
                    let request = Request::new(request_id, port, socket_address);
                    if let Err(e) = socket.send_to(packet.as_slice(), socket_address) {
                        eprintln!("Error replying to broadcaster' {}", e);
                    } else {
                        // save the request to the registry
                        context.register_event(Events::AddRequest {
                            request_id,
                            request,
                        })?;
                    }
                }
                DiscoveryPacket::Ackn { request_id } => {
                    // remove the request to the registry
                    context.register_event(Events::RemoveRequest(request_id))?;
                }
                _ => {}
            }
        }
    }
    Ok(())
}
pub fn retransmit_to_sender(context: ThreadContext, socket: UdpSocket) -> NBResult<()> {
    // we need to continue sending the request packets for all pending requests
    while !context.is_shutdown() {
        // send the retransmit event
        _ = context.register_event(Events::RetransmitPendingPackets(socket.try_clone()?));
        // sleep for a fixed amount of time
        thread::sleep(Duration::from_secs(DEFAULT_RETRANSMIT_PERIOD));
    }
    Ok(())
}

// #[cfg(test)]
// mod test {
//     use std::sync::mpsc::{self, Receiver, Sender};
//     use crate::thread::ShutdownSignal;
//     use super::*;

//     #[test]
//     fn protocol_packet_exchange() {
//         let sender_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
//         let receiver_socket = UdpSocket::bind("127.0.0.1:0").unwrap();

//         // set timeout so that test does not get stuck
//         receiver_socket.set_read_timeout(Some(Duration::from_millis(500))).unwrap();
//         let receiver_addr = receiver_socket.local_addr().unwrap();
//         let (event_tx, event_rx): (Sender<Events>, Receiver<Events>) = mpsc::channel();
//         let shutdown = ShutdownSignal::new();
//         let context = ThreadContext::new()
//         let receiver_addr
//     }
// }
