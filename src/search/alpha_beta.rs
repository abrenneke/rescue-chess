use tracing::trace;

use crate::{
    evaluation::{ordering::order_moves, piece_value},
    features::{EvaluationWeights, Features},
    piece_move::GameType,
    Color, PieceMove, PieceType, Position,
};

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
    pub initial_alpha: i32,
    pub initial_beta: i32,

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
    pub weights: EvaluationWeights,
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            initial_alpha: MIN_ALPHA,
            initial_beta: MAX_BETA,
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
            weights: EvaluationWeights::default(),
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

#[derive(Debug, Clone)]
pub enum AlphaBetaError {
    Timeout,
}

impl std::fmt::Display for AlphaBetaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlphaBetaError::Timeout => write!(f, "Search timed out"),
        }
    }
}

pub struct ScorePV {
    pub score: i32,
    pub pv: Vec<PieceMove>,
}

const WINDOW_MODIFIER: i32 = 2;

pub const MIN_ALPHA: i32 = -2_000_000;
pub const MAX_BETA: i32 = 2_000_000;

pub struct MoveScore {
    pub mv: PieceMove,
    pub score: i32,
    pub principal_variation: Option<Vec<PieceMove>>,
}

pub fn score_all_moves(
    position: &Position,
    state: &mut SearchState,
    params: SearchParams,
    ply: usize,
) -> Result<Vec<MoveScore>, AlphaBetaError> {
    let mut scores = Vec::new();
    let mut position = position.clone();

    // Get all legal moves
    let moves = position.get_all_legal_moves(params.game_type).unwrap();
    if moves.is_empty() {
        // Handle checkmate or stalemate
        if position.is_checkmate(params.game_type).unwrap() {
            return Ok(vec![]);
        }
        return Ok(vec![]);
    }

    // Order moves using the existing move ordering function
    let prev_pv = state.data.previous_pv.as_ref();
    let ordered_moves = order_moves(&mut position, moves, prev_pv, state, ply, &params);

    // Evaluate each move
    for mv in ordered_moves {
        let restore = position.apply_move(mv).unwrap();
        position.invert();

        // Use a wide window for accurate scoring
        let result = alpha_beta(
            &mut position,
            params.initial_alpha,
            params.initial_beta,
            params.depth - 1,
            state,
            &params,
            ply + 1,
        );

        position.invert();
        position.unapply_move(mv, restore).unwrap();

        match result {
            Ok(search_result) => {
                scores.push(MoveScore {
                    mv,
                    score: -search_result.score, // Negate score since it's from opponent's perspective
                    principal_variation: search_result.principal_variation,
                });
            }
            Err(e) => return Err(e),
        }
    }

    // Sort moves by score in descending order
    scores.sort_by(|a, b| b.score.cmp(&a.score));

    Ok(scores)
}

pub fn search(
    position: &Position,
    state: &mut SearchState,
    params: SearchParams,
    ply: usize,
) -> Result<SearchResults, AlphaBetaError> {
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
        alpha = MIN_ALPHA;
        beta = MAX_BETA;
    }

    // Fine to clone the root position
    let mut position = position.clone();

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
                alpha,
                beta,
            });
        }

        alpha = alpha.max(MIN_ALPHA);
        beta = beta.min(MAX_BETA);

        match alpha_beta(
            &mut position,
            alpha,
            beta,
            params.depth,
            state,
            &params,
            ply,
        ) {
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
                            alpha,
                            beta,
                        });
                    }
                }
            }
            Err(AlphaBetaError::Timeout) => return Err(AlphaBetaError::Timeout),
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
        if beta - alpha >= params.initial_alpha * 5 {
            alpha = MIN_ALPHA;
            beta = MAX_BETA;
        }

        if failures > 10 {
            alpha = MIN_ALPHA;
            beta = MAX_BETA;
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
    state: &'state mut SearchState<'table, 'a>,
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
fn should_reduce_move(
    mv: &PieceMove,
    depth: u32,
    move_index: usize,
    in_check: bool,
    alpha: i32,
    beta: i32,
) -> bool {
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
    !(mv.piece_type == PieceType::Pawn && (mv.to.get_col() == 3 || mv.to.get_col() == 4) && (mv.to.get_row() == 3 || mv.to.get_row() == 4))
}

pub fn alpha_beta(
    position: &mut Position,
    alpha: i32,
    beta: i32,
    depth: u32,
    state: &mut SearchState,
    params: &SearchParams,
    ply: usize,
) -> Result<SearchResult, AlphaBetaError> {
    let original_alpha = alpha;

    // If we have already searched this position to the same depth or greater,
    // we can use the cached result directly.
    if params.features.enable_transposition_table {
        if let Some(entry) =
            state
                .transposition_table
                .try_get(&position.to_hashable(), depth, alpha, beta)
        {
            if params.debug_print_verbose {
                trace!(
                    "{}Cached position found: {}",
                    "\t".repeat((params.depth - depth) as usize),
                    entry.score
                );
            }

            if depth >= params.depth - 2 {
                // Only log near root
                println!(
                    "TT hit: depth={}, node_type={:?}, score={}, pos={}, entry.alpha={}, entry.beta={}, alpha={}, beta={}",
                    depth,
                    entry.node_type,
                    entry.score,
                    position.to_fen(),
                    entry.alpha,
                    entry.beta,
                    alpha,
                    beta
                );
            }

            state.data.cached_positions += 1;
            return Ok(SearchResult {
                principal_variation: Some(entry.principal_variation.clone()),
                score: entry.score,
            });
        }
    }

    // If we have exceeded the time limit, we should return an error.
    if state.data.start_time.elapsed().as_millis() >= state.data.time_limit as u128 {
        return Err(AlphaBetaError::Timeout);
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
            ply,
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

    if params.features.enable_null_move_pruning && should_try_null_move(position, depth, beta) {
        // Make a null move - essentially just switch sides without making a move
        let mut null_pos = position.clone();
        null_pos.invert();

        // Enhanced adaptive null move reduction
        let r = if depth > 6 {
            // Increase R when we have more material
            let material = get_material_count(position, position.true_active_color);
            if material > piece_value(PieceType::Queen) * 2 {
                4 // More aggressive pruning in piece-heavy positions
            } else {
                3
            }
        } else {
            2
        };
        let null_depth = (depth - 1 - r).max(0);

        // Search with a null window around beta
        match alpha_beta(
            &mut null_pos,
            -beta,
            -beta + 1,
            null_depth,
            state,
            params,
            ply + 1,
        ) {
            Ok(null_result) => {
                let null_score = -null_result.score;

                // If the null move fails high, we can likely prune this subtree
                if null_score >= beta {
                    // Do a reduced-depth verification search when the margin is small
                    if null_score < beta + 100 {
                        match alpha_beta(
                            &mut null_pos,
                            beta - 1,
                            beta,
                            depth - r - 1,
                            state,
                            params,
                            ply + 1,
                        ) {
                            Ok(verify_result) => {
                                if -verify_result.score < beta {
                                    // Verification failed, continue with normal search
                                    // Fall through to regular move generation
                                } else {
                                    return Ok(SearchResult {
                                        principal_variation: None,
                                        score: beta,
                                    });
                                }
                            }
                            Err(e) => return Err(e),
                        }
                    } else {
                        // Don't return mate scores from null move
                        if null_score < 900_000 {
                            return Ok(SearchResult {
                                principal_variation: None,
                                score: beta,
                            });
                        }
                    }
                }
            }
            Err(e) => return Err(e),
        }
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

    let prev_best_move = state.data.previous_pv.as_ref();
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

    let mut results = vec![];

    for (move_index, mv) in ordered_moves.iter().enumerate() {
        let result = test_move(
            *mv,
            position,
            &mut iteration,
            params,
            depth,
            move_index,
            ply,
        );

        results.push(result.clone());

        if let Some(result) = result {
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

    if principal_variation.is_none() {
        panic!(
            "No principal variation found. Alpha: {}, beta: {}, possible moves: {:?}",
            alpha, beta, ordered_moves
        );
    }

    if params.features.enable_transposition_table {
        if let Some(principal_variation) = &principal_variation {
            let (node_type, store_score) = if score >= beta {
                (NodeType::LowerBound, beta)
            } else if score <= original_alpha {
                (NodeType::UpperBound, original_alpha)
            } else {
                (NodeType::Exact, score)
            };

            iteration.state.transposition_table.insert_if_better(
                position.to_hashable(),
                TranspositionTableEntry {
                    depth,
                    score: store_score,
                    principal_variation: principal_variation.clone(),
                    node_type,
                    alpha: original_alpha,
                    beta,
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
    position: &mut Position,
    iteration: &mut SearchIteration,
    params: &SearchParams,
    depth: u32,
    move_index: usize,
    ply: usize,
) -> Option<Result<SearchResult, AlphaBetaError>> {
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

    let restore = position.apply_move(mv).unwrap();

    let in_check = position.is_king_in_check().unwrap();
    position.invert();

    // Implement Late Move Reduction
    let mut score_pv: Option<ScorePV> = if params.features.enable_lmr
        && should_reduce_move(
            &mv,
            depth,
            move_index,
            in_check,
            iteration.alpha,
            iteration.beta,
        ) {
        let reduction = if depth >= 6 {
            // For deeper searches, scale reduction more carefully
            ((depth as f32).ln().floor() as u32)
                .min(move_index as u32 / 2)
                .max(1)
        } else {
            // For shallower searches, use minimal reduction
            1
        };

        if params.debug_print_verbose {
            trace!(
                "{}Reduced search for move: {}",
                "\t".repeat((params.depth - iteration.depth) as usize),
                mv
            );
        }

        // Reduced depth search
        let result = alpha_beta(
            position,
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
            position,
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

        if params.features.enable_history {
            iteration.state.history.update_history(&mv, depth, true);
        }

        if params.features.enable_transposition_table {
            iteration.state.transposition_table.insert(
                position.to_hashable(),
                TranspositionTableEntry {
                    depth,
                    score: iteration.beta,
                    principal_variation: vec![mv],
                    node_type: NodeType::LowerBound,
                    alpha: iteration.alpha,
                    beta: iteration.beta,
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

        position.invert();
        position.unapply_move(mv, restore).unwrap();

        return Some(Ok(SearchResult {
            principal_variation: None,
            score: iteration.beta,
        }));
    } else {
        if params.features.enable_history {
            iteration.state.history.update_history(&mv, depth, false);
        }
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
    } else if iteration.principal_variation.is_none() {
        // The move didn't beta cutoff, and didn't improve alpha, but we don't yet have a
        // principal variation. So this move is equivalent to the best move we've found so far, so
        // store it in the principal variation for this node until we find a better move.
        iteration.principal_variation = Some(vec![mv]);

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

    position.invert();
    position.unapply_move(mv, restore).unwrap();

    None
}

fn should_try_null_move(position: &Position, depth: u32, beta: i32) -> bool {
    // Don't do null move if:
    // 1. In check
    // 2. At shallow depth
    // 3. Side to move has very few pieces (endgame)
    // 4. Previous score indicates zugzwang is likely
    // 5. Beta is close to mate score

    if depth < 3 || position.is_king_in_check().unwrap() || beta > 900_000 || beta < -900_000 {
        return false;
    }

    // Don't do null move if we don't have enough material
    // This helps avoid zugzwang positions
    let side_to_move_material = get_material_count(position, position.true_active_color);
    if side_to_move_material < 3 * piece_value(PieceType::Pawn) {
        return false;
    }

    // Avoid null move in pawn endgames
    let white_has_major_pieces = position
        .white_pieces
        .iter()
        .filter_map(|p| p.as_ref())
        .any(|p| p.piece_type == PieceType::Queen || p.piece_type == PieceType::Rook);
    let black_has_major_pieces = position
        .black_pieces
        .iter()
        .filter_map(|p| p.as_ref())
        .any(|p| p.piece_type == PieceType::Queen || p.piece_type == PieceType::Rook);

    if !white_has_major_pieces || !black_has_major_pieces {
        return false;
    }

    true
}

fn get_material_count(position: &Position, color: Color) -> i32 {
    let mut material = 0;

    let pieces = if color == Color::White {
        &position.white_pieces
    } else {
        &position.black_pieces
    };

    for piece in pieces.iter() {
        if let Some(piece) = piece {
            material += piece_value(piece.piece_type);
        }
    }

    material
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
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .init();

        let position =
            Position::parse_from_fen("Q1Q5/P6k/8/5P2/6Q1/6B1/6PP/R3K2R w kq - 0 1").unwrap();

        let mut transposition_table = TranspositionTable::new();
        let mut state = SearchState::new(&mut transposition_table);

        let params = SearchParams {
            depth: 4,
            game_type: GameType::Classic,
            debug_print_verbose: false,
            ..Default::default()
        };

        let result = search(&position, &mut state, params, 0).unwrap();
        let best_move = result.best_move.unwrap().to_string();

        // White should play Qh8#
        assert_eq!(
            best_move, "Qh8",
            "Expected mate with Qh8, got {}",
            best_move
        );
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

        assert_eq!(
            best_move, expected_move,
            "Expected mate with {}, got {}",
            expected_move, best_move
        );
        assert!(
            result.score >= -CHECKMATE,
            "Expected mate, got score {}",
            result.score
        );
    }

    #[test]
    fn mate_in_2() {
        test_mate(
            "3qr2k/pbpp2pp/1p5N/3Q2b1/2P1P3/P7/1PP2PPP/R4RK1 w - - 0 1",
            "Qg8",
            6,
        );
    }

    #[test]
    fn mate_in_2_2() {
        test_mate(
            "r1bq2k1/ppp2r1p/2np1pNQ/2bNpp2/2B1P3/3P4/PPP2PPP/R3K2R w KQ - 0 1",
            "Nxf6",
            6,
        );
    }

    #[test]
    fn mate_in_2_3() {
        test_mate(
            "r1bk3r/1pp2ppp/pb1p1n2/n2P4/B3P1q1/2Q2N2/PB3PPP/RN3RK1 w - - 0 1",
            "Qxf6",
            6,
        );
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
        test_one_of_best_moves(
            "r4rk1/ppp3pp/3q4/1P4R1/2Pn1n2/2N5/PP3PPP/R1BQ2K1 b - - 0 15",
            &["h6", "Rae8", "Nh3"],
            16,
        );
    }

    #[test]
    fn undermining_1() {
        test_one_of_best_moves(
            "1kr5/3n4/q3p2p/p2n2p1/PppB1P2/5BP1/1P2Q2P/3R2K1 w - - 0 1",
            &["f5"],
            12,
        );
    }
}
