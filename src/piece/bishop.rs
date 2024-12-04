use std::sync::LazyLock;

use crate::{
    bitboard::Bitboard, evaluation::square_bonus::SquareBonus, piece::Piece, piece_move::CanMove,
    Pos, Position,
};

use super::{ChessPiece, PieceType, RescueChessPiece};

pub struct Bishop;

impl RescueChessPiece for Bishop {
    fn can_hold(other: PieceType) -> bool {
        match other {
            PieceType::Pawn | PieceType::Bishop => true,
            _ => false,
        }
    }
}

impl ChessPiece for Bishop {
    fn piece_type() -> crate::PieceType {
        crate::PieceType::Bishop
    }

    fn to_unicode() -> &'static str {
        "♝"
    }
}

static ATTACK_MAPS: LazyLock<[Bitboard; 64]> = LazyLock::new(|| {
    let mut maps = [Bitboard::new(); 64];

    for i in 0..64 {
        let mut board = Bitboard::new();
        let start_pos = Pos(i as u8);

        let mut pos = start_pos;
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

pub fn attack_map(pos: Pos) -> &'static Bitboard {
    &ATTACK_MAPS[pos.0 as usize]
}

#[rustfmt::skip]
const BISHOP_TABLE: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

impl SquareBonus for Bishop {
    fn square_bonus(pos: crate::Pos) -> i32 {
        BISHOP_TABLE[pos.0 as usize]
    }
}

impl CanMove for Bishop {
    fn get_legal_moves(piece: &Piece, position: &Position) -> Bitboard {
        let mut board = Bitboard::new();
        let mut pos = piece.position;

        let white = position.white_map;
        let black = position.black_map;

        // Down Left
        while pos.can_move_down() && pos.can_move_left() {
            pos = pos.moved_unchecked(-1, 1);
            if white.get(pos) {
                break;
            }
            board.set(pos);
            if black.get(pos) {
                break;
            }
        }

        // Down Right
        pos = piece.position;
        while pos.can_move_down() && pos.can_move_right() {
            pos = pos.moved_unchecked(1, 1);
            if white.get(pos) {
                break;
            }
            board.set(pos);
            if black.get(pos) {
                break;
            }
        }

        // Down right
        pos = piece.position;
        while pos.can_move_up() && pos.can_move_left() {
            pos = pos.moved_unchecked(-1, -1);
            if white.get(pos) {
                break;
            }
            board.set(pos);
            if black.get(pos) {
                break;
            }
        }

        // Top left
        pos = piece.position;
        while pos.can_move_up() && pos.can_move_right() {
            pos = pos.moved_unchecked(1, -1);
            if white.get(pos) {
                break;
            }
            board.set(pos);
            if black.get(pos) {
                break;
            }
        }

        board
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        bitboard::Bitboard,
        piece::{Piece, PieceType},
        pos::Pos,
        Position,
    };

    #[test]
    fn test_get_legal_moves_empty_1() {
        let bishop = Piece::new_white(PieceType::Bishop, (4, 4).into());

        let position = Position {
            white_map: Default::default(),
            black_map: Default::default(),
            ..Default::default()
        };

        let legal_moves = bishop.get_legal_moves(&position);

        assert_eq!(
            legal_moves,
            r#"
                10000000
                01000001
                00100010
                00010100
                00000000
                00010100
                00100010
                01000001
            "#
            .parse()
            .unwrap(),
        )
    }

    #[test]
    fn test_get_legal_moves_empty_2() {
        let bishop = Piece::new_white(PieceType::Bishop, Pos::top_left());

        let position = Position {
            white_map: Default::default(),
            black_map: Default::default(),
            ..Default::default()
        };

        let legal_moves = bishop.get_legal_moves(&position);

        assert_eq!(
            legal_moves,
            r#"
                00000000
                01000000
                00100000
                00010000
                00001000
                00000100
                00000010
                00000001
                "#
            .parse()
            .unwrap()
        );
    }

    #[test]
    fn test_get_legal_moves_blocked_white_up() {
        let bishop = Piece::new_white(PieceType::Bishop, (4, 4).into());
        let white: Bitboard = r#"
            00000000
            00000000
            00000000
            00010100
            00000000
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

        let legal_moves = bishop.get_legal_moves(&position);

        assert_eq!(
            legal_moves,
            r#"
                00000000
                00000000
                00000000
                00000000
                00000000
                00010100
                00100010
                01000001
            "#
            .parse()
            .unwrap(),
        )
    }

    #[test]
    fn test_get_legal_moves_with_black_top() {
        let bishop = Piece::new_white(PieceType::Bishop, (4, 4).into());
        let black: Bitboard = r#"
            00000000
            00000000
            00000000
            00010100
            00000000
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

        let legal_moves = bishop.get_legal_moves(&position);

        assert_eq!(
            legal_moves,
            r#"
                00000000
                00000000
                00000000
                00010100
                00000000
                00010100
                00100010
                01000001
            "#
            .parse()
            .unwrap(),
        )
    }

    #[test]
    fn square_bonus() {
        let bishop = Piece::new_white(PieceType::Bishop, (3, 3).into());
        assert_eq!(bishop.square_bonus(), 10);

        let bishop = Piece::new_white(PieceType::Bishop, Pos::from_algebraic("c5").unwrap());
        assert_eq!(bishop.square_bonus(), 10);
    }
}
