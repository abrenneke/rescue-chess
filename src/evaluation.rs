use crate::{Color, PieceType, Position};

pub fn evaluate_position(board: &Position) -> i32 {
    let mut score = 0;

    for piece in board.pieces.iter() {
        let value = match piece.piece_type {
            PieceType::Pawn => 10,
            PieceType::Knight => 300,
            PieceType::Bishop => 300,
            PieceType::Rook => 500,
            PieceType::Queen => 900,
            PieceType::King => 20000,
        };

        score += if piece.color == Color::White {
            value
        } else {
            -value
        };

        let holding_value = match piece.holding {
            Some(piece_type) => match piece_type {
                PieceType::Pawn => 100,
                PieceType::Knight => 300,
                PieceType::Bishop => 300,
                PieceType::Rook => 500,
                PieceType::Queen => 900,
                PieceType::King => 20000,
            },
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
