use crate::{
    app::AppState,
    common::{RegistryId, Request},
    device::Device,
    packet::DiscoveryPacket,
};
use crossterm::event::Event;
use std::{
    io,
    net::{SocketAddr, UdpSocket},
    sync::mpsc::Receiver,
};

pub enum Events {
    // keyboard event
    Key(Event),
    BroadcastError {
        destination: SocketAddr,
        error: io::Error,
    },
    // Discovery Event
    DeviceFound {
        request_id: RegistryId,
        device: Device,
    },
    DeviceLost(RegistryId),
    AddRequest {
        request_id: RegistryId,
        request: Request,
    },
    RemoveRequest(RegistryId),
    RetransmitPendingPackets(UdpSocket),
}

pub struct EventManager {
    receiver: Receiver<Events>,
}

impl EventManager {
    pub fn new(event_receiver: Receiver<Events>) -> Self {
        EventManager {
            receiver: event_receiver,
        }
    }

    pub fn process_key_events(&self, state: &mut AppState, key_event: Event) {
        _ = state;
        _ = key_event;
    }

    pub fn process_events(&self, state: &mut AppState) {
        while let Ok(event) = self.receiver.recv() {
            match event {
                Events::Key(key_event) => self.process_key_events(state, key_event),
                Events::DeviceFound { request_id, device } => state.add_device(request_id, device),
                Events::DeviceLost(request_id) => state.remove_device(request_id),
                Events::AddRequest {
                    request_id,
                    request,
                } => state.add_request(request_id, request),
                Events::RemoveRequest(request_id) => state.remove_request(request_id),
                Events::RetransmitPendingPackets(socket) => {
                    let reg = state.request_registry.lock().unwrap();
                    for (_, value) in reg.0.iter() {
                        let packet = DiscoveryPacket::Info {
                            port: value.port,
                            request_id: value.request_id,
                        }
                        .encode();
                        if let Err(e) = socket.send_to(packet.as_slice(), value.socket_address) {}
                    }
                }
                Events::BroadcastError { destination, error } => {
                    eprintln!("Broadcast to {destination} failed: {error}");
                }
            }
        }
    }
}
