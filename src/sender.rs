use crate::common::{DEFAULT_UDP_IP, DEFAULT_UDP_PORT};
use crate::{
    errors::NBResult,
    protocol,
    thread::{ThreadContext, ThreadGroup},
};
use std::net::UdpSocket;

pub struct FileSender {
    context: ThreadContext,
    socket: UdpSocket,
}

impl FileSender {
    pub fn new(context: ThreadContext) -> NBResult<Self> {
        let discovery_socket_address = format!("{}:{}", DEFAULT_UDP_IP, DEFAULT_UDP_PORT);
        let socket = UdpSocket::bind(discovery_socket_address)?;
        socket.set_broadcast(true)?;
        Ok(FileSender { context, socket })
    }

    pub fn run(&self) -> NBResult<ThreadGroup> {
        let mut thread_group = ThreadGroup::new();

        let broadcast_thread_context = self.context.clone();
        let broadcast_socket = self.socket.try_clone()?;

        let reply_listener_context = self.context.clone();
        let reply_listener_socket = self.socket.try_clone()?;

        thread_group
            .spawn_thread(move || protocol::broadcast(broadcast_thread_context, broadcast_socket));
        thread_group.spawn_thread(move || {
            protocol::reply_to_info(reply_listener_context, reply_listener_socket)
        });
        // let ui_thread_context = self.context.clone();
        // thread_group.spawn_thread(move || {
        //     Ok(())
        // });
        Ok(thread_group)
    }
}
