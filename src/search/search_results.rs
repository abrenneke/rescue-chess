use std::time::Instant;

use serde::Serialize;

use crate::PieceMove;

use super::{
    history::HistoryTable, iterative_deepening::OnNewBestMove, killer_moves::KillerMoves,
    transposition_table::TranspositionTable,
};

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
    pub alpha: i32,
    pub beta: i32,
}

pub struct SearchState<'table, 'a> {
    pub data: SearchStateData,
    pub transposition_table: &'table mut TranspositionTable,
    pub callbacks: SearchStateCallbacks<'a>,
    pub killer_moves: KillerMoves,
    pub history: HistoryTable,
}

pub struct SearchStateData {
    pub nodes_searched: u32,
    pub cached_positions: u32,
    pub rng: rand::rngs::ThreadRng,
    pub start_time: Instant,
    pub time_limit: u64,
    pub pruned: u32,
    pub best_move_so_far: Option<PieceMove>,
    pub previous_pv: Option<Vec<PieceMove>>,
}

pub struct SearchStateCallbacks<'a> {
    pub on_new_best_move: Option<&'a OnNewBestMove>,
}

#[derive(Debug, Clone)]
pub struct SearchStats {
    pub nodes_searched: u32,
    pub cached_positions: u32,
    pub pruned: u32,
    pub time_taken_ms: u128,
}

impl Default for SearchStats {
    fn default() -> Self {
        Self {
            nodes_searched: 0,
            cached_positions: 0,
            pruned: 0,
            time_taken_ms: 0,
        }
    }
}

impl SearchStats {
    pub fn add(&mut self, stats: SearchStats) {
        self.nodes_searched += stats.nodes_searched;
        self.cached_positions += stats.cached_positions;
        self.pruned += stats.pruned;
        self.time_taken_ms = stats.time_taken_ms;
    }
}

impl<'table, 'a> SearchState<'table, 'a> {
    pub fn to_stats(&self) -> SearchStats {
        SearchStats {
            nodes_searched: self.data.nodes_searched,
            cached_positions: self.data.cached_positions,
            pruned: self.data.pruned,
            time_taken_ms: self.data.start_time.elapsed().as_millis(),
        }
    }
}

impl<'table, 'a> SearchState<'table, 'a> {
    pub fn new(transposition_table: &'table mut TranspositionTable) -> Self {
        Self {
            data: SearchStateData {
                nodes_searched: 0,
                cached_positions: 0,
                start_time: Instant::now(),
                rng: rand::thread_rng(),
                time_limit: u64::MAX,
                pruned: 0,
                best_move_so_far: None,
                previous_pv: None,
            },
            transposition_table,
            callbacks: SearchStateCallbacks {
                on_new_best_move: None,
            },
            killer_moves: KillerMoves::new(64),
            history: HistoryTable::new(),
        }
    }
}
