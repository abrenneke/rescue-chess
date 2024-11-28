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
    pub principal_variation: Option<Vec<PieceMove>>,
}

pub struct SearchState<'table> {
    pub nodes_searched: u32,
    pub cached_positions: u32,
    pub transposition_table: &'table mut TranspositionTable,
    pub rng: rand::rngs::ThreadRng,
    pub start_time: Instant,
    pub time_limit: u128,
    pub pruned: u32,
    pub best_move_so_far: Option<PieceMove>,
}

pub struct SearchStats {
    pub nodes_searched: u32,
    pub cached_positions: u32,
    pub pruned: u32,
    pub time_taken_ms: u128,
}

impl<'a> SearchState<'a> {
    pub fn to_stats(&self) -> SearchStats {
        SearchStats {
            nodes_searched: self.nodes_searched,
            cached_positions: self.cached_positions,
            pruned: self.pruned,
            time_taken_ms: self.start_time.elapsed().as_millis(),
        }
    }
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
            best_move_so_far: None,
        }
    }
}
