use std::sync::LazyLock;

use crate::{
    bitboard::Bitboard, evaluation::square_bonus::SquareBonus, piece::Piece, piece_move::CanMove,
    Pos, Position,
};

use super::{ChessPiece, PieceType, RescueChessPiece};

pub struct Knight;

impl RescueChessPiece for Knight {
    fn can_hold(other: super::PieceType) -> bool {
        match other {
            PieceType::Pawn | PieceType::Knight | PieceType::Bishop => true,
            _ => false,
        }
    }
}

impl ChessPiece for Knight {
    fn piece_type() -> crate::PieceType {
        crate::PieceType::Knight
    }

    fn to_unicode() -> &'static str {
        "♞"
    }
}

static ATTACK_MAPS: LazyLock<[Bitboard; 64]> = LazyLock::new(|| {
    let mut maps = [Bitboard::new(); 64];

    for i in 0..64 {
        let mut board = Bitboard::new();
        let start_pos = crate::Pos(i as u8);

        let mut pos = start_pos;
        if let Some(pos) = pos.moved(-1, -2) {
            board.set(pos);
        }

        pos = start_pos;
        if let Some(pos) = pos.moved(1, -2) {
            board.set(pos);
        }

        pos = start_pos;
        if let Some(pos) = pos.moved(2, -1) {
            board.set(pos);
        }

        pos = start_pos;
        if let Some(pos) = pos.moved(2, 1) {
            board.set(pos);
        }

        pos = start_pos;
        if let Some(pos) = pos.moved(1, 2) {
            board.set(pos);
        }

        pos = start_pos;
        if let Some(pos) = pos.moved(-1, 2) {
            board.set(pos);
        }

        pos = start_pos;
        if let Some(pos) = pos.moved(-2, 1) {
            board.set(pos);
        }

        pos = start_pos;
        if let Some(pos) = pos.moved(-2, -1) {
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
const KNIGHT_TABLE: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

impl SquareBonus for Knight {
    fn square_bonus(_pos: crate::Pos) -> i32 {
        KNIGHT_TABLE[_pos.0 as usize]
    }
}

impl CanMove for Knight {
    fn get_legal_moves(piece: &Piece, position: &Position) -> Bitboard {
        let mut board = Bitboard::new();
        let pos = piece.position;

        let white = position.white_map;

        // Up left
        if let Some(pos) = pos.moved(-1, -2) {
            board.set(pos);
        }

        // Up right
        if let Some(pos) = pos.moved(1, -2) {
            board.set(pos);
        }

        // Right up
        if let Some(pos) = pos.moved(2, -1) {
            board.set(pos);
        }

        // Right down
        if let Some(pos) = pos.moved(2, 1) {
            board.set(pos);
        }

        // Down right
        if let Some(pos) = pos.moved(1, 2) {
            board.set(pos);
        }

        // Down left
        if let Some(pos) = pos.moved(-1, 2) {
            board.set(pos);
        }

        // Left down
        if let Some(pos) = pos.moved(-2, 1) {
            board.set(pos);
        }

        // Left up
        if let Some(pos) = pos.moved(-2, -1) {
            board.set(pos);
        }

        board & !white
    }
}

#[cfg(test)]
mod tests {
    use crate::{Piece, PieceType, Position};

    #[test]
    pub fn move_empty_spaces() {
        let knight = Piece::new_white(PieceType::Knight, (4, 4).into());

        let position = Position {
            white_map: Default::default(),
            black_map: Default::default(),
            ..Default::default()
        };

        let legal_moves = knight.get_legal_moves(&position);

        assert_eq!(
            legal_moves,
            r#"
            00000000
            00000000
            00010100
            00100010
            00000000
            00100010
            00010100
            00000000
            "#
            .parse()
            .unwrap()
        );
    }

    #[test]
    pub fn move_wall() {
        let knight = Piece::new_white(PieceType::Knight, (0, 0).into());

        let position = Position {
            white_map: Default::default(),
            black_map: Default::default(),
            ..Default::default()
        };

        let legal_moves = knight.get_legal_moves(&position);

        assert_eq!(
            legal_moves,
            r#"
            00000000
            00100000
            01000000
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
}
