use crate::common::{DEFAULT_TCP_PORT, DEFAULT_UDP_PORT, UDP_SOCKET_ADDRESS};
use crate::device::{Device, DeviceRegistry};
use crate::errors::{NBError, NBResult};
use crate::event::{EventManager, Events};
use crate::packet::DiscoveryPacket;
use crate::thread::{ShutdownSignal, ThreadContext, ThreadGroup};
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::mpsc::{self};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{env, thread};

pub struct App {
    sender: FileSender,
    receiver: FileReceiver,
    event_manager: EventManager,
    state: AppState,
    thread_context: ThreadContext,
    socket: UdpSocket,
}

impl App {
    pub fn new() -> NBResult<Self> {
        let (event_sender, event_sreceiver) = mpsc::channel::<Events>();
        let shutdown = ShutdownSignal::new();
        let thread_context = ThreadContext::new(shutdown, event_sender);
        let sender_socket_address = format!("{}:{}", UDP_SOCKET_ADDRESS, DEFAULT_UDP_PORT);
        let socket = UdpSocket::bind(sender_socket_address)?;
        Ok(App {
            sender: FileSender::new(thread_context.clone(), &socket)?,
            receiver: FileReceiver::new(thread_context.clone())?,
            event_manager: EventManager::new(event_sreceiver),
            state: AppState::new(),
            thread_context,
            socket,
        })
    }

    pub fn run(&mut self) -> NBResult<()> {
        // validate the command line argument
        // usage should be netbeam <send|receive>
        let args: Vec<String> = env::args().collect();
        if args.len() != 2 {
            return Err(NBError::InvalidCommandLineArgs);
        }
        let app_thread_group = match args[1].as_str() {
            "send" => self.sender.run(&self.socket)?,
            "receive" => self.receiver.run(&self.socket)?,
            _ => return Err(NBError::InvalidCommandLineArgs),
        };
        while self.state.is_running {
            self.process_events();
        }
        self.thread_context.shutdown();
        _ = app_thread_group.join_all();
        Ok(())
    }

    pub fn process_events(&mut self) {
        self.event_manager.process_events(&mut self.state);
    }
}

struct FileSender {
    context: ThreadContext,
}

impl FileSender {
    pub fn new(context: ThreadContext, socket: &UdpSocket) -> NBResult<Self> {
        socket.set_broadcast(true)?;
        Ok(FileSender { context })
    }

    pub fn run(&self, udp_socket: &UdpSocket) -> NBResult<ThreadGroup> {
        let mut thread_group = ThreadGroup::new();
        let broadcast_thread_context = self.context.clone();
        let socket = udp_socket.try_clone()?;
        thread_group.spawn_thread(move || {
            let interfaces = NetworkInterface::show()?;
            let mut broadcast_addrs: Vec<IpAddr> = Vec::new();
            for interface in interfaces {
                // collect all the address that we can send a broadcast to
                for addr in interface.addr {
                    if let Some(addr) = addr.broadcast()
                        && !addr.is_loopback()
                    {
                        broadcast_addrs.push(addr);
                    }
                }
            }
            while !broadcast_thread_context.is_shutdown() {
                println!("Sending boardcast");
                for ip in &broadcast_addrs {
                    socket.send_to(
                        DiscoveryPacket::Conn.encode().as_slice(),
                        SocketAddr::new(*ip, DEFAULT_UDP_PORT),
                    )?;
                }
                thread::sleep(Duration::from_secs(5));
            }
            Ok(())
        });

        let mut reply_listener_context = self.context.clone();
        let socket = udp_socket.try_clone()?;
        let mut buf = [0u8; 1024];
        thread_group.spawn_thread(move || {
            while !reply_listener_context.is_shutdown() {
            }
            Ok(())
        });

        // let ui_thread_context = self.context.clone();
        // thread_group.spawn_thread(move || {
        //     Ok(())
        // });
        Ok(thread_group)
    }
}

struct FileReceiver {
    context: ThreadContext,
}

impl FileReceiver {
    pub fn new(context: ThreadContext) -> NBResult<Self> {
        Ok(FileReceiver { context })
    }

    pub fn run(&self, udp_socket: &UdpSocket) -> NBResult<ThreadGroup> {
        let mut thread_group = ThreadGroup::new();
        let broadcast_listener_thread_context = self.context.clone();
        let socket = udp_socket.try_clone()?;
        thread_group.spawn_thread(move || {
            while !broadcast_listener_thread_context.is_shutdown() {
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
                                eprint!("Error replying to broadcaster");
                            }
                        }
                        DiscoveryPacket::Ackn { request_id } => {}
                        _ => {}
                    }
                }
            }
            Ok(())
        });
        Ok(thread_group)
    }
}

pub struct AppState {
    pub is_running: bool,
    pub registry: Arc<Mutex<DeviceRegistry>>,
}

impl AppState {
    fn new() -> Self {
        AppState {
            is_running: true,
            registry: Arc::new(Mutex::new(DeviceRegistry::new())),
        }
    }
}
