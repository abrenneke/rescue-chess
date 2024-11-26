use crate::{Bitboard, Pos};

pub mod bishop;
pub mod king;
pub mod knight;
pub mod pawn;
pub mod queen;
pub mod rook;

use colored::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash, Serialize, Deserialize)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceType {
    pub fn can_hold(&self, other: PieceType) -> bool {
        match self {
            // Right now pieces can rescue pieces of the same type, but maybe
            // pawns couldn't hold other pawns, or pieces couldn't hold same type pieces
            PieceType::Pawn => match other {
                PieceType::Pawn => true,
                _ => false,
            },
            PieceType::Rook => match other {
                PieceType::Pawn | PieceType::Rook | PieceType::Knight | PieceType::Bishop => true,
                _ => false,
            },
            PieceType::Knight => match other {
                PieceType::Pawn | PieceType::Knight | PieceType::Bishop => true,
                _ => false,
            },
            PieceType::Bishop => match other {
                PieceType::Pawn | PieceType::Bishop => true,
                _ => false,
            },
            PieceType::Queen => match other {
                PieceType::King => false,
                _ => true,
            },
            PieceType::King => true,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq, Serialize)]
pub enum Color {
    White,
    Black,
}

#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq, Serialize)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
    pub position: Pos,

    /// For Rescue Chess, the friendly piece that this piece is holding
    pub holding: Option<PieceType>,
}

impl Piece {
    pub fn new(piece_type: PieceType, color: Color, position: Pos) -> Piece {
        Piece {
            piece_type,
            color,
            position,
            holding: None,
        }
    }

    pub fn new_white(piece_type: PieceType, position: Pos) -> Piece {
        Piece {
            piece_type,
            color: Color::White,
            position,
            holding: None,
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
