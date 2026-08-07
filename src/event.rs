use std::sync::mpsc::Receiver;
use crossterm::event::Event;
use crate::{app::AppState, common::TransferReqId, device::Device};

pub enum Events {
    // keyboard event
    Key(Event),

    // Discovery Event
    DeviceFound{
        request_id: TransferReqId,
        device: Device,
    },
    DeviceLost(TransferReqId),
}

pub struct EventManager {
    receiver: Receiver<Events>,
}

impl EventManager {
    pub fn new(event_receiver: Receiver<Events>) -> Self {
        EventManager { receiver: event_receiver }
    }

    pub fn process_key_events(&self, state: &mut AppState, key_event: Event) {
        _ = state;
        _ = key_event;
    }

    pub fn process_events(&self, state: &mut AppState) {
        while let Ok(event) = self.receiver.recv() {
            match event {
                Events::Key(key_event) => self.process_key_events(state, key_event),
                Events::DeviceFound{ request_id, device } => state.add_device(request_id, device),
                Events::DeviceLost(request_id) => state.remove_device(request_id),
            }
        }
    }
}
