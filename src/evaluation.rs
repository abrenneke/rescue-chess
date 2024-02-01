use crate::{Color, PieceType, Position};

pub fn evaluate_position(board: &Position) -> i32 {
    let mut score = 0;

    for piece in board.pieces.iter() {
        let value = match piece.piece_type {
            PieceType::Pawn => 1,
            PieceType::Knight => 3,
            PieceType::Bishop => 3,
            PieceType::Rook => 5,
            PieceType::Queen => 9,
            PieceType::King => 200,
        };

        score += if piece.color == Color::White {
            value
        } else {
            -value
        };
    }

    // TODO doubled, blocked, isolated pawns
    // TODO mobility

    score
}
