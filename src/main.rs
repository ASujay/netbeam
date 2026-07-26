use std::{env, io, net::{IpAddr::{self}, SocketAddr, UdpSocket}, process::exit, thread, time::{Duration}};
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use rand::{RngExt, rng};

const DEFAULT_UDP_PORT: u16 = 11665;
const DEFAULT_TCP_PORT: u16 = 11666;
const DEFAULT_UDP_IP: &str = "0.0.0.0";
const DEFAULT_MAX_RETRIES: u16 = 20;

type NBResult<T> = Result<T, NBError>; 

#[derive(Debug)]
enum NBError {
    TcpSocketBindFail,
    UdpSocketBindFail(io::Error),
    NetworkInterfaceError(network_interface::Error),
}

impl From<io::Error> for NBError {
    fn from(value: io::Error) -> Self {
        NBError::UdpSocketBindFail(value)
    }
}

impl From<network_interface::Error> for NBError {
    fn from(value: network_interface::Error) -> Self {
        NBError::NetworkInterfaceError(value)
    }
}


enum UdpProtocolPacket {
    Conn,
    Info(u16, u64),
    Ack,
}

impl UdpProtocolPacket {
    fn create_packet(&self) -> Vec<u8> {
        match self {
            UdpProtocolPacket::Conn => b"CONN".to_vec(),
            UdpProtocolPacket::Info(port, request_id) => {
                let mut info_packet = Vec::<u8>::with_capacity(6);
                info_packet.extend_from_slice(b"INFO");
                // IMPORTANT(Aniket): Sender side should also use be scheme
                info_packet.extend_from_slice(&port.to_le_bytes());
                info_packet
            },
            UdpProtocolPacket::Ack => b"ACK".to_vec(),
        }
    }
}


#[derive(PartialEq)]
enum UdpCommMode {
    Send,
    Receive,
}

struct UdpComm {
    socket: UdpSocket,
    mode: UdpCommMode,
}

impl UdpComm {
    fn new(mode: UdpCommMode) -> NBResult<UdpComm> {
        let socket = UdpSocket::bind(&format!("{}:{}", DEFAULT_UDP_IP, DEFAULT_UDP_PORT))?;
        if mode == UdpCommMode::Send {
            socket.set_broadcast(true)?;
        }
        Ok(UdpComm {socket, mode})
    }

    fn send(&self) -> NBResult<()>{
        let interfaces = NetworkInterface::show()?;
        let mut broadcast_addrs: Vec<IpAddr> = Vec::new();
        for interface in interfaces {
            // collect all the address that we can send a broadcast to
            for addr in interface.addr {
                if let Some(addr) = addr.broadcast() && !addr.is_loopback(){
                    broadcast_addrs.push(addr);
                }
            }
        }

        let socket = self.socket.try_clone()?;
        // we will send broadcase and we will listen for acknowledgement
        let boardcast_handle = thread::spawn(move || {
            loop {
                for ip in &broadcast_addrs {
                    // send a CONN broadcast to the subnet
                    // TODO(Aniket): Fix this later, we should check what error we are getting
                    _ = socket.send_to(UdpProtocolPacket::Conn.create_packet().as_slice(), SocketAddr::new(*ip, DEFAULT_UDP_PORT));
                    
                }
                // wait for 5 secs to resend
                thread::sleep(Duration::from_secs(5));
            }
        });

        let socket = self.socket.try_clone()?;
        let info_receiver_handle = thread::spawn(move || {
            loop {
                let mut buf = [0u8; 1024];
                if let Ok((bytes_read, socket_addr)) = socket.recv_from(&mut buf) {
                    if &buf[0..(bytes_read - 2)] == b"INFO" {
                        let port: u16 = (buf[bytes_read - 2] as u16) | ((buf[bytes_read - 1] as u16) << 8);
                        println!("{}:{}", socket_addr.ip().to_string(), port);
                        // sending back the acknowledgement
                        _ = socket.send_to(UdpProtocolPacket::Conn.create_packet().as_slice(), socket_addr); 
                    }
                }
            }
        });

        _ = boardcast_handle.join();
        _ = info_receiver_handle.join();
        Ok(())
    }

    fn receive(&self) -> NBResult<()>{
        let socket = self.socket.try_clone()?;
        let mut buf = [0u8; 1024];
        if let Ok((bytes_read, socket_addr)) = socket.recv_from(&mut buf) {
            if &buf[0..bytes_read] == b"CONN" {
                println!("CONNECTION REQUEST: {}", socket_addr.ip());
                println!("SENDING RECEIVER INFO...");
                socket.send_to(UdpProtocolPacket::Info(DEFAULT_TCP_PORT, 0).create_packet().as_slice(), socket_addr)?;
            }
        }

        Ok(())
    }

    fn run(&self) -> NBResult<()> {
        match self.mode {
            UdpCommMode::Send => Ok(self.send()?),
            UdpCommMode::Receive => Ok(self.receive()?
        ),
        }
    }
}

struct TCPComm {}

impl TCPComm {
    fn new() -> NBResult<TCPComm> {
        Ok(TCPComm {})   
    }

    fn retry_until_bind(ip: &str, port: u16) -> NBResult<UdpSocket> {
        let mut port = port;
        let mut retry_count = 0;
        loop {
            match UdpSocket::bind(&format!("{}:{}", ip, port)) {
                Ok(socket) => return Ok(socket),
                Err(e) => {
                    if retry_count < DEFAULT_MAX_RETRIES {
                        eprintln!("Failed to bind socket: {}. Retrying...", e);
                        port += rng().random_range(1..10);
                        retry_count += 1;
                    } else {
                        return Err(NBError::TcpSocketBindFail);
                    }
                },
            };
        }
    }
}

fn main() -> NBResult<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Please provide operation mode!");
        eprintln!("Usage netbeam <send|receive>");
        exit(1);
    }

    let mut mode: Option<UdpCommMode> = None;
    match args[1].as_str() {
        "send" => {mode = Some(UdpCommMode::Send);},
        "receive" => {mode = Some(UdpCommMode::Receive);},
        _ => {},
    };

    if let Some(mode) = mode {
        let udp_comm = UdpComm::new(mode)?;
        udp_comm.run();
    } else {
        eprintln!("Invalid mode provided");
        eprintln!("Usage netbeam <send|receive>");
        exit(1);
    }
    Ok(())
}