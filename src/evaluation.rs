pub mod square_bonus;

use crate::{piece_move::MoveType, Color, PieceMove, PieceType, Position};

pub fn piece_value(piece_type: PieceType) -> i32 {
    match piece_type {
        PieceType::Pawn => 10,
        PieceType::Knight => 300,
        PieceType::Bishop => 300,
        PieceType::Rook => 500,
        PieceType::Queen => 900,
        PieceType::King => 20000,
    }
}

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
) -> Vec<PieceMove> {
    let mut scored_moves: Vec<ScoredMove> = moves
        .into_iter()
        .map(|mv| {
            let score = score_move(position, &mv, prev_best_move);
            ScoredMove { score, mv }
        })
        .collect();

    // Sort in descending order (highest score first)
    scored_moves.sort_by(|a, b| b.score.cmp(&a.score));

    scored_moves.into_iter().map(|sm| sm.mv).collect()
}

fn score_move(_position: &Position, mv: &PieceMove, prev_best_move: Option<PieceMove>) -> i32 {
    let mut score = 0;

    // 1. Hash table move from previous iteration
    if prev_best_move.is_some() && *mv == prev_best_move.unwrap() {
        return 20000; // Highest priority
    }

    // 2. Captures, scored by MVV-LVA (Most Valuable Victim - Least Valuable Aggressor)
    if mv.is_capture() {
        // Base capture score
        score += 10000;

        // Add MVV-LVA scoring
        // Victim value - try to capture most valuable pieces first

        if let MoveType::Capture(captured_piece) = mv.move_type {
            score += piece_value(captured_piece) * 100;
        }

        // Subtract attacker value - prefer capturing with less valuable pieces
        score -= piece_value(mv.piece_type) * 10;
    }

    // 3. Killer moves (moves that caused beta cutoffs at the same ply in other branches)
    // This would require storing killer moves in the search state
    // score += check_if_killer_move(mv) * 9000;

    // 4. Special moves
    if let MoveType::Promotion(_) = mv.move_type {
        score += 8000;
    }

    // if mv.is_check() {
    //     score += 7000;
    // }

    // 5. Piece-square table bonuses
    // score += get_piece_square_bonus(mv.piece(), mv.to_square());

    // 6. History heuristic (moves that were good in earlier positions)
    // This would require maintaining a history table
    // score += get_history_score(mv);

    score
}

pub fn evaluate_position(board: &Position) -> i32 {
    let mut score = 0;

    for piece in board.pieces.iter() {
        let value = piece_value(piece.piece_type);

        score += if piece.color == Color::White {
            value
        } else {
            -value
        };

        let holding_value = match piece.holding {
            Some(piece_type) => piece_value(piece_type),
            None => 0,
        };

        score += if piece.color == Color::White {
            holding_value
        } else {
            -holding_value
        };
    }

    // TODO doubled, blocked, isolated pawns
    // TODO mobility

    score
}
