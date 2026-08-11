use crate::common::{
    DEFAULT_UDP_IP, DEFAULT_UDP_PORT, RECEIVE_MODE_IDENTIFIER, RegistryId, Request,
    SEND_MODE_IDENTIFIER,
};
use crate::device::Device;
use crate::errors::{NBError, NBResult};
use crate::event::{EventManager, Events};
use crate::receiver::FileReceiver;
use crate::registry::Registry;
use crate::sender::FileSender;
use crate::thread::{ShutdownSignal, ThreadContext, ThreadGroup};
use std::env;
use std::net::UdpSocket;
use std::sync::mpsc::{self};
use std::sync::{Arc, Mutex};

enum Mode {
    Send,
    Receive,
}

impl Mode {
    fn from_cmd_args() -> NBResult<Self> {
        let args: Vec<String> = env::args().collect();
        if args.len() != 2 {
            return Err(NBError::InvalidCommandLineArgs);
        }
        match args[1].as_str() {
            SEND_MODE_IDENTIFIER => return Ok(Self::Send),
            RECEIVE_MODE_IDENTIFIER => return Ok(Self::Receive),
            _ => return Err(NBError::InvalidCommandLineArgs),
        }
    }
}

pub fn try_main() -> NBResult<()> {
    let mode = Mode::from_cmd_args()?;
    let mut app = App::new(mode)?;
    app.run()?;
    Ok(())
}

pub struct App {
    mode: Mode,
    sender: FileSender,
    receiver: FileReceiver,
    event_manager: EventManager,
    state: AppState,
    thread_context: ThreadContext,
    socket: UdpSocket,
}

impl App {
    pub fn new(mode: Mode) -> NBResult<Self> {
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
            mode,
        })
    }

    fn start_module(&mut self) -> NBResult<ThreadGroup> {
        let thread_group = match self.mode {
            Mode::Send => Ok(self.sender.run(&self.socket)?),
            Mode::Receive => Ok(self.receiver.run(&self.socket)?),
        };
        thread_group
    }

    pub fn run(&mut self) -> NBResult<()> {
        let app_thread_group = self.start_module()?;
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
