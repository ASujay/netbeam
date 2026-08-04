use std::sync::mpsc::Receiver;
use crossterm::event::Event;
use crate::{app::AppState, common::TransferReqId, device::Device};

pub enum Events {
    // keyboard event
    Key(Event),

    // Discovery Event
    DeviceFound{
        transfer_req_id: TransferReqId,
        device: Device,
    },
    DeviceLost(TransferReqId),
    Quit,
}

pub struct EventManager {
    receiver: Receiver<Events>,
}

impl EventManager {
    pub fn new(event_receiver: Receiver<Events>) -> Self {
        EventManager { receiver: event_receiver }
    }

    pub fn process_events(&self, state: &mut AppState) {
        while let Ok(event) = self.receiver.recv() {
            match event {
                Events::Key(key_event) => {},
                Events::DeviceFound{ transfer_req_id, device } => {
                    let mut reg = state.registry.lock().unwrap();
                    reg.add_device(transfer_req_id, device);
                },
                Events::DeviceLost(transfer_req_id) => {
                    let mut reg = state.registry.lock().unwrap();
                    reg.remove_device(transfer_req_id);
                },
                Events::Quit => {
                    state.is_running = false;
                },
            }
        }
    }
}
