use tracing::trace;

use crate::{
    evaluation::{evaluate_position, ordering::order_moves},
    piece_move::MoveType,
    Position,
};

use super::{
    alpha_beta::{AlphaBetaError, SearchParams, SearchResult, CHECKMATE},
    search_results::SearchState,
};

pub fn quiescence_search(
    position: &mut Position,
    mut alpha: i32,
    beta: i32,
    depth: u32,
    state: &mut SearchState,
    params: &SearchParams,
    initial_depth: u32,
    ply: usize,
) -> Result<SearchResult, AlphaBetaError> {
    if position.is_checkmate(params.game_type).unwrap() {
        if params.debug_print_verbose {
            trace!(
                "{}[Quiescence] Checkmate found",
                "\t".repeat((initial_depth + (params.quiescence_depth - depth)) as usize)
            );
        }

        return Ok(SearchResult {
            principal_variation: None,
            score: CHECKMATE - (depth as i32),
        });
    }

    // First, do a standing pat evaluation
    let stand_pat = evaluate_position(position, params.game_type, params);

    // Fail-high if standing pat beats beta
    if stand_pat >= beta {
        if params.debug_print_verbose {
            trace!(
                "{}[Quiescence] Standing pat beats beta: {}",
                "\t".repeat((initial_depth + (params.quiescence_depth - depth)) as usize),
                stand_pat
            );
        }

        return Ok(SearchResult {
            principal_variation: Some(vec![]),
            score: beta,
        });
    }

    // Update alpha if standing pat is better
    if stand_pat > alpha {
        if params.debug_print_verbose {
            trace!(
                "{}[Quiescence] Standing pat is better: {}",
                "\t".repeat((initial_depth + (params.quiescence_depth - depth)) as usize),
                stand_pat
            );
        }

        alpha = stand_pat;
    }

    // Stop searching if we've hit maximum quiescence depth
    if depth == 0 {
        if params.debug_print_verbose {
            trace!(
                "{}[Quiescence] Reached maximum depth: {}",
                "\t".repeat((initial_depth + (params.quiescence_depth - depth)) as usize),
                stand_pat
            );
        }

        return Ok(SearchResult {
            principal_variation: Some(vec![]),
            score: stand_pat,
        });
    }

    // Get only capture moves
    let mut moves = position.get_all_legal_moves(params.game_type).unwrap();

    moves.retain(|mv| {
        mv.is_capture()
            || matches!(
                mv.move_type,
                MoveType::Normal {
                    promoted_to: Some(_),
                    ..
                }
            )
    });

    // If no captures are available, return standing pat
    if moves.is_empty() {
        if params.debug_print_verbose {
            trace!(
                "{}[Quiescence] No captures available: {}",
                "\t".repeat((initial_depth + (params.quiescence_depth - depth)) as usize),
                stand_pat
            );
        }

        return Ok(SearchResult {
            principal_variation: Some(vec![]),
            score: stand_pat,
        });
    }

    let mut best_line = None;

    let ordered_moves = order_moves(position, moves, None, state, ply, params);

    // Search capture moves
    for mv in ordered_moves {
        // Apply move
        let restore = position.apply_move(mv).unwrap();
        position.invert();

        if params.debug_print_verbose {
            trace!(
                "{}[Quiescence] Searching move: {}",
                "\t".repeat((initial_depth + (params.quiescence_depth - depth)) as usize),
                mv
            );
        }

        // Recursively search position
        let result = quiescence_search(
            position,
            -beta,
            -alpha,
            depth - 1,
            state,
            params,
            initial_depth,
            ply + 1,
        )?;

        // Unapply move
        position.invert();
        position.unapply_move(mv, restore).unwrap();

        let score = -result.score;

        // Beta cutoff
        if score >= beta {
            state.data.pruned += 1;
            return Ok(SearchResult {
                principal_variation: None,
                score: beta,
            });
        }

        // Update alpha and best line
        if score > alpha {
            alpha = score;
            let mut principal_variation = result.principal_variation.unwrap_or_default();
            principal_variation.insert(0, mv);
            best_line = Some(principal_variation);
        }
    }

    Ok(SearchResult {
        principal_variation: best_line,
        score: alpha,
    })
}
