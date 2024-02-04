use std::{ops::Deref, sync::Mutex};

use rescue_chess::Position;

pub struct GlobalState(Mutex<GlobalStateData>);

impl Default for GlobalState {
    fn default() -> Self {
        GlobalState(Mutex::new(GlobalStateData::default()))
    }
}

impl Deref for GlobalState {
    type Target = Mutex<GlobalStateData>;

    fn deref(&self) -> &Mutex<GlobalStateData> {
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
