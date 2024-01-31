use crate::{bitboard::Bitboard, piece::Piece};

use super::bishop;
use super::rook;

pub fn get_legal_moves(piece: &Piece, white: Bitboard, black: Bitboard) -> Bitboard {
    bishop::get_legal_moves(piece, white, black) | rook::get_legal_moves(piece, white, black)
}
