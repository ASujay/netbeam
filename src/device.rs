use core::fmt;
use std::{net::IpAddr, write};

pub struct Device {
    pub name: String,
    pub ip_address: IpAddr,
    pub port: u16,
}

impl Device {
    pub fn new(name: String, ip_address: IpAddr, port: u16) -> Device {
        Device {
            name,
            ip_address,
            port,
        }
    }
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}:{}",
            self.name,
            self.ip_address.to_string(),
            self.port
        )
    }
}
