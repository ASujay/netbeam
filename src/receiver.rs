use std::net::UdpSocket;
use crate::{errors::NBResult, protocol, thread::{ThreadContext, ThreadGroup}};

pub struct FileReceiver {
    context: ThreadContext,
}

impl FileReceiver {
    pub fn new(context: ThreadContext) -> NBResult<Self> {
        Ok(FileReceiver { context })
    }

    pub fn run(&self, udp_socket: &UdpSocket) -> NBResult<ThreadGroup> {
        let mut thread_group = ThreadGroup::new();
        let context = self.context.clone();
        let socket = udp_socket.try_clone()?;
        thread_group.spawn_thread(move || protocol::reply_to_sender(context, socket));
        
        let context = self.context.clone();
        let socket = udp_socket.try_clone()?;
        thread_group.spawn_thread(move || protocol::retransmit_to_sender(context, socket));
        Ok(thread_group)
    }
}
