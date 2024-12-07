use crate::{
    piece::{knight, pawn},
    piece_move::MoveType,
    search::{alpha_beta::SearchParams, search_results::SearchState},
    PieceMove, PieceType, Position,
};

use super::piece_value;

#[derive(PartialEq, Eq)]
struct ScoredMove {
    score: i32,
    mv: PieceMove,
}

impl PartialOrd for ScoredMove {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.score.cmp(&other.score))
    }
}

impl Ord for ScoredMove {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score.cmp(&other.score)
    }
}

pub fn order_moves(
    position: &mut Position,
    moves: Vec<PieceMove>,
    prev_pv: Option<&Vec<PieceMove>>,
    state: &SearchState,
    ply: usize,
    params: &SearchParams,
) -> Vec<PieceMove> {
    let mut scored_moves: Vec<ScoredMove> = moves
        .into_iter()
        .map(|mv| {
            let prev_best_move = prev_pv.as_ref().and_then(|pv| pv.get(ply)).cloned();

            let score = score_move(position, &mv, prev_best_move, state, ply, params);
            ScoredMove { score, mv }
        })
        .collect();

    // Sort in descending order (highest score first)
    scored_moves.sort_by(|a, b| b.score.cmp(&a.score));

    scored_moves.into_iter().map(|sm| sm.mv).collect()
}

fn score_move(
    position: &mut Position,
    mv: &PieceMove,
    prev_best_move: Option<PieceMove>,
    state: &SearchState,
    ply: usize,
    params: &SearchParams,
) -> i32 {
    let mut score = 0;

    // 1. Hash table move from previous iteration
    if prev_best_move.is_some() && *mv == prev_best_move.unwrap() {
        return 20000; // Highest priority
    }

    // Check moves should be prioritized
    {
        let restore = position.apply_move(mv.clone()).unwrap();

        if position.is_black_king_in_check().unwrap() {
            // position.invert();
            score += 25_000;

            // Slow
            // let escape_moves = position.get_all_legal_moves(params.game_type).unwrap();
            // let escape_moves = position.count_pseudolegal_moves();
            // if escape_moves.len() <= 2 {
            //     score += 15_000;
            // }
            // position.invert();
        }

        position.unapply_move(mv.clone(), restore).unwrap();
    }

    // 2. Captures, scored by MVV-LVA (Most Valuable Victim - Least Valuable Aggressor)
    if mv.is_capture() {
        // Base capture score
        score += 10000;

        // Add MVV-LVA scoring
        // Victim value - try to capture most valuable pieces first

        if let MoveType::Normal {
            captured: Some(captured),
            captured_holding,
            ..
        } = mv.move_type
        {
            score += piece_value(captured) * 100;

            if let Some(captured_holding) = captured_holding {
                score += piece_value(captured_holding) * 100;
            }
        }

        // Subtract attacker value - prefer capturing with less valuable pieces
        score -= piece_value(mv.piece_type) * 10;
    }

    if params.features.enable_killer_moves {
        let killers = state.killer_moves.get_killers(ply);
        if killers[0].as_ref() == Some(mv) {
            return 19000; // First killer move
        }
        if killers[1].as_ref() == Some(mv) {
            return 18000; // Second killer move
        }
    }

    // Central pawn pushes in opening/middlegame
    if mv.piece_type == PieceType::Pawn {
        let to_col = mv.to.get_col();
        let to_row = mv.to.get_row();
        if (to_col == 3 || to_col == 4) && (to_row == 3 || to_row == 4) {
            score += 6000; // High but below captures
        }
    }

    // 4. Special moves
    if let MoveType::Normal {
        promoted_to: Some(_),
        ..
    } = mv.move_type
    {
        score += 15000;
    }

    if params.features.enable_history {
        score += state.history.get_history_score(mv);
    }

    score += quick_threat_score(position, mv);

    score
}

fn quick_threat_score(position: &Position, mv: &PieceMove) -> i32 {
    let mut score: i32 = 0;

    let maps = position.get_piece_maps();

    // Knight fork patterns - looking for moves that put knight at fork distances from multiple pieces
    if mv.piece_type == PieceType::Knight {
        let valuable_targets =
            maps.black_queens | maps.black_rooks | maps.black_bishops | maps.black_knights;
        let attacked_valuable_targets = *knight::attack_map(mv.to) & valuable_targets;

        if attacked_valuable_targets.count() >= 2 {
            score += 7000;
        }
    }

    // Pawn advances that threaten pieces
    if mv.piece_type == PieceType::Pawn {
        let attacked_pieces = *pawn::attack_map(mv.to) & position.black_map;

        for pos in attacked_pieces {
            let piece = position.get_piece_at(pos).unwrap();
            score += piece_value(piece.piece_type) * 100;
        }
    }

    score
}
