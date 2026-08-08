use std::collections::{HashMap};
use crate::common::RegistryId;

pub struct Registry<T>(pub HashMap<RegistryId, T>);

impl<T> Registry<T> {
    pub fn new() -> Self {
        Registry(HashMap::new())
    }

    pub fn add_entity(&mut self, request_id: RegistryId, entity: T) {
        self.0.insert(request_id, entity);
    }

    pub fn remove_entity(&mut self, request_id: RegistryId) {
        self.0.remove(&request_id);
    }
}