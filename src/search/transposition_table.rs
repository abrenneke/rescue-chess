use fxhash::FxHashMap;

use crate::{position::HashablePosition, PieceMove};

/// A transposition table that stores positions and their scores and depths.
///
/// This table is used to store the results of previous searches so that they
/// can be reused in future searches.
#[derive(Clone, Debug)]
pub struct TranspositionTable {
    table: FxHashMap<HashablePosition, TranspositionTableEntry>,
}

#[derive(Clone, Debug)]
pub struct TranspositionTableEntry {
    pub score: i32,
    pub alpha: i32,
    pub beta: i32,
    pub depth: u32,
    pub principal_variation: Vec<PieceMove>,
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
    pub fn get(&self, position: &HashablePosition) -> Option<TranspositionTableEntry> {
        self.table.get(&position).cloned()
    }

    /// Tries to get the score of a position from the table. If the depth of the
    /// stored score is greater than or equal to the given depth, the score is
    /// returned. Otherwise, `None` is returned.
    pub fn try_get(
        &self,
        position: &HashablePosition,
        depth: u32,
        alpha: i32,
        beta: i32,
    ) -> Option<&TranspositionTableEntry> {
        if let Some(entry) = self.table.get(&position) {
            if entry.depth >= depth {
                match entry.node_type {
                    // For exact scores, just check if score is within current window
                    NodeType::Exact if entry.score > alpha && entry.score < beta => Some(entry),
                    // For lower bounds, need current beta >= stored beta
                    NodeType::LowerBound if beta >= entry.beta && entry.score >= beta => {
                        Some(entry)
                    }
                    // For upper bounds, need current alpha <= stored alpha
                    NodeType::UpperBound if alpha <= entry.alpha && entry.score <= alpha => {
                        Some(entry)
                    }
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
    pub fn insert(&mut self, position: HashablePosition, entry: TranspositionTableEntry) {
        self.table.insert(position, entry);
    }

    pub fn insert_if_better(&mut self, position: HashablePosition, entry: TranspositionTableEntry) {
        if let Some(existing_entry) = self.table.get(&position) {
            if entry.depth > existing_entry.depth {
                self.table.insert(position, entry);
            } else if entry.depth == existing_entry.depth && entry.node_type == NodeType::Exact {
                self.table.insert(position, entry);
            }
        } else {
            self.table.insert(position, entry);
        }
    }

    pub fn clear(&mut self) {
        self.table.clear();
    }
}
