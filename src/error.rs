use std::{io, net::{self, AddrParseError}};

#[derive(Debug)]
pub enum NetbeamError {
    IoError(io::Error),
    BroadcastIp(local_ip_address::Error),
    AddrParseError(net::AddrParseError),
}

impl From<io::Error> for NetbeamError {
    fn from(value: io::Error) -> Self {
       NetbeamError::IoError(value) 
    }
}

impl From<local_ip_address::Error> for NetbeamError {
    fn from(value: local_ip_address::Error) -> Self {
        NetbeamError::BroadcastIp(value)
    }
}

impl From<net::AddrParseError> for NetbeamError {
    fn from(value: net::AddrParseError) -> Self {
        NetbeamError::AddrParseError(value)
    }
}

pub type NetbeamResult<T> = std::result::Result<T, NetbeamError>;