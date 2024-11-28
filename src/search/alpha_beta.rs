use std::i32;

use crate::{evaluation::order_moves, piece_move::GameType, PieceMove, Position};

use super::{
    quiescence_search::quiescence_search,
    search_results::{SearchResults, SearchState},
    transposition_table::TranspositionTableEntry,
};

#[derive(Clone)]
pub struct SearchResult {
    pub principal_variation: Option<Vec<PieceMove>>,
    pub score: i32,
}

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
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            initial_bound: 1000000,
            depth: 3,
            quiescence_depth: 4,
            time_limit: u128::MAX,
            game_type: GameType::Classic,
            debug_print: false,
            debug_print_verbose: false,
            debug_print_all_moves: false,
            previous_score: None,
            window_size: 50,
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

    loop {
        alpha = alpha.max(-params.initial_bound);
        beta = beta.min(params.initial_bound);

        match alpha_beta(position, alpha, beta, params.depth, state, &params, true) {
            Ok(result) => {
                if let Some(pv) = result.principal_variation {
                    if !pv.is_empty() {
                        // Score within window - we're done!
                        let time_taken_ms = state.start_time.elapsed().as_millis();
                        let best_move = pv.first().cloned();

                        return SearchResults {
                            best_move,
                            principal_variation: pv,
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

        if params.debug_print {
            println!(
                "Window search failed: alpha={}, beta={}, widening window",
                alpha, beta
            );
        }

        // If we get here, the score was outside our window
        // Double the window size and try again
        alpha -= params.window_size;
        beta += params.window_size;

        // If window gets too big, just use full bounds
        if beta - alpha >= params.initial_bound * 2 {
            alpha = -params.initial_bound;
            beta = params.initial_bound;
        }
    }
}

struct SearchIteration<'table: 'state, 'state> {
    alpha: i32,
    beta: i32,
    depth: u32,
    state: &'state mut SearchState<'table>,
    principal_variation: Option<Vec<PieceMove>>,
    is_white: bool,
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

pub fn alpha_beta(
    position: &Position,
    alpha: i32,
    beta: i32,
    depth: u32,
    state: &mut SearchState,
    params: &SearchParams,
    is_white: bool,
) -> Result<SearchResult, Error> {
    // If we have already searched this position to the same depth or greater,
    // we can use the cached result directly.
    if let Some(entry) = state.transposition_table.try_get(position, depth) {
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

    // If we have exceeded the time limit, we should return an error.
    if state.start_time.elapsed().as_millis() >= state.time_limit {
        return Err(Error::Timeout);
    }

    // Increment the total number of nodes searched.
    state.nodes_searched += 1;

    if state.nodes_searched % 1000000 == 0 {
        println!("Nodes searched: {}", state.nodes_searched);
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
            is_white,
            params.depth,
        )?
        .score;

        return Ok(SearchResult {
            principal_variation: Some(vec![]),
            score,
        });
    }

    // If the position is a checkmate, we should return a very low score.
    // This is just to prevent the engine from continuing past a king capture.
    if position.is_checkmate(params.game_type).unwrap() {
        let score = -1000000;

        if params.debug_print_verbose {
            println!(
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

    let moves = position.get_all_legal_moves(params.game_type).unwrap();

    let prev_best_move = state
        .transposition_table
        .try_get(position, depth)
        .map(|entry| entry.principal_variation);

    let ordered_moves = order_moves(position, moves, prev_best_move);

    let mut iteration = SearchIteration {
        alpha,
        beta,
        depth,
        state,
        principal_variation: None,
        is_white,
    };

    for mv in ordered_moves {
        if let Some(result) = test_move(mv, position, &mut iteration, params) {
            return result;
        }
    }

    if params.debug_print_verbose {
        println!(
            "{}Principal variation: {:?}",
            "\t".repeat((params.depth - iteration.depth) as usize),
            iteration.principal_variation
        );
    }

    let principal_variation = iteration.principal_variation;

    if let Some(principal_variation) = &principal_variation {
        iteration.state.transposition_table.insert(
            position.clone(),
            TranspositionTableEntry {
                depth,
                score: iteration.alpha,
                principal_variation: principal_variation.first().cloned().unwrap(),
            },
        );
    }

    return Ok(SearchResult {
        principal_variation,
        score: iteration.alpha,
    });
}

fn test_move(
    mv: PieceMove,
    position: &Position,
    iteration: &mut SearchIteration,
    params: &SearchParams,
) -> Option<Result<SearchResult, Error>> {
    if params.debug_print_verbose {
        println!(
            "{}Testing move for {}: {} (alpha: {}, beta: {})",
            "\t".repeat((params.depth - iteration.depth) as usize),
            if iteration.is_white { "white" } else { "black" },
            if iteration.is_white {
                mv
            } else {
                mv.inverted()
            },
            iteration.alpha,
            iteration.beta
        );
    }

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
        params,
        !iteration.is_white,
    );

    match result {
        Err(e) => Some(Err(e)),
        Ok(result) => {
            // Negate the score to switch back to the original player's perspective.
            let score = -result.score;

            // If the score is greater than or equal to beta, we can prune the search.
            if score >= iteration.beta {
                iteration.state.pruned += 1;

                if params.debug_print_verbose {
                    println!(
                        "{}Pruned move: {} (score: {})",
                        "\t".repeat((params.depth - iteration.depth) as usize),
                        if iteration.is_white {
                            mv
                        } else {
                            mv.inverted()
                        },
                        score
                    );
                }

                return Some(Ok(SearchResult {
                    principal_variation: None,
                    score: iteration.beta,
                }));
            }

            // If the score is greater than alpha, we have found a new best move.
            if score > iteration.alpha {
                if params.debug_print_verbose {
                    println!(
                        "{}New best move: {} (score: {})",
                        "\t".repeat((params.depth - iteration.depth) as usize),
                        if iteration.is_white {
                            mv
                        } else {
                            mv.inverted()
                        },
                        score
                    );
                }

                iteration.alpha = score;

                let mut principal_variation =
                    result.principal_variation.clone().unwrap_or_default();
                principal_variation.insert(0, mv);

                iteration.principal_variation = Some(principal_variation);
            } else {
                if params.debug_print_verbose {
                    println!(
                        "{}Move: {} (score: {})",
                        "\t".repeat((params.depth - iteration.depth) as usize),
                        if iteration.is_white {
                            mv
                        } else {
                            mv.inverted()
                        },
                        score
                    );
                }
            }

            None
        }
    }
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

        println!(
            "{}",
            result
                .principal_variation
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

        println!(
            "Defended Scholar's Mate with {} (score: {}, nodes: {}, cached: {}, pruned: {})",
            best_move, result.score, result.nodes_searched, result.cached_positions, result.pruned
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

            println!(
                "chose {} (score: {}, nodes: {}, cached: {}, pruned: {})",
                best_move,
                result.score,
                result.nodes_searched,
                result.cached_positions,
                result.pruned
            );

            println!(
                "Principal variation: {}",
                result
                    .principal_variation
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

        println!("{}", position.to_board_string_with_rank_file(false));

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

            println!(
                "chose {} (score: {}, nodes: {}, cached: {}, pruned: {})",
                best_move,
                result.score,
                result.nodes_searched,
                result.cached_positions,
                result.pruned
            );

            println!(
                "Principal variation: {}",
                result
                    .principal_variation
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
