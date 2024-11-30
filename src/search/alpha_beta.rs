use tracing::trace;

use crate::{evaluation::order_moves, piece_move::GameType, Color, PieceMove, Position};

use super::{
    quiescence_search::quiescence_search,
    search_results::{SearchResults, SearchState},
    transposition_table::{NodeType, TranspositionTableEntry},
};

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub principal_variation: Option<Vec<PieceMove>>,
    pub score: i32,
}

const CHECKMATE: i32 = -1000000;
const STALEMATE: i32 = 0;

pub struct SearchParams {
    pub initial_bound: i32,
    pub depth: u32,
    pub quiescence_depth: u32,
    pub time_limit: u128,
    pub game_type: GameType,

    pub debug_print: bool,
    pub debug_print_verbose: bool,
    pub debug_print_all_moves: bool,

    pub previous_score: Option<i32>,
    pub window_size: i32,

    pub enable_transposition_table: bool,
    pub enable_lmr: bool,
    pub enable_window_search: bool,
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            initial_bound: 2000000,
            depth: 3,
            quiescence_depth: 4,
            time_limit: u128::MAX,
            game_type: GameType::Classic,
            debug_print: false,
            debug_print_verbose: false,
            debug_print_all_moves: false,
            previous_score: None,
            window_size: 50,
            enable_transposition_table: true,
            enable_lmr: true,
            enable_window_search: true,
        }
    }
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

#[derive(Debug)]
pub enum Error {
    Timeout,
}

pub fn search(position: &Position, state: &mut SearchState, params: SearchParams) -> SearchResults {
    let mut alpha = match params.previous_score {
        Some(score) => score - params.window_size,
        None => -params.window_size,
    };

    let mut beta = match params.previous_score {
        Some(score) => score + params.window_size,
        None => params.window_size,
    };

    let mut failures = 0;

    if !params.enable_window_search {
        alpha = -params.initial_bound;
        beta = params.initial_bound;
    }

    loop {
        if position
            .get_all_legal_moves(params.game_type)
            .unwrap()
            .is_empty()
        {
            return SearchResults {
                best_move: None,
                principal_variation: None,
                score: CHECKMATE,
                nodes_searched: state.nodes_searched,
                cached_positions: state.cached_positions,
                depth: params.depth,
                time_taken_ms: state.start_time.elapsed().as_millis(),
                pruned: state.pruned,
            };
        }

        alpha = alpha.max(-params.initial_bound);
        beta = beta.min(params.initial_bound);

        match alpha_beta(position, alpha, beta, params.depth, state, &params) {
            Ok(result) => {
                if let Some(pv) = result.principal_variation {
                    if !pv.is_empty() {
                        // Score within window - we're done!
                        let time_taken_ms = state.start_time.elapsed().as_millis();
                        let best_move = pv.first().cloned();

                        return SearchResults {
                            best_move,
                            principal_variation: Some(pv),
                            score: result.score,
                            nodes_searched: state.nodes_searched,
                            cached_positions: state.cached_positions,
                            depth: params.depth,
                            time_taken_ms,
                            pruned: state.pruned,
                        };
                    }
                }
            }
            Err(Error::Timeout) => panic!("Search timed out"),
        }

        if !params.enable_window_search {
            panic!("Search failed to find a score within window");
        }

        if params.debug_print {
            trace!(
                "Window search failed: alpha={}, beta={}, widening window",
                alpha,
                beta
            );
        }

        failures += 1;

        // If we get here, the score was outside our window
        // Double the window size and try again
        alpha -= params.window_size * failures;
        beta += params.window_size * failures;

        // If window gets too big, just use full bounds
        if beta - alpha >= params.initial_bound * 2 {
            alpha = -params.initial_bound;
            beta = params.initial_bound;
        }

        if failures > 10 {
            alpha = -params.initial_bound;
            beta = params.initial_bound;
        }

        if failures > 11 {
            trace!("Failed to find a score within window after 10 tries");
            trace!("Failing position: {}", position.to_fen());

            panic!("Failed to find a score within window after 10 tries");
        }
    }
}

struct SearchIteration<'table: 'state, 'state> {
    alpha: i32,
    beta: i32,
    depth: u32,
    state: &'state mut SearchState<'table>,
    principal_variation: Option<Vec<PieceMove>>,
}

impl<'a, 'b> std::fmt::Debug for SearchIteration<'a, 'b> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SearchIteration")
            .field("alpha", &self.alpha)
            .field("beta", &self.beta)
            .field("depth", &self.depth)
            .field("principal_variation", &self.principal_variation)
            .finish()
    }
}

// Late move reduction
fn should_reduce_move(mv: &PieceMove, depth: u32, move_index: usize, in_check: bool) -> bool {
    depth >= 3 && // Only reduce at deeper depths
    move_index >= 4 && // Don't reduce first few moves
    !mv.is_capture() && // Don't reduce captures
    !in_check // Don't reduce when in check
}

pub fn alpha_beta(
    position: &Position,
    alpha: i32,
    beta: i32,
    depth: u32,
    state: &mut SearchState,
    params: &SearchParams,
) -> Result<SearchResult, Error> {
    let original_alpha = alpha;

    // If we have already searched this position to the same depth or greater,
    // we can use the cached result directly.
    if params.enable_transposition_table {
        if let Some(entry) = state
            .transposition_table
            .try_get(position, depth, alpha, beta)
        {
            if params.debug_print_verbose {
                trace!(
                    "{}Cached position found: {}",
                    "\t".repeat((params.depth - depth) as usize),
                    entry.score
                );
            }

            state.cached_positions += 1;
            return Ok(SearchResult {
                principal_variation: Some(
                    state
                        .transposition_table
                        .principal_variation_list(position, depth),
                ),
                score: entry.score,
            });
        }
    }

    // If we have exceeded the time limit, we should return an error.
    if state.start_time.elapsed().as_millis() >= state.time_limit {
        return Err(Error::Timeout);
    }

    // Increment the total number of nodes searched.
    state.nodes_searched += 1;

    if state.nodes_searched % 1000000 == 0 {
        trace!("Nodes searched: {}", state.nodes_searched);
    }

    // If the position is a checkmate, we should return a very low score.
    // This is just to prevent the engine from continuing past a king capture.
    if position.is_checkmate(params.game_type).unwrap() {
        let score = -1000000;

        if params.debug_print_verbose {
            trace!(
                "{}Checkmate found: {}",
                "\t".repeat((params.depth - depth) as usize),
                score
            );
        }

        return Ok(SearchResult {
            principal_variation: Some(vec![]),
            score,
        });
    }

    // If we have reached the maximum depth, we should evaluate the position
    // and return the result.
    if depth == 0 {
        let score = quiescence_search(
            position,
            alpha,
            beta,
            params.quiescence_depth,
            state,
            params,
            params.depth,
        )?
        .score;

        if params.debug_print_verbose {
            trace!(
                "{}Quiescence search complete: {}",
                "\t".repeat((params.depth - depth) as usize),
                score
            );
        }

        return Ok(SearchResult {
            principal_variation: Some(vec![]),
            score,
        });
    }

    let moves = position.get_all_legal_moves(params.game_type).unwrap();

    if moves.is_empty() {
        if params.debug_print_verbose {
            trace!(
                "{}Stalemate found, scoring {}",
                "\t".repeat((params.depth - depth) as usize),
                STALEMATE
            );
        }

        return Ok(SearchResult {
            principal_variation: Some(vec![]),
            score: STALEMATE,
        });
    }

    let prev_best_move = state
        .transposition_table
        .try_get(position, depth, alpha, beta)
        .and_then(|entry| entry.principal_variation);

    let ordered_moves = order_moves(position, moves, prev_best_move);

    let mut iteration = SearchIteration {
        alpha,
        beta,
        depth,
        state,
        principal_variation: None,
    };

    if params.debug_print_verbose {
        trace!(
            "{}Searching depth {} with alpha={}, beta={}",
            "\t".repeat((params.depth - depth) as usize),
            depth,
            alpha,
            beta
        );
    }

    for (move_index, mv) in ordered_moves.iter().enumerate() {
        if let Some(result) = test_move(*mv, position, &mut iteration, params, depth, move_index) {
            return result;
        }
    }

    if params.debug_print_verbose {
        trace!(
            "{}Principal variation: {:?}",
            "\t".repeat((params.depth - iteration.depth) as usize),
            iteration.principal_variation
        );
    }

    let principal_variation = iteration.principal_variation;
    let score = iteration.alpha;

    // if principal_variation.is_none() {
    //     panic!(
    //         "No principal variation found. Alpha: {}, beta: {}, possible moves: {:?}",
    //         alpha, beta, ordered_moves
    //     );
    // }

    // Determine node type based on search result
    let node_type = if score <= original_alpha {
        NodeType::UpperBound
    } else if score >= beta {
        NodeType::LowerBound
    } else {
        NodeType::Exact
    };

    if params.enable_transposition_table {
        if let Some(principal_variation) = &principal_variation {
            iteration.state.transposition_table.insert(
                position.clone(),
                TranspositionTableEntry {
                    depth,
                    score,
                    principal_variation: principal_variation.first().cloned(),
                    node_type,
                },
            );
        }
    }

    return Ok(SearchResult {
        principal_variation,
        score: iteration.alpha,
    });
}

// Modify the test_move function to implement LMR
fn test_move(
    mv: PieceMove,
    position: &Position,
    iteration: &mut SearchIteration,
    params: &SearchParams,
    depth: u32,
    move_index: usize,
) -> Option<Result<SearchResult, Error>> {
    if params.debug_print_verbose {
        trace!(
            "{}Testing move for {}: {} (alpha: {}, beta: {}) at {}",
            "\t".repeat((params.depth - iteration.depth) as usize),
            if position.true_active_color == Color::White {
                "white"
            } else {
                "black"
            },
            if position.true_active_color == Color::White {
                mv
            } else {
                mv.inverted()
            },
            iteration.alpha,
            iteration.beta,
            position.to_fen(),
        );
    }

    // Apply the move to a clone of the position
    let mut child = position.clone();
    child.apply_move(mv).unwrap();
    let in_check = child.is_king_in_check().unwrap();

    child.invert();

    // Implement Late Move Reduction
    let mut score = if params.enable_lmr && should_reduce_move(&mv, depth, move_index, in_check) {
        // Calculate reduction depth - can be tuned
        let reduction = if move_index > 6 { 2 } else { 1 };

        if params.debug_print_verbose {
            trace!(
                "{}Reduced search for move: {}",
                "\t".repeat((params.depth - iteration.depth) as usize),
                mv
            );
        }

        // Reduced depth search
        let result = alpha_beta(
            &child,
            -iteration.alpha - 1, // Use a null window for reduced search
            -iteration.alpha,
            iteration.depth - 1 - reduction,
            iteration.state,
            params,
        );

        match result {
            Ok(reduced_result) => {
                let reduced_score = -reduced_result.score;
                // If the reduced search beats alpha, we need to do a full-depth search
                if reduced_score > iteration.alpha {
                    None // Signal that we need a full-depth search
                } else {
                    Some(reduced_score)
                }
            }
            Err(e) => return Some(Err(e)),
        }
    } else {
        None
    };

    // If LMR was not done or the reduced search beat alpha, do a full-depth search
    if score.is_none() {
        if params.debug_print_verbose {
            trace!(
                "{}Full-depth search for move: {}",
                "\t".repeat((params.depth - iteration.depth) as usize),
                if position.true_active_color == Color::White {
                    mv
                } else {
                    mv.inverted()
                }
            );
        }

        match alpha_beta(
            &child,
            -iteration.beta,
            -iteration.alpha,
            iteration.depth - 1,
            iteration.state,
            params,
        ) {
            Ok(result) => {
                score = Some(-result.score);
            }
            Err(e) => return Some(Err(e)),
        }
    }

    let score = score.unwrap();

    // Rest of the move processing remains the same
    if score >= iteration.beta {
        iteration.state.pruned += 1;

        if params.enable_transposition_table {
            iteration.state.transposition_table.insert(
                position.clone(),
                TranspositionTableEntry {
                    depth,
                    score: iteration.beta,
                    principal_variation: Some(mv),
                    node_type: NodeType::LowerBound,
                },
            );
        }

        if params.debug_print_verbose {
            trace!(
                "{}Pruned move: {} (score: {}, beta: {})",
                "\t".repeat((params.depth - iteration.depth) as usize),
                mv,
                score,
                iteration.beta,
            );
        }

        return Some(Ok(SearchResult {
            principal_variation: None,
            score: iteration.beta,
        }));
    }

    if score > iteration.alpha {
        iteration.alpha = score;
        let mut principal_variation = vec![mv];
        if let Ok(result) = alpha_beta(
            &child,
            -iteration.beta,
            -iteration.alpha,
            iteration.depth - 1,
            iteration.state,
            params,
        ) {
            if let Some(mut child_pv) = result.principal_variation {
                principal_variation.append(&mut child_pv);
            }
        }
        iteration.principal_variation = Some(principal_variation);

        if params.debug_print_verbose {
            trace!(
                "{}New best move: {} (score: {})",
                "\t".repeat((params.depth - iteration.depth) as usize),
                mv,
                iteration.alpha
            );
        }
    }

    if params.debug_print_verbose {
        trace!(
            "{}Move search complete. No beta cutoff: {} (score: {})",
            "\t".repeat((params.depth - iteration.depth) as usize),
            mv,
            score
        );
    }

    None
}

#[cfg(test)]
pub mod tests {
    use crate::search::transposition_table::TranspositionTable;

    use super::*;

    #[test]
    fn test_scholars_mate_defense() {
        // Set up a position one move before Scholar's Mate
        // 1. e4 e5
        // 2. Bc4 Nc6
        // 3. Qh5 (threatening Qxf7#)
        let position =
            Position::from_moves(&["e4", "e5", "Bc4", "Nc6", "Qh5"], GameType::Classic).unwrap();

        let mut transposition_table = TranspositionTable::new();
        let mut state = SearchState::new(&mut transposition_table);

        let params = SearchParams {
            depth: 4,
            game_type: GameType::Classic,
            ..Default::default()
        };

        // Search to depth 4 which should be enough to detect the checkmate threat
        let result = search(&position, &mut state, params);
        let best_move = result.best_move.unwrap().inverted().to_string();

        trace!(
            "{}",
            result
                .principal_variation
                .unwrap()
                .iter()
                .map(|mv| mv.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        );

        // Black should defend against Qxf7# by either:
        // 1. g6 (blocking the queen's attack)
        // 2. Nf6 (blocking and threatening the queen)
        assert!(
            best_move == "Qf6" || best_move == "g6" || best_move == "Qe7",
            "Expected defensive move Qf6 or g6 or Qe7, got {}",
            best_move
        );

        trace!(
            "Defended Scholar's Mate with {} (score: {}, nodes: {}, cached: {}, pruned: {})",
            best_move,
            result.score,
            result.nodes_searched,
            result.cached_positions,
            result.pruned
        );
    }

    #[test]
    fn test_scholars_mate_completion() {
        // Test what happens after b3
        let position = Position::from_moves(
            &["e4", "e5", "Bc4", "Nc6", "Qh5", "b6", "Qxf7"],
            GameType::Classic,
        )
        .unwrap();

        let mut transposition_table = TranspositionTable::new();
        let mut state = SearchState::new(&mut transposition_table);

        let params = SearchParams {
            depth: 1,
            game_type: GameType::Classic,
            ..Default::default()
        };

        let result = search(&position, &mut state, params);
        assert!(position.is_checkmate(GameType::Classic).unwrap());
        assert_eq!(result.score, -1000000);
        assert!(result.best_move.is_none());
    }

    #[test]
    fn test_obvious_queen_capture() {
        let position = Position::parse_from_fen(
            "rnb1kbnr/pppp1ppp/8/4q3/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 1",
        )
        .unwrap();

        // Test at multiple depths to see where it breaks
        for depth in 1..=4 {
            let mut transposition_table = TranspositionTable::new();
            let mut state = SearchState::new(&mut transposition_table);

            let params = SearchParams {
                depth,
                game_type: GameType::Classic,
                debug_print_all_moves: true,
                debug_print_verbose: true,
                ..Default::default()
            };

            dbg!(position.get_all_legal_moves(GameType::Classic)).unwrap();

            let result = search(&position, &mut state, params);
            let best_move = result.best_move.unwrap().to_string();

            trace!(
                "chose {} (score: {}, nodes: {}, cached: {}, pruned: {})",
                best_move,
                result.score,
                result.nodes_searched,
                result.cached_positions,
                result.pruned
            );

            trace!(
                "Principal variation: {}",
                result
                    .principal_variation
                    .unwrap()
                    .iter()
                    .map(|mv| mv.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            );

            assert_eq!(
                best_move, "xe5",
                "At depth {}, expected queen capture dxe5, got {}",
                depth, best_move
            );
        }
    }

    #[test]
    fn test_obvious_defense() {
        let position = Position::from_moves(&["e4", "e6", "e5", "Nc6"], GameType::Classic).unwrap();

        trace!("{}", position.to_board_string_with_rank_file(false));

        // Test at multiple depths to see where it breaks
        for depth in 2..=5 {
            let mut transposition_table = TranspositionTable::new();
            let mut state = SearchState::new(&mut transposition_table);

            let params = SearchParams {
                depth,
                game_type: GameType::Classic,
                debug_print_all_moves: true,
                debug_print_verbose: false,
                ..Default::default()
            };

            let result = search(&position, &mut state, params);

            let best_move = result.best_move.unwrap().to_string();

            trace!(
                "chose {} (score: {}, nodes: {}, cached: {}, pruned: {})",
                best_move,
                result.score,
                result.nodes_searched,
                result.cached_positions,
                result.pruned
            );

            trace!(
                "Principal variation: {}",
                result
                    .principal_variation
                    .unwrap()
                    .iter()
                    .map(|mv| mv.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            );

            assert!(
                best_move == "d4" || best_move == "Nf3" || best_move == "f4" || best_move == "Qh5",
                "At depth {}, expected d4 or Nf3 or f4 or Qh5, got {}",
                depth,
                best_move
            );
        }
    }

    #[test]
    fn test_fork_recognition() {
        // Set up a position where white can fork black's king and rook with a knight
        let position =
            Position::parse_from_fen("r3k3/ppp2ppp/8/3N4/8/8/PPP2PPP/4K3 w - - 0 1").unwrap();

        let mut transposition_table = TranspositionTable::new();
        let mut state = SearchState::new(&mut transposition_table);

        let params = SearchParams {
            depth: 3,
            game_type: GameType::Classic,
            ..Default::default()
        };

        let result = search(&position, &mut state, params);
        let best_move = result.best_move.unwrap().to_string();

        // White should play Nf6+, forking king and rook
        assert_eq!(
            best_move, "Nxc7",
            "Expected knight fork Nf6+, got {}",
            best_move
        );
    }

    #[test]
    fn test_pin_recognition() {
        // Set up a position where white can pin black's knight to their king with a bishop
        let position =
            Position::parse_from_fen("r3k2r/pppn1p1p/8/8/8/3B4/PPP2PPP/4K3 w - - 0 1").unwrap();

        let mut transposition_table = TranspositionTable::new();
        let mut state = SearchState::new(&mut transposition_table);

        let params = SearchParams {
            depth: 3,
            game_type: GameType::Classic,
            ..Default::default()
        };

        let result = search(&position, &mut state, params);
        let best_move = result.best_move.unwrap().to_string();

        // White should play Bb5, pinning the knight
        assert_eq!(best_move, "Bb5", "Expected pin with Bb5, got {}", best_move);
    }
}
