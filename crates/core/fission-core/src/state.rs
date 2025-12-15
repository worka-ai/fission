use crate::action::AppState;
use std::collections::HashMap;
use std::any::TypeId;

#[derive(Default)]
pub struct StateMap {
    pub states: HashMap<TypeId, Box<dyn AppState>>,
}