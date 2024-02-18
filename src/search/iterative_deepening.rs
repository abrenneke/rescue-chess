use std::time::Instant;

use crate::{PieceMove, Position};

use super::{
    alpha_beta,
    search_results::{SearchResults, SearchState},
    transposition_table::TranspositionTable,
};

pub struct IterativeDeepeningData {
    current_position: Position,
    transposition_table: TranspositionTable,
    best_move: Option<PieceMove>,
}

impl IterativeDeepeningData {
    pub fn new() -> Self {
        Self {
            current_position: Position::start_position(),
            transposition_table: TranspositionTable::new(),
            best_move: None,
        }
    }

    pub fn update_position(&mut self, position: Position) {
        self.current_position = position;
    }

    pub fn search(&mut self, time_limit: u128) {
        let mut depth = 1;
        let start_time = Instant::now();

        loop {
            let elapsed = start_time.elapsed().as_millis();
            if elapsed >= time_limit {
                break;
            }

            let search_results = self.search_at_depth(depth, start_time, time_limit);

            println!(
                "Depth: {} Score: {} Nodes: {} Cached: {} Time: {} Best Move: {} Pruned: {}, Principal Variation: {:?}",
                depth,
                search_results.score,
                search_results.nodes_searched,
                search_results.cached_positions,
                search_results.time_taken_ms,
                search_results.best_move,
                search_results.pruned,
                search_results.principal_variation
            );

            self.best_move = Some(search_results.best_move);

            depth += 1;
        }
    }

    fn search_at_depth(
        &mut self,
        depth: u32,
        start_time: Instant,
        time_limit: u128,
    ) -> SearchResults {
        let mut state = SearchState::new(&mut self.transposition_table);
        state.start_time = start_time;
        state.time_limit = time_limit;

        alpha_beta::search(&self.current_position, depth, &mut state)
    }

    pub fn get_best_move(&self) -> Option<PieceMove> {
        self.best_move
    }
}

#[cfg(test)]
pub mod tests {
    use super::IterativeDeepeningData;

    #[test]
    pub fn iterative_deepening_1() {
        let mut data = IterativeDeepeningData::new();

        data.update_position("2K5/7p/RPp5/1rPP4/1b4p1/PbN5/3k4/2q4Q w - - 0 1".into());
        data.search(1000000);

        println!("{}", data.get_best_move().unwrap());
    }
}
