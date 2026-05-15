use std::{io, process::exit, sync::{Arc, Mutex}, thread};

use crate::udp::UdpListener;

mod udp;
mod tcp;

fn usage_err() {
    eprintln!("Usage: netbeam send <file_name> | netbeam receive");
    exit(1);
}
fn invalid_cmd_err(cmd: &String) {
    eprintln!("Invalid command: {}", cmd);
    exit(1);
}

fn send(file_path: &String) {}

fn receive() -> io::Result<()>{
    let mut listener = UdpListener::new()?;
    let hostname = get_hostname()?;
    println!("Listening to connections...");
    let udp_handle = thread::spawn(move || {
        loop {
            match listener.socket.recv_from(&mut listener.recv_buf) {
                Ok(_) => {},
                Err(_) => {
                    break;
                },
            }
        }
    });
    let tcp_handle = thread::spawn(|| {});
    udp_handle.join();
    tcp_handle.join();
    Ok(())
}

fn get_hostname() -> io::Result<String> {
    let hostname = hostname::get()?.to_string_lossy().to_string();
    Ok(hostname)
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