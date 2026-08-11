use crate::common::{RECEIVE_MODE_IDENTIFIER, RegistryId, Request, SEND_MODE_IDENTIFIER};
use crate::device::Device;
use crate::errors::{NBError, NBResult};
use crate::event::{EventManager, Events};
use crate::receiver::FileReceiver;
use crate::registry::Registry;
use crate::sender::FileSender;
use crate::thread::{ShutdownSignal, ThreadContext, ThreadGroup};
use std::env;
use std::sync::mpsc::{self};
use std::sync::{Arc, Mutex};

enum Mode {
    Send,
    Receive,
}

enum AppModule {
    Sender(FileSender),
    Receiver(FileReceiver),
}

impl Mode {
    fn from_cmd_args() -> NBResult<Self> {
        let args: Vec<String> = env::args().collect();
        if args.len() != 2 {
            return Err(NBError::InvalidCommandLineArgs);
        }
        match args[1].as_str() {
            SEND_MODE_IDENTIFIER => Ok(Self::Send),
            RECEIVE_MODE_IDENTIFIER => Ok(Self::Receive),
            _ => Err(NBError::InvalidCommandLineArgs),
        }
    }
}

pub(super) fn try_main() -> NBResult<()> {
    let mode = Mode::from_cmd_args()?;
    let mut app = App::new(mode)?;
    app.run()
}

struct App {
    event_manager: EventManager,
    state: AppState,
    thread_context: ThreadContext,
    module: AppModule,
}

impl App {
    fn new(mode: Mode) -> NBResult<Self> {
        let (event_sender, event_sreceiver) = mpsc::channel::<Events>();
        let shutdown = ShutdownSignal::new();
        let thread_context = ThreadContext::new(shutdown, event_sender);

        // we propogate the error because if we cannot initalize this struct the app wont runt, so this is an irrecoverable failure
        let module = match mode {
            Mode::Send => AppModule::Sender(FileSender::new(thread_context.clone())?),
            Mode::Receive => AppModule::Receiver(FileReceiver::new(thread_context.clone())?),
        };
        let event_manager = EventManager::new(event_sreceiver);
        let state = AppState::new();
        Ok(App {
            event_manager,
            state,
            thread_context,
            module,
        })
    }

    fn start_module(&self) -> NBResult<ThreadGroup> {
        match &self.module {
            AppModule::Sender(sender) => sender.run(),
            AppModule::Receiver(receiver) => receiver.run(),
        }
    }

    fn run(&mut self) -> NBResult<()> {
        let app_thread_group = self.start_module()?;
        while self.state.is_running {
            self.process_events();
        }
        self.thread_context.shutdown();
        _ = app_thread_group.join_all();
        Ok(())
    }

    fn process_events(&mut self) {
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
