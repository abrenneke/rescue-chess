use tracing::trace;

use crate::{evaluation::order_moves, piece_move::GameType, Color, PieceMove, PieceType, Position};

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

pub const CHECKMATE: i32 = -1000000;
pub const STALEMATE: i32 = 0;

#[derive(Debug, Clone)]
pub struct SearchParams {
    pub initial_bound: i32,
    pub depth: u32,
    pub quiescence_depth: u32,
    pub time_limit: u64,
    pub game_type: GameType,

    pub debug_print: bool,
    pub debug_print_verbose: bool,
    pub debug_print_all_moves: bool,

    pub previous_score: Option<i32>,
    pub window_size: i32,

    pub features: Features,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Features {
    pub enable_transposition_table: bool,
    pub enable_lmr: bool,
    pub enable_window_search: bool,
    pub enable_killer_moves: bool,
}

impl Default for Features {
    fn default() -> Self {
        Self {
            enable_transposition_table: true,
            enable_lmr: true,
            enable_window_search: true,
            enable_killer_moves: true,
        }
    }
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            initial_bound: 2000000,
            depth: 3,
            quiescence_depth: 4,
            time_limit: u64::MAX,
            game_type: GameType::Classic,
            debug_print: false,
            debug_print_verbose: false,
            debug_print_all_moves: false,
            previous_score: None,
            window_size: 50,
            features: Features::default(),
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

pub struct ScorePV {
    pub score: i32,
    pub pv: Vec<PieceMove>,
}

const WINDOW_MODIFIER: i32 = 2;

pub fn search(
    position: &Position,
    state: &mut SearchState,
    params: SearchParams,
    ply: usize
) -> Result<SearchResults, Error> {
    let mut alpha = match params.previous_score {
        Some(score) => score - params.window_size * WINDOW_MODIFIER,
        None => -params.window_size * WINDOW_MODIFIER,
    };

    let mut beta = match params.previous_score {
        Some(score) => score + params.window_size * WINDOW_MODIFIER,
        None => params.window_size * WINDOW_MODIFIER,
    };

    let mut failures = 0;

    if !params.features.enable_window_search {
        alpha = -params.initial_bound;
        beta = params.initial_bound;
    }

    loop {
        if position
            .get_all_legal_moves(params.game_type)
            .unwrap()
            .is_empty()
        {
            return Ok(SearchResults {
                best_move: None,
                principal_variation: None,
                score: CHECKMATE,
                nodes_searched: state.data.nodes_searched,
                cached_positions: state.data.cached_positions,
                depth: params.depth,
                time_taken_ms: state.data.start_time.elapsed().as_millis(),
                pruned: state.data.pruned,
            });
        }

        alpha = alpha.max(-params.initial_bound);
        beta = beta.min(params.initial_bound);

        match alpha_beta(position, alpha, beta, params.depth, state, &params, ply) {
            Ok(result) => {
                if let Some(pv) = result.principal_variation {
                    if !pv.is_empty() {
                        // Score within window - we're done!
                        let time_taken_ms = state.data.start_time.elapsed().as_millis();
                        let best_move = pv.first().cloned();

                        return Ok(SearchResults {
                            best_move,
                            principal_variation: Some(pv),
                            score: result.score,
                            nodes_searched: state.data.nodes_searched,
                            cached_positions: state.data.cached_positions,
                            depth: params.depth,
                            time_taken_ms,
                            pruned: state.data.pruned,
                        });
                    }
                }
            }
            Err(Error::Timeout) => return Err(Error::Timeout),
        }

        if !params.features.enable_window_search {
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

struct SearchIteration<'table: 'state, 'state, 'a> {
    alpha: i32,
    beta: i32,
    depth: u32,
    state: &'state mut SearchState< 'table, 'a>,
    principal_variation: Option<Vec<PieceMove>>,
}

impl<'a, 'b, 'c> std::fmt::Debug for SearchIteration<'a, 'b, 'c> {
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
fn should_reduce_move(mv: &PieceMove, depth: u32, move_index: usize, in_check: bool, alpha: i32, beta: i32) -> bool {
    if alpha > 900_000 || beta > 900_000 || alpha < -900_000 || beta < -900_000 {
        return false;
    }

    depth >= 3 && // Only reduce at deeper depths
    move_index >= 4 && // Don't reduce first few moves
    !mv.is_capture() && // Don't reduce captures
    !in_check && // Don't reduce when in check
    // Don't reduce pawn moves that are about to promote
    !(mv.piece_type == PieceType::Pawn && mv.to.get_row() <= 1) &&
    // Don't reduce central pawn moves in early/midgame
    !(mv.piece_type == PieceType::Pawn && 
        (mv.to.get_col() == 3 || mv.to.get_col() == 4) && 
        (mv.to.get_row() == 3 || mv.to.get_row() == 4))
}

pub fn alpha_beta(
    position: &Position,
    alpha: i32,
    beta: i32,
    depth: u32,
    state: &mut SearchState,
    params: &SearchParams,
    ply: usize,
) -> Result<SearchResult, Error> {
    let original_alpha = alpha;

    // If we have already searched this position to the same depth or greater,
    // we can use the cached result directly.
    if params.features.enable_transposition_table {
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

            state.data.cached_positions += 1;
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
    if state.data.start_time.elapsed().as_millis() >= state.data.time_limit as u128 {
        return Err(Error::Timeout);
    }

    // Increment the total number of nodes searched.
    state.data.nodes_searched += 1;

    if state.data.nodes_searched % 1_000_000 == 0 {
        trace!("Nodes searched: {}M", state.data.nodes_searched / 1_000_000);
    }

    // If the position is a checkmate, we should return a very low score.
    if position.is_checkmate(params.game_type).unwrap() {
        let score = CHECKMATE - (depth as i32);

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

    let prev_best_move = state.data.previous_pv;
    let ordered_moves = order_moves(position, moves, prev_best_move, state, ply, params);

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
        if let Some(result) = test_move(*mv, position, &mut iteration, params, depth, move_index, ply) {
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

    if params.features.enable_transposition_table {
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
    ply: usize,
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
    let mut score_pv: Option<ScorePV> = if params.features.enable_lmr && should_reduce_move(&mv, depth, move_index, in_check, iteration.alpha, iteration.beta) {
        let reduction = (depth.min(move_index as u32) / 3).max(1);

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
            ply + 1,
        );

        match result {
            Ok(reduced_result) => {
                let reduced_score = -reduced_result.score;
                // If the reduced search beats alpha, we need to do a full-depth search
                if reduced_score > iteration.alpha {
                    None // Signal that we need a full-depth search
                } else {
                    Some(ScorePV {
                        score: reduced_score,
                        pv: reduced_result.principal_variation.unwrap_or_default(),
                    })
                }
            }
            Err(e) => return Some(Err(e)),
        }
    } else {
        None
    };

    // If LMR was not done or the reduced search beat alpha, do a full-depth search
    if score_pv.is_none() {
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
            ply + 1,
        ) {
            Ok(result) => {
                score_pv = Some(ScorePV {
                    score: -result.score,
                    pv: result.principal_variation.unwrap_or_default(),
                });
            }
            Err(e) => return Some(Err(e)),
        }
    }

    let score_pv = score_pv.unwrap();

    // Rest of the move processing remains the same
    if score_pv.score >= iteration.beta {
        iteration.state.data.pruned += 1;

        if params.features.enable_killer_moves {
            iteration.state.killer_moves.add_killer(mv, ply);
        }

        if params.features.enable_transposition_table {
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
                score_pv.score,
                iteration.beta,
            );
        }

        return Some(Ok(SearchResult {
            principal_variation: None,
            score: iteration.beta,
        }));
    }

    if score_pv.score > iteration.alpha {
        iteration.alpha = score_pv.score;
        let mut principal_variation = vec![mv];
        principal_variation.extend(score_pv.pv);
        iteration.principal_variation = Some(principal_variation);

        if params.debug_print_verbose {
            trace!(
                "{}New best move: {} (score: {})",
                "\t".repeat((params.depth - iteration.depth) as usize),
                mv,
                iteration.alpha
            );
        }

        // Update the best move so far if we are at the root
        if depth == params.depth {
            iteration.state.data.best_move_so_far = Some(mv);

            if let Some(on_new_best_move) = iteration.state.callbacks.on_new_best_move {
                on_new_best_move(mv, iteration.alpha);
            }
        }
    }

    if params.debug_print_verbose {
        trace!(
            "{}Move search complete. No beta cutoff: {} (score: {})",
            "\t".repeat((params.depth - iteration.depth) as usize),
            mv,
            score_pv.score
        );
    }

    None
}

#[cfg(test)]
pub mod tests {
    use crate::{position::extended_fen::{EpdOperand, ExtendedPosition}, search::transposition_table::TranspositionTable};

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
        let result = search(&position, &mut state, params, 0).unwrap();
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

        let result = search(&position, &mut state, params, 0).unwrap();
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

            let result = search(&position, &mut state, params, 0).unwrap();
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

            let result = search(&position, &mut state, params, 0).unwrap();

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

        let result = search(&position, &mut state, params, 0).unwrap();
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

        let result = search(&position, &mut state, params, 0).unwrap();
        let best_move = result.best_move.unwrap().to_string();

        // White should play Bb5, pinning the knight
        assert_eq!(best_move, "Bb5", "Expected pin with Bb5, got {}", best_move);
    }

    #[test]
    fn mate_in_1() {
        tracing_subscriber::fmt().with_max_level(tracing::Level::TRACE).init();

        let position = Position::parse_from_fen("Q1Q5/P6k/8/5P2/6Q1/6B1/6PP/R3K2R w kq - 0 1").unwrap();

        let mut transposition_table = TranspositionTable::new();
        let mut state = SearchState::new(&mut transposition_table);

        let params = SearchParams {
            depth: 4,
            game_type: GameType::Classic,
            debug_print_verbose: true,
            ..Default::default()
        };

        let result = search(&position, &mut state, params, 0).unwrap();
        let best_move = result.best_move.unwrap().to_string();

        // White should play Qh8#
        assert_eq!(best_move, "Qh8", "Expected mate with Qh8, got {}", best_move);
    }

    fn test_mate(position: &str, expected_move: &str, depth: u32) {
        let position = Position::parse_from_fen(position).unwrap();

        let mut transposition_table = TranspositionTable::new();
        let mut state = SearchState::new(&mut transposition_table);

        let params = SearchParams {
            depth,
            game_type: GameType::Classic,
            features: Features {
                enable_lmr: false,
                enable_window_search: false,
                // enable_transposition_table: false,
                ..Default::default()
            },
            ..Default::default()
        };      

        let result = search(&position, &mut state, params, 0).unwrap();
        let best_move = result.best_move.unwrap().to_string();

        println!("Score: {}", result.score);
        println!("Principal variation: {:?}", result.principal_variation);

        assert_eq!(best_move, expected_move, "Expected mate with {}, got {}", expected_move, best_move);
        assert!(result.score >= -CHECKMATE, "Expected mate, got score {}", result.score);
    }

    #[test]
    fn mate_in_2() {
        test_mate("3qr2k/pbpp2pp/1p5N/3Q2b1/2P1P3/P7/1PP2PPP/R4RK1 w - - 0 1", "Qg8", 6);
    }

    #[test]
    fn mate_in_2_2() {
        test_mate("r1bq2k1/ppp2r1p/2np1pNQ/2bNpp2/2B1P3/3P4/PPP2PPP/R3K2R w KQ - 0 1", "Nxf6", 6);
    }

    #[test]
    fn mate_in_2_3() {
        test_mate("r1bk3r/1pp2ppp/pb1p1n2/n2P4/B3P1q1/2Q2N2/PB3PPP/RN3RK1 w - - 0 1", "Qxf6", 6);
    }

    #[test]
    fn mate_in_10() {
        test_mate("2R5/8/4p2K/6r1/5p2/2b5/2k4p/8 b - - 0 1", "Rb8", 10);
    }

    #[test]
    #[should_panic]
    fn opera_mate() {
        // Not sure about this one
        test_mate("8/1r3k2/5b2/8/8/p1p5/2K5/1R6 b - - 0 1", "Rxg8", 10);
    }

    #[test]
    fn mate_in_5() {
        test_mate("8/4K3/1P6/2PB1r1n/5p2/pp6/k1p4p/3Q4 w - - 0 1", "Qxc2", 10);
    }

    fn test_one_of_best_moves(position: &str, expected_moves: &[&str], depth: u32) {
        tracing_subscriber::fmt().init();

        let position = Position::parse_from_fen(position).unwrap();

        let mut transposition_table = TranspositionTable::new();
        let mut state = SearchState::new(&mut transposition_table);

        let params = SearchParams {
            depth,
            game_type: GameType::Classic,
            ..Default::default()
        };

        let result = search(&position, &mut state, params, 0).unwrap();

        let best_move = result.best_move.unwrap();

        let best_move = if position.true_active_color == Color::White {
            best_move.to_string()
        } else {
            best_move.inverted().to_string()
        };

        assert!(
            expected_moves.contains(&best_move.as_str()),
            "Expected one of {:?}, got {}",
            expected_moves,
            best_move
        );
    }

    #[test]
    fn stockfish_analysis_1() {
        test_one_of_best_moves("r4rk1/ppp3pp/3q4/1P4R1/2Pn1n2/2N5/PP3PPP/R1BQ2K1 b - - 0 15", &["h6", "Rae8", "Nh3"] , 16);
    }

    fn test_sts(extended_position: &str, depth: u32) {
        tracing_subscriber::fmt().init();

        let position = ExtendedPosition::parse_from_epd(extended_position).unwrap();

        let mut transposition_table = TranspositionTable::new();
        let mut state = SearchState::new(&mut transposition_table);

        let params = SearchParams {
            depth,
            game_type: GameType::Classic,
            ..Default::default()
        };

        let result = search(&position.position, &mut state, params, 0).unwrap();

        let best_move = result.best_move.unwrap();

        let best_move = if position.position.true_active_color == Color::White {
            best_move.to_string()
        } else {
            best_move.inverted().to_string()
        };

        let expected_best_move = position.get_operation("bm").unwrap().first().unwrap();

        match expected_best_move {
            EpdOperand::SanMove(expected_best_move) => {

                if best_move != *expected_best_move {
                    println!("Expected best move: {}", expected_best_move);
                    println!("Principal variation: {:?}", result.principal_variation);

                    println!("{}", position.position.to_board_string_with_rank_file_holding());
                }

                assert_eq!(best_move, *expected_best_move);
            }
            _ => panic!("Expected best move not found"),
        }
    }

    #[test]
    fn sts1() {
        test_sts("1kr5/3n4/q3p2p/p2n2p1/PppB1P2/5BP1/1P2Q2P/3R2K1 w - - bm f5; id \"Undermine.001\"; c0 \"f5=10, Be5+=2, Bf2=3, Bg4=2\";", 10);
    }
}
