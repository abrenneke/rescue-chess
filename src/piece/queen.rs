use crate::evaluation::square_bonus::SquareBonus;
use crate::piece_move::CanMove;
use crate::Position;
use crate::{bitboard::Bitboard, piece::Piece};

use super::{Bishop, PieceType, Rook};
use super::{ChessPiece, RescueChessPiece};

pub struct Queen;

impl RescueChessPiece for Queen {
    fn can_hold(other: crate::PieceType) -> bool {
        match other {
            PieceType::King => false,
            _ => true,
        }
    }
}

impl ChessPiece for Queen {
    fn piece_type() -> crate::PieceType {
        crate::PieceType::Queen
    }

    fn to_unicode() -> &'static str {
        "♛"
    }
}

#[rustfmt::skip]
const QUEEN_TABLE: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
    -5,   0,  5,  5,  5,  5,  0, -5,
     0,   0,  5,  5,  5,  5,  0, -5,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20
];

impl SquareBonus for Queen {
    fn square_bonus(pos: crate::Pos) -> i32 {
        QUEEN_TABLE[pos.0 as usize]
    }
}

impl CanMove for Queen {
    fn get_legal_moves(piece: &Piece, position: &Position) -> Bitboard {
        Bishop::get_legal_moves(piece, position) | Rook::get_legal_moves(piece, position)
    }
}
