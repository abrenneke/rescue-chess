use rand::seq::SliceRandom;
use tracing::{debug, info, trace};

use crate::{
    evaluation::evaluate_position,
    variation::{Variation, Variations},
    Color, PieceMove, Position,
};

use super::{
    search_results::{SearchResults, SearchState},
    transposition_table::TranspositionTableEntry,
};

#[derive(Clone)]
pub struct SearchResult {
    pub principal_variation: Variation,
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
    let mut move_stack = vec![];
    let result = alpha_beta(
        position,
        -1000,
        1000,
        depth,
        Color::White,
        state,
        &mut move_stack,
    );

    let time_taken_ms = state.start_time.elapsed().as_millis();

    match result {
        AlphaBetaResult::SearchResult(result) => {
            let best_move = result.principal_variation.next_move();

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
        AlphaBetaResult::Error(_) => panic!("Search timed out"),
    }
}

struct SearchIteration<'a: 'b, 'b> {
    alpha: i32,
    beta: i32,
    depth: u32,
    state: &'b mut SearchState<'a>,
    principal_variations: Variations,
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

pub enum AlphaBetaResult {
    /// There was an error during the search.
    Error(Error),

    /// The score and selected principal variation of the alpha-beta search.
    SearchResult(SearchResult),
}

pub fn alpha_beta(
    position: &Position,
    alpha: i32,
    beta: i32,
    depth: u32,
    color: Color,
    state: &mut SearchState,
    move_stack: &mut Vec<PieceMove>,
) -> AlphaBetaResult {
    // If we have already searched this position to the same depth or greater,
    // we can use the cached result directly.
    if let Some(entry) = state.transposition_table.try_get(position, depth) {
        state.cached_positions += 1;
        return AlphaBetaResult::SearchResult(SearchResult {
            principal_variation: entry.principal_variation.clone(),
            score: entry.score,
        });
    }

    // If we have exceeded the time limit, we should return an error.
    if state.start_time.elapsed().as_millis() >= state.time_limit {
        return AlphaBetaResult::Error(Error::Timeout);
    }

    // Increment the total number of nodes searched.
    state.nodes_searched += 1;

    // If we have reached the maximum depth, we should evaluate the position
    // and return the result.
    if depth == 0 {
        let score = evaluate_position(position);
        return AlphaBetaResult::SearchResult(SearchResult {
            principal_variation: Variation::new(),
            score,
        });
    }

    let moves = position.get_all_legal_moves().unwrap();

    let mut iteration = SearchIteration {
        alpha,
        beta,
        depth,
        state,
        principal_variations: Variations::new(),
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
        match test_move(mv, position, color, &mut iteration, move_stack) {
            MoveTestResult::Error(e) => return AlphaBetaResult::Error(e),

            // if score >= beta, return beta
            MoveTestResult::Pruned => {
                return AlphaBetaResult::SearchResult(SearchResult {
                    principal_variation: Variation::new_from(mv),
                    score: iteration.beta,
                })
            }
            MoveTestResult::ContinueSearching => {}
            MoveTestResult::Checkmate => {
                return AlphaBetaResult::SearchResult(SearchResult {
                    principal_variation: Variation::new_from(mv),
                    score: 1000,
                });
            }
        }
    }

    for mv in pass2 {
        match test_move(mv, position, color, &mut iteration, move_stack) {
            MoveTestResult::Error(e) => return AlphaBetaResult::Error(e),

            // if score >= beta, return beta
            MoveTestResult::Pruned => {
                return AlphaBetaResult::SearchResult(SearchResult {
                    principal_variation: Variation::new_from(mv),
                    score: iteration.beta,
                })
            }
            MoveTestResult::ContinueSearching => {}
            MoveTestResult::Checkmate => {
                return AlphaBetaResult::SearchResult(SearchResult {
                    principal_variation: Variation::new_from(mv),
                    score: 1000,
                });
            }
        }
    }

    trace!(
        "({}) Tested all moves for this board and no cutoff at depth {}. Alpha: {}, Beta: {}",
        color,
        depth,
        iteration.alpha,
        iteration.beta
    );
    trace!("\n{}", position.to_board_string());

    // If we have no PV, that means all moves were pruned.
    if iteration.principal_variations.variations.is_empty() {
        return AlphaBetaResult::SearchResult(SearchResult {
            principal_variation: Variation::new(),
            score: iteration.alpha,
        });
    }

    let picked_variation = iteration
        .principal_variations
        .variations
        .choose(&mut iteration.state.rng)
        .expect("No principal variations found");

    trace!("Picked principal variation: {}", picked_variation);

    iteration.state.transposition_table.insert(
        position.clone(),
        TranspositionTableEntry {
            depth,
            score: iteration.alpha,
            principal_variation: picked_variation.clone(),
        },
    );

    return AlphaBetaResult::SearchResult(SearchResult {
        principal_variation: picked_variation.clone(),
        score: iteration.alpha,
    });
}

enum MoveTestResult {
    Error(Error),
    Pruned,
    ContinueSearching,
    Checkmate,
}

fn test_move(
    mv: PieceMove,
    position: &Position,
    color: Color,
    iteration: &mut SearchIteration,
    move_stack: &mut Vec<PieceMove>,
) -> MoveTestResult {
    move_stack.push(mv);

    trace!(
        "({}) Testing move: {:?} at {} depth remaining",
        color,
        move_stack,
        iteration.depth
    );

    // Apply the move to a clone of the position, then
    // switch to the other player's perspective.
    let mut child = position.clone();
    child.apply_move(&mv).unwrap();
    child.invert();

    if child.is_checkmate().unwrap() {
        trace!(
            "({}) Checkmate found at depth remaining {}",
            color,
            iteration.depth
        );

        println!("{}", child.to_board_string());

        move_stack.pop();
        return MoveTestResult::Checkmate;
    }

    // Depth-first search the child position.
    let result = alpha_beta(
        &child,
        -iteration.beta,
        -iteration.alpha,
        iteration.depth - 1,
        match color {
            Color::White => Color::Black,
            Color::Black => Color::White,
        },
        iteration.state,
        move_stack,
    );

    match result {
        AlphaBetaResult::Error(e) => MoveTestResult::Error(e),
        AlphaBetaResult::SearchResult(result) => {
            // Negate the score to switch back to the original player's perspective.
            let score = -result.score;

            trace!(
                "({}) Move {:?} at {} depth remaining scored {}",
                color,
                move_stack,
                iteration.depth,
                score
            );

            // If the score is greater than or equal to beta, we can prune the search.
            if score >= iteration.beta {
                iteration.state.pruned += 1;

                trace!(
                    "({}) Pruning move: {:?} with {} depth remaining: {} >= {}",
                    color,
                    move_stack,
                    iteration.depth,
                    score,
                    iteration.beta
                );

                move_stack.pop();
                return MoveTestResult::Pruned;
            }

            let mut principal_variation = result.principal_variation;

            // If the score is greater than alpha, we have found a new best move.
            if score > iteration.alpha {
                iteration.alpha = score;
                principal_variation.prepend_move(mv);

                trace!(
                    "({}) Found new best move {:?} with {} depth remaining: {} ({})",
                    color,
                    move_stack,
                    iteration.depth,
                    &principal_variation,
                    score
                );
                iteration.principal_variations = Variations::new_from(principal_variation);
            } else if score == iteration.alpha {
                principal_variation.prepend_move(mv);
                trace!(
                    "({}) Found alternate best move {:?} with {} depth remaining: {} ({})",
                    color,
                    move_stack,
                    iteration.depth,
                    &principal_variation,
                    score
                );

                iteration.principal_variations.push(principal_variation);
            } else if iteration.principal_variations.variations.is_empty() {
                iteration.principal_variations.push(Variation::new_from(mv));
            }

            move_stack.pop();
            MoveTestResult::ContinueSearching
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

    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;

    #[test]
    pub fn alpha_beta() {
        let position: Position = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR".into();

        let mut transposition_table = TranspositionTable::new();
        let mut state = SearchState::new(&mut transposition_table);

        let result = search(&position, 4, &mut state);

        println!("{}", result.best_move);

        println!(
            "{}",
            result.principal_variation.print_as_positions(&position)
        )
    }

    #[test]
    pub fn dumb_check() {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::TRACE)
            .without_time()
            .finish();

        tracing::subscriber::set_global_default(subscriber).unwrap();

        let mut position: Position = "8/8/8/Rqpbkp2/8/8/8/K".into();
        position.invert();

        println!("{}", position.to_board_string());

        let mut transposition_table = TranspositionTable::new();
        let mut state = SearchState::new(&mut transposition_table);

        let result = search(&position, 6, &mut state);

        println!("{}", result.best_move);

        println!(
            "{}",
            result.principal_variation.print_as_positions(&position)
        )
    }
}
