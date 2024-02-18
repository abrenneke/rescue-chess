use rand::seq::SliceRandom;

use crate::{evaluation::evaluate_position, PieceMove, Position};

use super::{
    search_results::{SearchResults, SearchState},
    transposition_table::TranspositionTableEntry,
};

#[derive(Clone)]
pub struct SearchResult {
    pub principal_variation: Vec<PieceMove>,
    pub score: i32,
}

impl std::ops::Neg for SearchResult {
    type Output = Self;

    fn neg(self) -> Self::Output {
        SearchResult {
            principal_variation: self.principal_variation,
            score: -self.score,
        }
    }
}

pub enum Error {
    Timeout,
}

pub fn search(position: &Position, depth: u32, state: &mut SearchState) -> SearchResults {
    let result = alpha_beta(position, -1000, 1000, depth, state);

    let time_taken_ms = state.start_time.elapsed().as_millis();

    match result {
        Ok(result) => {
            let best_move = result.principal_variation.first().cloned().unwrap();

            SearchResults {
                best_move,
                principal_variation: result.principal_variation,
                score: result.score,
                nodes_searched: state.nodes_searched,
                cached_positions: state.cached_positions,
                depth,
                time_taken_ms,
                pruned: state.pruned,
            }
        }
        Err(_) => panic!("Search timed out"),
    }
}

struct SearchIteration<'a: 'b, 'b> {
    alpha: i32,
    beta: i32,
    depth: u32,
    state: &'b mut SearchState<'a>,
    principal_variations: Vec<Vec<PieceMove>>,
}

impl<'a, 'b> std::fmt::Debug for SearchIteration<'a, 'b> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SearchIteration")
            .field("alpha", &self.alpha)
            .field("beta", &self.beta)
            .field("depth", &self.depth)
            .field("principal_variations", &self.principal_variations)
            .finish()
    }
}

pub fn alpha_beta(
    position: &Position,
    alpha: i32,
    beta: i32,
    depth: u32,
    state: &mut SearchState,
) -> Result<SearchResult, Error> {
    // If we have already searched this position to the same depth or greater,
    // we can use the cached result directly.
    if let Some(entry) = state.transposition_table.try_get(position, depth) {
        state.cached_positions += 1;
        return Ok(SearchResult {
            principal_variation: entry.principal_variation.clone(),
            score: entry.score,
        });
    }

    // If we have exceeded the time limit, we should return an error.
    if state.start_time.elapsed().as_millis() >= state.time_limit {
        return Err(Error::Timeout);
    }

    // Increment the total number of nodes searched.
    state.nodes_searched += 1;

    // If we have reached the maximum depth, we should evaluate the position
    // and return the result.
    if depth == 0 {
        let score = evaluate_position(position);
        return Ok(SearchResult {
            principal_variation: vec![],
            score,
        });
    }

    // If the position is a checkmate, we should return a very low score.
    // This is just to prevent the engine from continuing past a king capture.
    if position.is_checkmate().unwrap() {
        return Ok(SearchResult {
            principal_variation: vec![],
            score: -1000,
        });
    }

    let moves = position.get_all_legal_moves().unwrap();

    let mut iteration = SearchIteration {
        alpha,
        beta,
        depth,
        state,
        principal_variations: vec![],
    };

    let mut pass1 = vec![];
    let mut pass2 = vec![];

    for mv in moves.into_iter() {
        if mv.is_capture() {
            pass1.push(mv);
        } else {
            pass2.push(mv);
        }
    }

    // Iterate through all the legal moves and search the resulting positions.
    for mv in pass1 {
        if let Some(result) = test_move(mv, position, &mut iteration) {
            return result;
        }
    }

    for mv in pass2 {
        if let Some(result) = test_move(mv, position, &mut iteration) {
            return result;
        }
    }

    let picked_variation = iteration
        .principal_variations
        .choose(&mut iteration.state.rng)
        .expect("No principal variations found");

    iteration.state.transposition_table.insert(
        position.clone(),
        TranspositionTableEntry {
            depth,
            score: iteration.alpha,
            principal_variation: picked_variation.clone(),
        },
    );

    return Ok(SearchResult {
        principal_variation: picked_variation.clone(),
        score: iteration.alpha,
    });
}

fn test_move(
    mv: PieceMove,
    position: &Position,
    iteration: &mut SearchIteration,
) -> Option<Result<SearchResult, Error>> {
    // Apply the move to a clone of the position, then
    // switch to the other player's perspective.
    let mut child = position.clone();
    child.apply_move(mv).unwrap();
    child.invert();

    // Depth-first search the child position.
    let result = alpha_beta(
        &child,
        -iteration.beta,
        -iteration.alpha,
        iteration.depth - 1,
        iteration.state,
    );

    match result {
        Err(e) => Some(Err(e)),
        Ok(result) => {
            // Negate the score to switch back to the original player's perspective.
            let score = -result.score;

            // If the score is greater than or equal to beta, we can prune the search.
            if score >= iteration.beta {
                iteration.state.pruned += 1;

                return Some(Ok(SearchResult {
                    principal_variation: vec![mv],
                    score,
                }));
            }

            // If the score is greater than alpha, we have found a new best move.
            if score > iteration.alpha {
                iteration.alpha = score;

                let mut principal_variation = result.principal_variation.clone();
                principal_variation.insert(0, mv);

                iteration.principal_variations = vec![principal_variation];
            } else if score == iteration.alpha {
                let mut principal_variation = result.principal_variation.clone();
                principal_variation.insert(0, mv);

                iteration.principal_variations.push(principal_variation);
            } else if iteration.principal_variations.is_empty() {
                iteration.principal_variations.push(vec![mv]);
            }

            None
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{
        search::{
            alpha_beta::search, search_results::SearchState,
            transposition_table::TranspositionTable,
        },
        Position,
    };

    #[test]
    pub fn alpha_beta() {
        let position: Position = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR".into();

        let mut transposition_table = TranspositionTable::new();
        let mut state = SearchState::new(&mut transposition_table);

        let result = search(&position, 4, &mut state);

        println!("{}", result.best_move);
    }

    #[test]
    pub fn dumb_check() {
        let mut position: Position = "8/8/8/Rqpbkp2/8/8/8/K".into();
        position.invert();

        println!("{}", position.to_board_string());

        let mut transposition_table = TranspositionTable::new();
        let mut state = SearchState::new(&mut transposition_table);

        let result = search(&position, 4, &mut state);

        println!("{}", result.best_move);
    }
}
