use crate::{
    evaluation::square_bonus::SquareBonus,
    piece_move::{get_legal_moves, CanMove},
    Bitboard, Pos,
};

pub mod bishop;
pub mod king;
pub mod knight;
pub mod pawn;
pub mod queen;
pub mod rook;

pub use bishop::Bishop;
pub use king::King;
pub use knight::Knight;
pub use pawn::Pawn;
pub use queen::Queen;
pub use rook::Rook;

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

pub trait ChessPiece: CanMove + SquareBonus {
    fn piece_type() -> PieceType;
    fn to_unicode() -> &'static str;
}

pub trait RescueChessPiece: ChessPiece {
    fn can_hold(other: PieceType) -> bool;
}

impl PieceType {
    pub fn can_hold(&self, other: PieceType) -> bool {
        match self {
            // Right now pieces can rescue pieces of the same type, but maybe
            // pawns couldn't hold other pawns, or pieces couldn't hold same type pieces
            PieceType::Pawn => Pawn::can_hold(other),
            PieceType::Rook => Rook::can_hold(other),
            PieceType::Knight => Knight::can_hold(other),
            PieceType::Bishop => Bishop::can_hold(other),
            PieceType::Queen => Queen::can_hold(other),
            PieceType::King => King::can_hold(other),
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
            PieceType::Pawn => get_legal_moves::<Pawn>(self, white, black),
            PieceType::Knight => get_legal_moves::<Knight>(self, white, black),
            PieceType::Bishop => get_legal_moves::<Bishop>(self, white, black),
            PieceType::Rook => get_legal_moves::<Rook>(self, white, black),
            PieceType::Queen => get_legal_moves::<Queen>(self, white, black),
            PieceType::King => get_legal_moves::<King>(self, white, black),
        }
    }

    pub fn to_colored_unicode(&self) -> ColoredString {
        let piece = match self.piece_type {
            PieceType::Pawn => Pawn::to_unicode(),
            PieceType::Knight => Knight::to_unicode(),
            PieceType::Bishop => Bishop::to_unicode(),
            PieceType::Rook => Rook::to_unicode(),
            PieceType::Queen => Queen::to_unicode(),
            PieceType::King => King::to_unicode(),
        };

        match self.color {
            Color::White => piece.yellow(),
            Color::Black => piece.cyan(),
        }
    }

    pub fn square_bonus(&self) -> i32 {
        match self.piece_type {
            PieceType::Pawn => Pawn::square_bonus(self.position),
            PieceType::Knight => Knight::square_bonus(self.position),
            PieceType::Bishop => Bishop::square_bonus(self.position),
            PieceType::Rook => Rook::square_bonus(self.position),
            PieceType::Queen => Queen::square_bonus(self.position),
            PieceType::King => King::square_bonus(self.position),
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
