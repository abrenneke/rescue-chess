use crate::{
    piece_move::MoveType,
    search::{alpha_beta::SearchParams, search_results::SearchState},
    Color, PieceMove, PieceType, Position,
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
    position: &Position,
    moves: Vec<PieceMove>,
    prev_best_move: Option<PieceMove>,
    state: &SearchState,
    ply: usize,
    params: &SearchParams,
) -> Vec<PieceMove> {
    let mut scored_moves: Vec<ScoredMove> = moves
        .into_iter()
        .map(|mv| {
            let score = score_move(position, &mv, prev_best_move, state, ply, params);
            ScoredMove { score, mv }
        })
        .collect();

    // Sort in descending order (highest score first)
    scored_moves.sort_by(|a, b| b.score.cmp(&a.score));

    scored_moves.into_iter().map(|sm| sm.mv).collect()
}

fn score_move(
    position: &Position,
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
        let mut position = position.clone();
        position.apply_move(mv.clone()).unwrap();
        position.invert();

        if position.is_king_in_check().unwrap() {
            score += 25_000;

            let escape_moves = position.get_all_legal_moves(params.game_type).unwrap();
            // let escape_moves = position.count_pseudolegal_moves();
            if escape_moves.len() <= 2 {
                score += 15_000;
            }
        }
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

    score += score_tactical_patterns(position, mv);

    if params.features.enable_history {
        score += state.history.get_history_score(mv);
    }

    score
}

fn score_tactical_patterns(position: &Position, mv: &PieceMove) -> i32 {
    let mut score = 0;

    // 1. Undermining (your existing effective pattern)
    if mv.piece_type == PieceType::Pawn {
        score += score_undermining(position, mv);
    }

    score
}

#[allow(dead_code)]
fn score_undermining(position: &Position, mv: &PieceMove) -> i32 {
    // Only consider pawn moves
    if mv.piece_type != PieceType::Pawn {
        return 0;
    }

    let mut score = 0;

    // Look for defended pieces that would become undefended
    // if a defending pawn has to move
    let to_pos = mv.to;

    // If we're attacking a black pawn...
    if let Some(target) = position.get_piece_at(to_pos) {
        if target.piece_type == PieceType::Pawn && target.color == Color::Black {
            // Look for pieces this pawn is defending
            for piece in position.black_pieces.iter() {
                if let Some(piece) = piece {
                    if piece.piece_type == PieceType::Pawn {
                        continue;
                    }

                    // Check if target pawn is defending this piece
                    let defending_distance =
                        (target.position.get_col() as i32 - piece.position.get_col() as i32).abs();
                    if defending_distance <= 1 {
                        // Check if piece has other defenders
                        let mut other_defenders = 0;
                        for defender in position.black_pieces.iter() {
                            if let Some(defender) = defender {
                                if defender != target {
                                    // Don't count the targeted pawn
                                    let moves = defender.get_legal_moves(position);
                                    if moves.into_iter().any(|m| m == piece.position) {
                                        other_defenders += 1;
                                    }
                                }
                            }
                        }

                        if other_defenders == 0 {
                            // Big bonus! This move would force the pawn to move/capture
                            // and leave a piece undefended
                            score += 8000 + piece_value(piece.piece_type) / 2;
                        }
                    }
                }
            }
        }
    }

    score
}
