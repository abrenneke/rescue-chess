use crate::{Bitboard, Pos};

pub mod bishop;
pub mod king;
pub mod knight;
pub mod pawn;
pub mod queen;
pub mod rook;

use colored::*;

#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq)]
pub enum Color {
    White,
    Black,
}

#[derive(PartialEq, Copy, Clone, Hash, Eq)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
    pub position: Pos,
}

impl Piece {
    pub fn new(piece_type: PieceType, color: Color, position: Pos) -> Piece {
        Piece {
            piece_type,
            color,
            position,
        }
    }

    pub fn new_white(piece_type: PieceType, position: Pos) -> Piece {
        Piece {
            piece_type,
            color: Color::White,
            position,
        }
    }

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

    pub fn to_colored_unicode(&self) -> ColoredString {
        let piece = match self.piece_type {
            PieceType::Pawn => "♟",
            PieceType::Knight => "♞",
            PieceType::Bishop => "♝",
            PieceType::Rook => "♜",
            PieceType::Queen => "♛",
            PieceType::King => "♚",
        };

        match self.color {
            Color::White => piece.yellow(),
            Color::Black => piece.cyan(),
        }
    }
}

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.color == Color::White {
            match self.piece_type {
                PieceType::Pawn => write!(f, "P"),
                PieceType::Knight => write!(f, "N"),
                PieceType::Bishop => write!(f, "B"),
                PieceType::Rook => write!(f, "R"),
                PieceType::Queen => write!(f, "Q"),
                PieceType::King => write!(f, "K"),
            }
        } else {
            match self.piece_type {
                PieceType::Pawn => write!(f, "p"),
                PieceType::Knight => write!(f, "n"),
                PieceType::Bishop => write!(f, "b"),
                PieceType::Rook => write!(f, "r"),
                PieceType::Queen => write!(f, "q"),
                PieceType::King => write!(f, "k"),
            }
        }
    }
}
