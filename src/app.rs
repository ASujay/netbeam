use std::time::Duration;
use std::{env, thread};
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::mpsc::{self};
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use crate::common::{DEFAULT_UDP_PORT, UDP_SOCKET_ADDRESS};
use crate::device::DeviceRegistry;
use crate::errors::{NBError, NBResult};
use crate::event::{EventManager, Events};
use crate::packet::DiscoveryPacket;
use crate::thread::{ShutdownSignal, ThreadContext, ThreadGroup};

pub struct App {
    sender: FileSender,
    receiver: FileReceiver,
    event_manager: EventManager,
    state: AppState,
    thread_context: ThreadContext,
}

impl App {
    pub fn new() -> NBResult<Self> {
        let (event_sender, event_sreceiver) = mpsc::channel::<Events>();
        let shutdown = ShutdownSignal::new();
        let thread_context = ThreadContext::new(shutdown, event_sender);
        Ok(App { 
            sender: FileSender::new(thread_context.clone())?, 
            receiver: FileReceiver::new(thread_context.clone()),
            event_manager: EventManager::new(event_sreceiver),
            state: AppState::new(),
            thread_context,
        })
    }

    pub fn run(&mut self) -> NBResult<()> {
        // validate the command line argument
        // usage should be netbeam <send|receive>
        let args: Vec<String> = env::args().collect();
        if args.len() != 2 { return Err(NBError::InvalidCommandLineArgs); }
        let app_thread_group = match args[1].as_str() {
            "send" => self.sender.run()?,
            "receive" => self.receiver.run()?,
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
    socket: UdpSocket,
    context: ThreadContext,

}

impl FileSender {
    pub fn new(context: ThreadContext) -> NBResult<Self> {
        let sender_socket_address = format!("{}:{}", UDP_SOCKET_ADDRESS, DEFAULT_UDP_PORT);
        let socket = UdpSocket::bind(sender_socket_address)?;
        socket.set_broadcast(true)?;
        Ok(FileSender { socket, context })
    }

    pub fn run(&self) -> NBResult<ThreadGroup> {
        let mut thread_group = ThreadGroup::new();
        let broadcast_thread_context = self.context.clone();
        let socket = self.socket.try_clone()?;
        thread_group.spawn_thread(move || {
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
            while !broadcast_thread_context.is_shutdown() {
                for ip in &broadcast_addrs {
                    socket.send_to(DiscoveryPacket::Conn.encode().as_slice(), SocketAddr::new(*ip, DEFAULT_UDP_PORT))?;
                }
                thread::sleep(Duration::from_secs(5));
            }
            Ok(())
        });

        let reply_listener_context = self.context.clone();
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
    pub fn new(context: ThreadContext) -> Self {
        FileReceiver { context }
    }

    pub fn run(&self) -> NBResult<ThreadGroup> {
        let mut thread_group = ThreadGroup::new();
        let broadcast_listener_thread_context = self.context.clone();
        thread_group.spawn_thread(move || {
            while broadcast_listener_thread_context.is_shutdown() {

            }
            Ok(())
        });
        Ok(thread_group)
    }
}

pub struct AppState {
    pub is_running: bool,
    pub registry: DeviceRegistry,
}

impl AppState {
    fn new() -> Self {
        AppState { 
            is_running: true,
            registry: DeviceRegistry::new(),
        }
    }
}