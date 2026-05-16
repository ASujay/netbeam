use std::net::{TcpListener, UdpSocket};
use std::{io, thread};
use crate::udp::{BROADCAST_ADDR, UdpNode};
use crate::utils::{usage_err, invalid_cmd_err};

mod udp;
mod tcp;
mod utils;

fn send(file_path: &String) -> io::Result<()> {
    let listener = UdpNode::new(Some(0))?;
    listener.socket.set_broadcast(true)?;
    println!("Sending discovery packet");
    // we will send the discovery packet
    listener.socket.send_to(b"DISCOVERY", BROADCAST_ADDR)?;
    Ok(())
}

fn receive() -> io::Result<()>{
    let listener = UdpNode::new(None)?; 
    let mut buf = [0u8;1024];
    println!("Listening to connections...");
    let udp_handle = thread::spawn(move || {
        loop {
            println!("Yosh");
            let (size, src_addr) = listener.socket.recv_from(&mut buf).unwrap();
            let scan_message = String::from_utf8_lossy(&buf[..size]);
            println!("{}", scan_message);
            println!("{}", src_addr);
        }
    });
    _ = udp_handle.join();
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        usage_err();
    }
    match args[1].as_str() {
        "send" => {
            if args.len() == 3 {
                send(&args[2]);
            } else {
                usage_err();
            }
        },
        "receive" => receive()?,
        _ => invalid_cmd_err(&args[1]),
    };
    Ok(())
}