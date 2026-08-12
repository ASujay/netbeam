use crate::common::{DEFAULT_UDP_IP, DEFAULT_UDP_PORT, SOCKET_READ_TIMEOUT, SOCKET_WRITE_TIMEOUT};
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
        socket.set_read_timeout(Some(SOCKET_READ_TIMEOUT))?;
        socket.set_write_timeout(Some(SOCKET_WRITE_TIMEOUT))?;
        Ok(FileReceiver { context, socket })
    }

    pub fn run(&self) -> NBResult<ThreadGroup> {
        let mut thread_group = ThreadGroup::new();

        let reply_thread_context = self.context.clone();
        let reply_thread_socket = self.socket.try_clone()?;

        let retransmit_thread_context = self.context.clone();
        let retransmit_thread_socket = self.socket.try_clone()?;

        thread_group.spawn_thread(move || {
            protocol::reply_to_sender(reply_thread_context, reply_thread_socket)
        });
        thread_group.spawn_thread(move || {
            protocol::retransmit_to_sender(retransmit_thread_context, retransmit_thread_socket)
        });

        Ok(thread_group)
    }
}
