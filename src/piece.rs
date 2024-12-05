use crate::{evaluation::square_bonus::SquareBonus, piece_move::CanMove, Bitboard, Pos, Position};

pub mod bishop;
pub mod king;
pub mod knight;
pub mod occupancy;
pub mod pawn;
pub mod queen;
pub mod rescue_drop;
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

impl std::fmt::Display for PieceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_algebraic(Color::White))
    }
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

impl Color {
    pub fn invert(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

pub static PAWN_PROMOTION_TYPES: [PieceType; 4] = [
    PieceType::Queen,
    PieceType::Rook,
    PieceType::Bishop,
    PieceType::Knight,
];

#[derive(Debug, PartialEq, Clone, Hash, Eq, Serialize)]
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

    pub fn get_legal_moves(&self, position: &Position) -> Bitboard {
        match self.piece_type {
            PieceType::Pawn => Pawn::get_legal_moves(self, position),
            PieceType::Knight => Knight::get_legal_moves(self, position),
            PieceType::Bishop => Bishop::get_legal_moves(self, position),
            PieceType::Rook => Rook::get_legal_moves(self, position),
            PieceType::Queen => Queen::get_legal_moves(self, position),
            PieceType::King => King::get_legal_moves(self, position),
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

    #[inline(always)]
    pub fn get_attack_map(&self) -> Bitboard {
        match self.piece_type {
            PieceType::Pawn => *pawn::attack_map(self.position),
            PieceType::Knight => *knight::attack_map(self.position),
            PieceType::Bishop => *bishop::attack_map(self.position),
            PieceType::Rook => *rook::attack_map(self.position),
            PieceType::Queen => *queen::attack_map(self.position),
            PieceType::King => *king::attack_map(self.position),
        }
    }
}

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.piece_type.to_algebraic(self.color))
    }
}

impl PieceType {
    pub fn to_algebraic(&self, color: Color) -> &'static str {
        match color {
            Color::White => match self {
                PieceType::Pawn => "P",
                PieceType::Knight => "N",
                PieceType::Bishop => "B",
                PieceType::Rook => "R",
                PieceType::Queen => "Q",
                PieceType::King => "K",
            },
            Color::Black => match self {
                PieceType::Pawn => "p",
                PieceType::Knight => "n",
                PieceType::Bishop => "b",
                PieceType::Rook => "r",
                PieceType::Queen => "q",
                PieceType::King => "k",
            },
        }
    }
}
