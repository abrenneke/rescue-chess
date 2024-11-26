use std::time::Instant;

use serde::Serialize;

use crate::PieceMove;

use super::transposition_table::TranspositionTable;

#[derive(Clone, Serialize)]
pub struct SearchResults {
    pub best_move: Option<PieceMove>,
    pub score: i32,
    pub nodes_searched: u32,
    pub cached_positions: u32,
    pub depth: u32,
    pub time_taken_ms: u128,
    pub pruned: u32,
    pub principal_variation: Vec<PieceMove>,
}

pub struct SearchState<'table> {
    pub nodes_searched: u32,
    pub cached_positions: u32,
    pub transposition_table: &'table mut TranspositionTable,
    pub rng: rand::rngs::ThreadRng,
    pub start_time: Instant,
    pub time_limit: u128,
    pub pruned: u32,
}

impl<'a> SearchState<'a> {
    pub fn new(transposition_table: &'a mut TranspositionTable) -> Self {
        Self {
            nodes_searched: 0,
            cached_positions: 0,
            transposition_table,
            start_time: Instant::now(),
            rng: rand::thread_rng(),
            time_limit: u128::MAX,
            pruned: 0,
        }
    }
}
