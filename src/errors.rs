use crate::event::Events;
use std::{
    eprintln, io,
    sync::mpsc::{RecvError, SendError},
};

pub type NBResult<T> = Result<T, NBError>;

#[derive(Debug)]
pub enum NBError {
    InvalidCommandLineArgs,
    Io(std::io::Error),
    NetworkInterface(network_interface::Error),
    EventRegister(SendError<Events>),
    EventChannelDisconnection(RecvError),
}

impl NBError {
    pub fn handle_error(&self) {
        match self {
            NBError::InvalidCommandLineArgs => {
                eprintln!("Invalid command line arguments. Usage: netbeam <send|receive>");
            }
            NBError::Io(err) => {
                eprintln!("IO error: {}", err);
            }
            NBError::NetworkInterface(err) => {
                eprintln!("Network interface error: {}", err);
            }
            NBError::EventRegister(err) => {
                eprintln!("Unable to register event: {}", err);
            }
            NBError::EventChannelDisconnection(err) => {
                eprintln!("Event channel was disconnected: {}", err);
            }
        }
    }
}

impl From<io::Error> for NBError {
    fn from(err: io::Error) -> Self {
        NBError::Io(err)
    }
}

impl From<network_interface::Error> for NBError {
    fn from(err: network_interface::Error) -> Self {
        NBError::NetworkInterface(err)
    }
}

impl From<SendError<Events>> for NBError {
    fn from(err: SendError<Events>) -> Self {
        NBError::EventRegister(err)
    }
}

impl From<RecvError> for NBError {
    fn from(value: RecvError) -> Self {
        NBError::EventChannelDisconnection(value)
    }
}
