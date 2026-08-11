use crate::common::{DEFAULT_UDP_IP, DEFAULT_UDP_PORT, RegistryId, Request};
use crate::device::{Device};
use crate::errors::{NBError, NBResult};
use crate::event::{EventManager, Events};
use crate::receiver::FileReceiver;
use crate::registry::Registry;
use crate::sender::FileSender;
use crate::thread::{ShutdownSignal, ThreadContext, ThreadGroup};
use std::net::{UdpSocket};
use std::sync::mpsc::{self};
use std::sync::{Arc, Mutex};
use std::{env};

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
        let sender_socket_address = format!("{}:{}", DEFAULT_UDP_IP, DEFAULT_UDP_PORT);
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

    fn start_module(&mut self, mode: &str) -> NBResult<ThreadGroup> {
        let thread_group = match mode {
            "send" => Ok(self.sender.run(&self.socket)?),
            "receive" => Ok(self.receiver.run(&self.socket)?),
            _ => return Err(NBError::InvalidCommandLineArgs),
        };
        thread_group
    }

    fn parse_cmdline_arg() -> NBResult<String> {
        let args: Vec<String> = env::args().collect();
        if args.len() != 2 {
            return Err(NBError::InvalidCommandLineArgs);
        }
        Ok(args[1].clone())
    }
 
    pub fn run(&mut self) -> NBResult<()> {
        // validate the command line argument
        // usage should be netbeam <send|receive>
        let mode = Self::parse_cmdline_arg()?;
        let app_thread_group = self.start_module(mode.as_str())?;
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

pub struct AppState {
    pub is_running: bool,
    pub device_registry: Arc<Mutex<Registry<Device>>>,
    pub request_registry: Arc<Mutex<Registry<Request>>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            is_running: true,
            device_registry: Arc::new(Mutex::new(Registry::new())),
            request_registry: Arc::new(Mutex::new(Registry::new())),
        }
    }

    pub fn add_device(&mut self, request_id: RegistryId, device: Device) {
        let mut reg = self.device_registry.lock().unwrap();
        reg.add_entity(request_id, device);
    }

    pub fn remove_device(&mut self, request_id: RegistryId) {
        let mut reg = self.device_registry.lock().unwrap();
        reg.remove_entity(request_id);
    }

    pub fn add_request(&mut self, request_id: RegistryId, request: Request) {
        let mut reg = self.request_registry.lock().unwrap();
        reg.add_entity(request_id, request);
        
    }

    pub fn remove_request(&mut self, request_id: RegistryId) {
        let mut reg = self.request_registry.lock().unwrap();
        reg.remove_entity(request_id);
    }

    pub fn shutdown_app(&mut self) {
        self.is_running = false;
    }
}
