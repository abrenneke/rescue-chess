use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

use rescue_chess::Position;

pub struct GlobalState(pub Arc<Mutex<GlobalStateData>>);

impl Default for GlobalState {
    fn default() -> Self {
        GlobalState(Arc::new(Mutex::new(GlobalStateData::default())))
    }
}

impl Deref for GlobalState {
    type Target = Arc<Mutex<GlobalStateData>>;

    fn deref(&self) -> &Arc<Mutex<GlobalStateData>> {
        &self.0
    }
}

pub struct GlobalStateData {
    pub position: Position,
}

impl Default for GlobalStateData {
    fn default() -> Self {
        GlobalStateData {
            position: Position::start_position(),
        }
    }
}

impl GlobalStateData {
    pub fn reset(&mut self) {
        self.position = Position::start_position();
    }
}
