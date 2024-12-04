pub mod magic;
pub mod occupancy;

use std::sync::LazyLock;

use crate::{
    bitboard::Bitboard, evaluation::square_bonus::SquareBonus, piece::Piece, piece_move::CanMove,
    Pos, Position,
};

use super::{ChessPiece, PieceType, RescueChessPiece};

pub struct Rook;

impl RescueChessPiece for Rook {
    fn can_hold(other: crate::PieceType) -> bool {
        match other {
            PieceType::Pawn | PieceType::Rook | PieceType::Knight | PieceType::Bishop => true,
            _ => false,
        }
    }
}

impl ChessPiece for Rook {
    fn piece_type() -> crate::PieceType {
        crate::PieceType::Rook
    }

    fn to_unicode() -> &'static str {
        "♜"
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

        maps[i as usize] = board;
    }

    maps
});

#[inline(always)]
pub fn attack_map(pos: Pos) -> &'static Bitboard {
    &ATTACK_MAPS[pos.0 as usize]
}

#[rustfmt::skip]
const ROOK_TABLE: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
     5, 10, 10, 10, 10, 10, 10,  5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
     0,  0,  0,  5,  5,  0,  0,  0
];

impl SquareBonus for Rook {
    fn square_bonus(pos: crate::Pos) -> i32 {
        ROOK_TABLE[pos.0 as usize]
    }
}

impl CanMove for Rook {
    fn get_legal_moves(piece: &Piece, position: &Position) -> Bitboard {
        let moves = magic::get_rook_moves_magic(piece.position, position.all_map);
        moves & !position.white_map
    }
}
#[cfg(test)]
mod tests {
    use crate::{Bitboard, Piece, PieceType, Position};

    #[test]
    fn legal_moves() {
        let piece = Piece::new_white(PieceType::Rook, (4, 4).into());

        let position = Position {
            white_map: Default::default(),
            black_map: Default::default(),
            ..Default::default()
        };

        let legal_moves = piece.get_legal_moves(&position);

        assert_eq!(
            legal_moves,
            r#"
            00001000
            00001000
            00001000
            00001000
            11110111
            00001000
            00001000
            00001000
        "#
            .parse()
            .unwrap()
        )
    }

    #[test]
    fn blocked_white() {
        let piece = Piece::new_white(PieceType::Rook, (4, 4).into());

        let white: Bitboard = r#"
            00000000
            00000000
            00000000
            00001000
            00010100
            00001000
            00000000
            00000000
        "#
        .parse()
        .unwrap();

        let position = Position {
            white_map: white,
            black_map: Default::default(),
            ..Default::default()
        };

        let legal_moves = piece.get_legal_moves(&position);

        assert_eq!(
            legal_moves,
            r#"
            00000000
            00000000
            00000000
            00000000
            00000000
            00000000
            00000000
            00000000
        "#
            .parse()
            .unwrap()
        )
    }

    #[test]
    fn blocked_black() {
        let piece = Piece::new_white(PieceType::Rook, (4, 4).into());

        let black: Bitboard = r#"
            00000000
            00000000
            00000000
            00001000
            00010100
            00001000
            00000000
            00000000
        "#
        .parse()
        .unwrap();

        let position = Position {
            white_map: Default::default(),
            black_map: black,
            ..Default::default()
        };

        let legal_moves = piece.get_legal_moves(&position);

        assert_eq!(
            legal_moves,
            r#"
            00000000
            00000000
            00000000
            00001000
            00010100
            00001000
            00000000
            00000000
        "#
            .parse()
            .unwrap()
        )
    }
}
