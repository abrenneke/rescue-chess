use std::sync::LazyLock;

use crate::{
    bitboard::Bitboard, evaluation::square_bonus::SquareBonus, piece::Piece, piece_move::CanMove,
    Pos, Position,
};

use super::{ChessPiece, PieceType, RescueChessPiece};

pub struct Pawn;

impl RescueChessPiece for Pawn {
    fn can_hold(other: super::PieceType) -> bool {
        match other {
            PieceType::Pawn => true,
            _ => false,
        }
    }
}

impl ChessPiece for Pawn {
    fn piece_type() -> crate::PieceType {
        crate::PieceType::Pawn
    }

    fn to_unicode() -> &'static str {
        "♟"
    }
}

static ATTACK_MAPS: LazyLock<[Bitboard; 64]> = LazyLock::new(|| {
    let mut maps = [Bitboard::new(); 64];

    for i in 0..64 {
        let mut board = Bitboard::new();
        let start_pos = crate::Pos(i as u8);

        let mut pos = start_pos;
        if let Some(pos) = pos.moved(-1, -1) {
            board.set(pos);
        }

        pos = start_pos;
        if let Some(pos) = pos.moved(1, -1) {
            board.set(pos);
        }

        maps[i as usize] = board;
    }

    maps
});

static ATTACK_MAPS_BLACK: LazyLock<[Bitboard; 64]> = LazyLock::new(|| {
    let mut maps = [Bitboard::new(); 64];

    for i in 0..64 {
        let mut board = Bitboard::new();
        let start_pos = crate::Pos(i as u8);

        let mut pos = start_pos;
        if let Some(pos) = pos.moved(-1, 1) {
            board.set(pos);
        }

        pos = start_pos;
        if let Some(pos) = pos.moved(1, 1) {
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

#[inline(always)]
pub fn attack_map_black(pos: Pos) -> &'static Bitboard {
    &ATTACK_MAPS_BLACK[pos.0 as usize]
}

#[rustfmt::skip]
const PAWN_TABLE: [i32; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
    5,  5, 10, 25, 25, 10,  5,  5,
    0,  0,  0, 20, 20,  0,  0,  0,
    5, -5,-10,  0,  0,-10, -5,  5,
    5, 10, 10,-20,-20, 10, 10,  5,
    0,  0,  0,  0,  0,  0,  0,  0
];

impl SquareBonus for Pawn {
    fn square_bonus(pos: crate::Pos) -> i32 {
        PAWN_TABLE[pos.0 as usize]
    }
}

impl CanMove for Pawn {
    fn get_legal_moves(piece: &Piece, position: &Position, exclude_white: bool) -> Bitboard {
        let mut board = Bitboard::new();
        let pos = piece.position;

        let white = position.white_map;
        let black = position.black_map;

        if !pos.can_move_up() {
            return board;
        }

        // Single move
        if !black.get(pos.moved_up_unchecked()) {
            board.set(pos.moved_up_unchecked());
        }

        // Double move
        if pos.moved_up_unchecked().can_move_up()
            && !white.get(pos.moved_up_unchecked()) // No white or black directly in front
            && !black.get(pos.moved_up_unchecked())
            && !black.get(pos.moved_unchecked(0, -2))
            && pos.is_row(6)
        {
            board.set(pos.moved_up_unchecked().moved_up_unchecked());
        }

        // Capture left
        if pos.can_move_left() && black.get(pos.moved_unchecked(-1, -1)) {
            board.set(pos.moved_unchecked(-1, -1));
        }

        // Capture right
        if pos.can_move_right() && black.get(pos.moved_unchecked(1, -1)) {
            board.set(pos.moved_unchecked(1, -1));
        }

        // En passant left
        if pos.can_move_left()
            && position.en_passant == Some(pos.moved_unchecked(-1, -1))
            && black.get(pos.moved_unchecked(-1, 0))
        {
            board.set(pos.moved_unchecked(-1, -1));
        }

        // En passant right
        if pos.can_move_right()
            && position.en_passant == Some(pos.moved_unchecked(1, -1))
            && black.get(pos.moved_unchecked(1, 0))
        {
            board.set(pos.moved_unchecked(1, -1));
        }

        if exclude_white {
            board = board & !white;
        }

        board
    }
}

pub fn generate_pawn_attacks(pawn_bitboard: Bitboard) -> Bitboard {
    let mut attacks = Bitboard::new();

    for pos in pawn_bitboard {
        attacks = attacks | ATTACK_MAPS[pos.0 as usize];
    }

    attacks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Bitboard, Piece, PieceType};

    #[test]
    pub fn move_from_starting_position() {
        let pawn = Piece::new_white(PieceType::Pawn, (4, 6).into());
        let legal_moves = Pawn::get_legal_moves(&pawn, &Default::default(), true);
        assert_eq!(
            legal_moves,
            r#"
            00000000
            00000000
            00000000
            00000000
            00001000
            00001000
            00000000
            00000000
        "#
            .parse()
            .unwrap()
        );
    }

    #[test]
    pub fn move_from_starting_position_blocked_white() {
        let pawn = Piece::new_white(PieceType::Pawn, (4, 6).into());

        let white: Bitboard = r#"
            00000000
            00000000
            00000000
            00000000
            00000000
            11111111
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

        let legal_moves = Pawn::get_legal_moves(&pawn, &position, true);
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
        );
    }

    #[test]
    pub fn move_from_starting_position_blocked_black() {
        let pawn = Piece::new_white(PieceType::Pawn, (4, 6).into());

        let black: Bitboard = r#"
            00000000
            00000000
            00000000
            00000000
            00000000
            11111111
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

        let legal_moves = Pawn::get_legal_moves(&pawn, &position, true);
        assert_eq!(
            legal_moves,
            r#"
            00000000
            00000000
            00000000
            00000000
            00000000
            00010100
            00000000
            00000000
        "#
            .parse()
            .unwrap()
        );
    }

    #[test]
    pub fn move_from_starting_position_blocked_white_double() {
        let pawn = Piece::new_white(PieceType::Pawn, (4, 6).into());

        let white: Bitboard = r#"
            00000000
            00000000
            00000000
            00000000
            11111111
            00000000
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

        let legal_moves = Pawn::get_legal_moves(&pawn, &position, true);
        assert_eq!(
            legal_moves,
            r#"
            00000000
            00000000
            00000000
            00000000
            00000000
            00001000
            00000000
            00000000
        "#
            .parse()
            .unwrap()
        );
    }

    #[test]
    pub fn move_from_starting_position_blocked_black_double() {
        let pawn = Piece::new_white(PieceType::Pawn, (4, 6).into());

        let black: Bitboard = r#"
            00000000
            00000000
            00000000
            00000000
            11111111
            00000000
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

        let legal_moves = Pawn::get_legal_moves(&pawn, &position, true);
        assert_eq!(
            legal_moves,
            r#"
            00000000
            00000000
            00000000
            00000000
            00000000
            00001000
            00000000
            00000000
        "#
            .parse()
            .unwrap()
        );
    }

    #[test]
    pub fn cannot_double_move_not_starting_position() {
        let pawn = Piece::new_white(PieceType::Pawn, (4, 5).into());

        let position = Position {
            white_map: Default::default(),
            black_map: Default::default(),
            ..Default::default()
        };

        let legal_moves = Pawn::get_legal_moves(&pawn, &position, true);
        assert_eq!(
            legal_moves,
            r#"
            00000000
            00000000
            00000000
            00000000
            00001000
            00000000
            00000000
            00000000
        "#
            .parse()
            .unwrap()
        );
    }
}
