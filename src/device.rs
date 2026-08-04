use std::collections::HashMap;

use crate::common::TransferReqId;

pub struct DeviceRegistry(HashMap<TransferReqId, Device>);

impl DeviceRegistry {
    pub fn new() -> Self {
        DeviceRegistry(HashMap::new())
    }

    pub fn add_device(&mut self, request_id: TransferReqId, device: Device) {
        self.0.insert(request_id, device);
    }

    pub fn remove_device(&mut self, request_id: TransferReqId) {
        self.0.remove(&request_id);
    }
}

pub struct Device {
    pub name: String,
    pub ip_address: String,
    pub port: u16,
}

impl Device {}