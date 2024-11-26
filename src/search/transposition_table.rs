use fxhash::FxHashMap;

use crate::{PieceMove, Position};

/// A transposition table that stores positions and their scores and depths.
///
/// This table is used to store the results of previous searches so that they
/// can be reused in future searches.
#[derive(Clone, Debug)]
pub struct TranspositionTable {
    table: FxHashMap<String, TranspositionTableEntry>,
}

#[derive(Clone, Debug)]
pub struct TranspositionTableEntry {
    pub score: i32,
    pub depth: u32,
    pub principal_variation: PieceMove,
}

impl TranspositionTable {
    /// Creates a new transposition table with an empty hash map.
    pub fn new() -> Self {
        Self {
            table: FxHashMap::default(),
        }
    }

    /// Gets the score and depth of a position from the table.
    pub fn get(&self, position: &Position) -> Option<TranspositionTableEntry> {
        self.table.get(&position.to_fen()).cloned()
    }

    /// Tries to get the score of a position from the table. If the depth of the
    /// stored score is greater than or equal to the given depth, the score is
    /// returned. Otherwise, `None` is returned.
    pub fn try_get(&self, position: &Position, depth: u32) -> Option<&TranspositionTableEntry> {
        if let Some(entry) = self.table.get(&position.to_fen()) {
            if entry.depth >= depth {
                return Some(entry);
            }
        }

        None
    }

    /// Inserts a position into the table with the given score and depth.
    pub fn insert(&mut self, position: Position, entry: TranspositionTableEntry) {
        self.table.insert(position.to_fen(), entry);
    }

    pub fn insert_if_better(&mut self, position: Position, entry: TranspositionTableEntry) {
        let fen = position.to_fen();
        if let Some(existing_entry) = self.table.get(&fen) {
            if entry.depth > existing_entry.depth {
                return;
            }
        }

        self.table.insert(fen, entry);
    }

    pub fn clear(&mut self) {
        self.table.clear();
    }

    pub fn principal_variation_list(&self, position: &Position) -> Vec<PieceMove> {
        let mut moves = Vec::new();
        let mut current_position = position.clone();

        while let Some(entry) = self.table.get(&current_position.to_fen()) {
            moves.push(entry.principal_variation);
            current_position
                .apply_move(entry.principal_variation)
                .unwrap();
        }

        moves
    }
}
