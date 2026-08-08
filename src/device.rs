use std::{net::IpAddr};

#[derive(Debug)]
pub struct Device {
    pub name: String,
    pub ip_address: IpAddr,
    pub port: u16,
}

impl Device {
    pub fn new(name: String, ip_address: IpAddr, port: u16) -> Device {
        Device { name, ip_address, port }
    }
}