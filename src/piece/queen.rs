pub mod magic;
pub mod occupancy;

use std::sync::LazyLock;

use crate::evaluation::square_bonus::SquareBonus;
use crate::piece_move::CanMove;
use crate::{bitboard::Bitboard, piece::Piece};
use crate::{Pos, Position};

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

static ATTACK_MAPS: LazyLock<[Bitboard; 64]> = LazyLock::new(|| {
    let mut maps = [Bitboard::new(); 64];

    for i in 0..64 {
        let mut board = Bitboard::new();
        let start_pos = crate::Pos(i as u8);

        let mut pos = start_pos;
        while pos.can_move_down() {
            pos = pos.moved_unchecked(0, 1);
            board.set(pos);
        }

        pos = start_pos;
        while pos.can_move_up() {
            pos = pos.moved_unchecked(0, -1);
            board.set(pos);
        }

        pos = start_pos;
        while pos.can_move_left() {
            pos = pos.moved_unchecked(-1, 0);
            board.set(pos);
        }

        pos = start_pos;
        while pos.can_move_right() {
            pos = pos.moved_unchecked(1, 0);
            board.set(pos);
        }

        pos = start_pos;
        while pos.can_move_down() && pos.can_move_left() {
            pos = pos.moved_unchecked(-1, 1);
            board.set(pos);
        }

        pos = start_pos;
        while pos.can_move_down() && pos.can_move_right() {
            pos = pos.moved_unchecked(1, 1);
            board.set(pos);
        }

        pos = start_pos;
        while pos.can_move_up() && pos.can_move_left() {
            pos = pos.moved_unchecked(-1, -1);
            board.set(pos);
        }

        pos = start_pos;
        while pos.can_move_up() && pos.can_move_right() {
            pos = pos.moved_unchecked(1, -1);
            board.set(pos);
        }

        maps[i] = board;
    }

    maps
});

#[inline(always)]
pub fn attack_map(pos: Pos) -> &'static Bitboard {
    &ATTACK_MAPS[pos.0 as usize]
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
