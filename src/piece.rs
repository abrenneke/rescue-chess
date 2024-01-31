use crate::{bitboard::Bitboard, board::Board};

pub mod bishop;
pub mod king;
pub mod knight;
pub mod pawn;
pub mod queen;
pub mod rook;

#[derive(PartialEq, Copy, Clone)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(PartialEq, Copy, Clone)]
pub enum Color {
    White,
    Black,
}

pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
    pub position: u8,
}

impl Piece {
    pub fn get_legal_moves(&self, white: Bitboard, black: Bitboard) -> Bitboard {
        match self.piece_type {
            PieceType::Pawn => pawn::get_legal_moves(self, white, black),
            PieceType::Knight => knight::get_legal_moves(self, white, black),
            PieceType::Bishop => bishop::get_legal_moves(self, white, black),
            PieceType::Rook => rook::get_legal_moves(self, white, black),
            PieceType::Queen => queen::get_legal_moves(self, white, black),
            PieceType::King => king::get_legal_moves(self, white, black),
        }
    }
}
