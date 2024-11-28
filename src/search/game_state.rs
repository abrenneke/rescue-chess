use std::collections::HashMap;

use crate::{piece_move::GameType, Color, PieceMove, Position};

use super::{
    alpha_beta::{self, SearchParams},
    search_results::{SearchResults, SearchState, SearchStats},
    transposition_table::TranspositionTable,
};

pub struct GameState {
    /// A map from positions to the number of times that position has been visited.
    pub positions: HashMap<Position, usize>,

    /// The current position.
    pub current_position: Position,

    /// The number of single moves that have been made.
    pub num_plies: usize,

    /// The number of two-move pairs that have been made.
    pub move_number: usize,

    /// Whose turn it is. If black, the current_position is inverted.
    pub current_turn: Color,

    /// Previous scores for white and black respectively
    pub previous_scores: (Option<i32>, Option<i32>),

    /// The depth to search to.
    pub search_depth: u32,

    pub transposition_table: TranspositionTable,

    pub game_type: GameType,

    pub debug_logs_1: bool,
}

impl GameState {
    pub fn new() -> Self {
        let mut state = Self {
            positions: HashMap::new(),
            current_position: Position::start_position(),
            num_plies: 0,
            current_turn: Color::White,
            move_number: 1,
            previous_scores: (None, None),
            search_depth: 4,
            transposition_table: TranspositionTable::new(),
            game_type: GameType::Classic,
            debug_logs_1: false,
        };

        state.positions.insert(state.current_position.clone(), 1);

        state
    }

    pub fn from_position(position: Position) -> Self {
        let mut state = Self {
            current_position: position,
            ..Default::default()
        };

        state.positions.insert(state.current_position.clone(), 1);

        state
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    pub fn apply_move(&mut self, mv: PieceMove) -> Result<(), anyhow::Error> {
        self.current_position.apply_move(mv)?;
        self.current_position.invert();

        *self
            .positions
            .entry(self.current_position.clone())
            .or_insert(0) += 1;

        self.num_plies += 1;

        self.current_turn = self.current_turn.invert();

        if self.current_turn == Color::White {
            self.move_number += 1;
        }

        Ok(())
    }

    pub fn times_current_position_seen(&self) -> usize {
        *self.positions.get(&self.current_position).unwrap_or(&0)
    }

    pub fn previous_score(&self, color: Color) -> Option<i32> {
        match color {
            Color::White => self.previous_scores.0,
            Color::Black => self.previous_scores.1,
        }
    }

    pub fn update_previous_score(&mut self, color: Color, score: i32) {
        match color {
            Color::White => self.previous_scores.0 = Some(score),
            Color::Black => self.previous_scores.1 = Some(score),
        }
    }

    pub fn search_and_apply(&mut self) -> Result<(SearchResults, SearchStats), anyhow::Error> {
        let params = SearchParams {
            depth: self.search_depth,
            game_type: self.game_type,
            previous_score: self.previous_score(self.current_turn),
            debug_print: true,
            ..Default::default()
        };

        let mut state = SearchState::new(&mut self.transposition_table);
        let result = alpha_beta::search(&self.current_position, &mut state, params);
        let stats = state.to_stats();

        self.update_previous_score(self.current_turn, result.score);

        if let Some(best_move) = result.best_move {
            self.apply_move(best_move)?;
        } else {
            return Err(anyhow::anyhow!("No best move found"));
        }

        if self.positions[&self.current_position] > 1 {
            if self.debug_logs_1 {
                println!(
                    "Position has been seen > 1 time, increasing depth to {}",
                    self.search_depth + 1,
                );
            }
            self.search_depth += 1;
        }

        Ok((result, stats))
    }
}
