use std::net::SocketAddr;

pub type RegistryId = u64;

pub const DEFAULT_UDP_PORT: u16 = 11665;
pub const DEFAULT_TCP_PORT: u16 = 11666;
pub const DEFAULT_UDP_IP: &str = "0.0.0.0";
//pub const DEFAULT_MAX_RETRIES: u16 = 20;
pub const DEFAULT_RETRANSMIT_PERIOD: u64 = 5;
pub const SEND_MODE_IDENTIFIER: &'static str = "send";
pub const RECEIVE_MODE_IDENTIFIER: &'static str = "receive";

pub struct Request {
    pub request_id: RegistryId,
    pub port: u16,
    pub socket_address: SocketAddr,
}

impl Request {
    pub fn new(request_id: RegistryId, port: u16, socket_address: SocketAddr) -> Self {
        Request {
            request_id,
            port,
            socket_address,
        }
    }
}
