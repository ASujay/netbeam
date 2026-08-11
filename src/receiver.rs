use crate::common::{DEFAULT_UDP_IP, DEFAULT_UDP_PORT};
use crate::{
    errors::NBResult,
    protocol,
    thread::{ThreadContext, ThreadGroup},
};
use std::net::UdpSocket;

pub struct FileReceiver {
    context: ThreadContext,
    socket: UdpSocket,
}

impl FileReceiver {
    pub fn new(context: ThreadContext) -> NBResult<Self> {
        let discovery_socket_address = format!("{}:{}", DEFAULT_UDP_IP, DEFAULT_UDP_PORT);
        let socket = UdpSocket::bind(discovery_socket_address)?;
        Ok(FileReceiver { context, socket })
    }

    pub fn run(&self) -> NBResult<ThreadGroup> {
        let mut thread_group = ThreadGroup::new();
        let context = self.context.clone();
        let socket = self.socket.try_clone()?;
        thread_group.spawn_thread(move || protocol::reply_to_sender(context, socket));

        let context = self.context.clone();
        let socket = self.socket.try_clone()?;
        thread_group.spawn_thread(move || protocol::retransmit_to_sender(context, socket));
        Ok(thread_group)
    }
}
