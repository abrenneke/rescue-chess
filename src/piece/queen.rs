use crate::evaluation::square_bonus::SquareBonus;
use crate::piece_move::CanMove;
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

impl SquareBonus for Queen {
    fn square_bonus(_pos: crate::Pos) -> i32 {
        0
    }
}

impl CanMove for Queen {
    fn get_legal_moves(piece: &Piece, white: Bitboard, black: Bitboard) -> Bitboard {
        Bishop::get_legal_moves(piece, white, black) | Rook::get_legal_moves(piece, white, black)
    }
}
