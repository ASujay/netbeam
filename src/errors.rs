use std::io;

pub type NBResult<T> = Result<T, NBError>;

#[derive(Debug)]
pub enum NBError {
    InvalidCommandLineArgs,
    Io(std::io::Error),
    NetworkInterface(network_interface::Error),
}

impl NBError {
    pub fn handle_error(&self) {
        match self {
            NBError::InvalidCommandLineArgs => {
                eprintln!("Invalid command line arguments. Usage: netbeam <send|receive>");
            },
            NBError::Io(err) => {
                eprintln!("IO error: {}", err);
            },
            NBError::NetworkInterface(err) => {
                eprintln!("Network interface error: {}", err);
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



