use fxhash::FxHashMap;

use crate::{PieceMove, Position};

/// A transposition table that stores positions and their scores and depths.
///
/// This table is used to store the results of previous searches so that they
/// can be reused in future searches.
#[derive(Clone, Debug)]
pub struct TranspositionTable {
    table: FxHashMap<Position, TranspositionTableEntry>,
}

#[derive(Clone, Debug)]
pub struct TranspositionTableEntry {
    pub score: i32,
    pub depth: u32,
    pub principal_variation: Option<PieceMove>,
    pub node_type: NodeType,
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum NodeType {
    Exact,
    LowerBound,
    UpperBound,
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
        self.table.get(&position).cloned()
    }

    /// Tries to get the score of a position from the table. If the depth of the
    /// stored score is greater than or equal to the given depth, the score is
    /// returned. Otherwise, `None` is returned.
    pub fn try_get(
        &self,
        position: &Position,
        depth: u32,
        alpha: i32,
        beta: i32,
    ) -> Option<&TranspositionTableEntry> {
        if let Some(entry) = self.table.get(&position) {
            if entry.depth >= depth {
                match entry.node_type {
                    NodeType::Exact => Some(entry),
                    NodeType::LowerBound if entry.score >= beta => Some(entry),
                    NodeType::UpperBound if entry.score <= alpha => Some(entry),
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Inserts a position into the table with the given score and depth.
    pub fn insert(&mut self, position: Position, entry: TranspositionTableEntry) {
        self.table.insert(position, entry);
    }

    pub fn insert_if_better(&mut self, position: Position, entry: TranspositionTableEntry) {
        if let Some(existing_entry) = self.table.get(&position) {
            if entry.depth > existing_entry.depth {
                return;
            }
        }

        self.table.insert(position, entry);
    }

    pub fn clear(&mut self) {
        self.table.clear();
    }

    pub fn principal_variation_list(&self, position: &Position, mut depth: u32) -> Vec<PieceMove> {
        let mut moves = Vec::new();
        let mut current_position = position.clone();

        while depth > 0 {
            // Only follow exact nodes for PV
            if let Some(entry) = self.table.get(&current_position) {
                if entry.depth >= depth && entry.node_type == NodeType::Exact {
                    if let Some(mv) = entry.principal_variation {
                        moves.push(mv);
                        current_position.apply_move(mv).unwrap();
                        depth -= 1;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        moves
    }
}
