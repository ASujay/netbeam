use std::net::UdpSocket;
use crate::{
    errors::NBResult, packet::DiscoveryPacket::{self}, protocol::Protocol, thread::{ThreadContext, ThreadGroup}
};

pub struct FileSender {
    context: ThreadContext,
}

impl FileSender {
    pub fn new(context: ThreadContext, socket: &UdpSocket) -> NBResult<Self> {
        socket.set_broadcast(true)?;
        Ok(FileSender { context })
    }

    pub fn run(&self, udp_socket: &UdpSocket) -> NBResult<ThreadGroup> {
        let mut thread_group = ThreadGroup::new();

        let broadcast_thread_context = self.context.clone();
        let socket = udp_socket.try_clone()?;
        let conn_packet = DiscoveryPacket::Conn.encode();
        thread_group.spawn_thread(move || Protocol::broadcast(conn_packet, broadcast_thread_context, socket));

        let reply_listener_context = self.context.clone();
        let socket = udp_socket.try_clone()?;
        thread_group.spawn_thread(move || Protocol::reply_to_info(reply_listener_context, socket));

        // let ui_thread_context = self.context.clone();
        // thread_group.spawn_thread(move || {
        //     Ok(())
        // });
        Ok(thread_group)
    }
}