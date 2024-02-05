use std::time::Instant;

use serde::Serialize;

use crate::PieceMove;

use super::transposition_table::TranspositionTable;

#[derive(Clone, Serialize)]
pub struct SearchResults {
    pub best_move: PieceMove,
    pub score: i32,
    pub nodes_searched: u32,
    pub cached_positions: u32,
    pub depth: u32,
    pub time_taken_ms: u128,
    pub pruned: u32,
    pub principal_variation: Vec<PieceMove>,
}

pub struct SearchState<'a> {
    pub nodes_searched: u32,
    pub cached_positions: u32,
    pub transposition_table: &'a mut TranspositionTable,
    pub start_time: Instant,
    pub time_limit: u128,
    pub pruned: u32,
}
