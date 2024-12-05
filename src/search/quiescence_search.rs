use tracing::trace;

use crate::{
    evaluation::{evaluate_position, piece_value},
    piece_move::MoveType,
    Position,
};

use super::{
    alpha_beta::{Error, SearchParams, SearchResult, CHECKMATE},
    search_results::SearchState,
};

pub fn quiescence_search(
    position: &Position,
    mut alpha: i32,
    beta: i32,
    depth: u32,
    state: &mut SearchState,
    params: &SearchParams,
    initial_depth: u32,
) -> Result<SearchResult, Error> {
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

    moves.sort_by_key(|mv| {
        if let MoveType::Normal {
            captured: Some(captured),
            captured_holding,
            ..
        } = mv.move_type
        {
            // Higher score = better move
            let base = piece_value(captured) * 10 - piece_value(mv.piece_type);

            if let Some(captured_holding) = captured_holding {
                base + piece_value(captured_holding)
            } else {
                base
            }
        } else {
            0
        }
    });
    moves.reverse();

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

    // Search capture moves
    for mv in moves {
        // Apply move
        let mut child = position.clone();
        child.apply_move(mv).unwrap();
        child.invert();

        if params.debug_print_verbose {
            trace!(
                "{}[Quiescence] Searching move: {}",
                "\t".repeat((initial_depth + (params.quiescence_depth - depth)) as usize),
                mv
            );
        }

        // Recursively search position
        let result = quiescence_search(
            &child,
            -beta,
            -alpha,
            depth - 1,
            state,
            params,
            initial_depth,
        )?;

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
