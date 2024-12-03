use std::time::Instant;

use tracing::trace;

use crate::{PieceMove, Position};

use super::{
    alpha_beta::{self, SearchParams},
    search_results::{SearchResults, SearchState, SearchStats},
    transposition_table::TranspositionTable,
};

pub struct IterativeDeepeningData {
    pub current_position: Position,
    pub transposition_table: TranspositionTable,
    pub stats: SearchStats,
    pub best_move: Option<PieceMove>,
    pub best_score: Option<i32>,
    pub previous_pv: Option<PieceMove>,
}

impl IterativeDeepeningData {
    pub fn new() -> Self {
        Self {
            current_position: Position::start_position(),
            transposition_table: TranspositionTable::new(),
            stats: SearchStats::default(),
            best_move: None,
            best_score: None,
            previous_pv: None,
        }
    }

    pub fn update_position(&mut self, position: Position) {
        self.current_position = position;
    }

    pub fn search(&mut self, params: SearchParams) {
        let mut depth = 1;
        let start_time = Instant::now();

        loop {
            if depth > params.depth {
                break;
            }

            let elapsed = start_time.elapsed().as_millis();
            if elapsed >= params.time_limit as u128 {
                break;
            }

            let search_results = self.search_at_depth(depth, start_time, &params);

            match search_results {
                Ok(search_results) => {
                    if params.debug_print_verbose {
                        trace!(
                            "Depth: {} Score: {} Nodes: {} Cached: {} Time: {} Best Move: {} Pruned: {}, Principal Variation: {:?}",
                            depth,
                            search_results.score,
                            search_results.nodes_searched,
                            search_results.cached_positions,
                            search_results.time_taken_ms,
                            search_results.best_move.unwrap(),
                            search_results.pruned,
                            search_results.principal_variation
                        );
                    }

                    self.best_move = search_results.best_move;
                    self.best_score = Some(search_results.score);
                    self.previous_pv = search_results.best_move;

                    depth += 1;
                }
                Err(e) => match e {
                    alpha_beta::Error::Timeout => {
                        break;
                    }
                },
            }
        }
    }

    fn search_at_depth(
        &mut self,
        depth: u32,
        start_time: Instant,
        params_base: &SearchParams,
    ) -> Result<SearchResults, alpha_beta::Error> {
        let mut state = SearchState::new(&mut self.transposition_table);
        state.data.start_time = start_time;
        state.data.time_limit = params_base.time_limit;
        state.data.previous_pv = self.previous_pv;

        let mut params = params_base.clone();
        params.depth = depth;

        let results = alpha_beta::search(&self.current_position, &mut state, params);

        self.stats.add(state.to_stats());

        results
    }

    pub fn get_best_move(&self) -> Option<PieceMove> {
        self.best_move
    }
}

#[cfg(test)]
pub mod tests {
    use crate::search::alpha_beta::SearchParams;

    use super::IterativeDeepeningData;

    #[test]
    pub fn iterative_deepening_1() {
        let mut data = IterativeDeepeningData::new();

        data.update_position("2K5/7p/RPp5/1rPP4/1b4p1/PbN5/3k4/2q4Q w - - 0 1".into());
        data.search(SearchParams {
            depth: 5,
            time_limit: 1000,
            ..Default::default()
        });

        println!("{}", data.get_best_move().unwrap());
    }
}
