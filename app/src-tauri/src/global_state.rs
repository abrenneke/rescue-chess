use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

use rescue_chess::{search::transposition_table::TranspositionTable, Position};

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
    pub depth: u32,
    pub transposition_table: Arc<Mutex<TranspositionTable>>,
}

impl Default for GlobalStateData {
    fn default() -> Self {
        GlobalStateData {
            position: Position::start_position(),
            depth: 5,
            transposition_table: Arc::new(Mutex::new(TranspositionTable::new())),
        }
    }
}

impl GlobalStateData {
    pub fn reset(&mut self) {
        self.position = Position::start_position();
    }
}
