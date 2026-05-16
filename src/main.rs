use std::sync::Arc;
use std::{thread};
use crate::error::NetbeamResult;
use crate::packet::{Packet, DiscoveryAckPacket};
use crate::udp::{UdpNode};
use crate::utils::{get_hostname, invalid_cmd_err, usage_err};

mod udp;
mod tcp;
mod utils;
mod error;
mod packet;

fn send(file_path: &String) -> NetbeamResult<()> {
    _ = file_path;
    let mut listener = UdpNode::new(None)?;
    listener.set_broadcast()?;
    println!("Sending discovery packet");
    // we will send the discovery packet
    listener.send_packet(Packet::Discovery, &listener.broadcast_addr)?;
    let mut buf = [0u8; 1024];
    let (size, _) = listener.socket.recv_from(&mut buf)?;
    println!("{}", size);
    Ok(())
}

fn receive() -> NetbeamResult<()>{
    let listener = UdpNode::new(None)?; 
    let mut buf = [0u8;1024];
    let hostname = Arc::new(get_hostname()?);
    println!("Listening to connections...");
    let udp_handle = thread::spawn(move || {
        let hostname = Arc::clone(&hostname);
        let (size, src_addr) = listener.socket.recv_from(&mut buf).unwrap();
        let scan_message = String::from_utf8_lossy(&buf[..size]);
        println!("{}", scan_message);
        println!("{}", src_addr);
        println!("Sending the acknowledgement");
        let result = listener.send_packet(Packet::DiscoveryAck(DiscoveryAckPacket{
            hostname: (*hostname).clone(),
            ip: listener.ip,
        }), &src_addr.to_string());
        match result {
            Ok(()) => {},
            Err(err) => {
                println!("{:?}", err);
            },
        }
    });
    _ = udp_handle.join();
    Ok(())
}

fn main() -> NetbeamResult<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        usage_err();
    }
    match args[1].as_str() {
        "send" => {
            if args.len() == 3 {
                send(&args[2])?;
            } else {
                usage_err();
            }
        },
        "receive" => receive()?,
        _ => invalid_cmd_err(&args[1]),
    };
    Ok(())
}