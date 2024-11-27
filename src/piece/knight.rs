use crate::{
    bitboard::Bitboard, evaluation::square_bonus::SquareBonus, piece::Piece, piece_move::CanMove,
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

impl SquareBonus for Knight {
    fn square_bonus(_pos: crate::Pos) -> i32 {
        0
    }
}

impl CanMove for Knight {
    fn get_legal_moves(piece: &Piece, white: Bitboard, _black: Bitboard) -> Bitboard {
        let mut board = Bitboard::new();
        let pos = piece.position;

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
    use crate::{Piece, PieceType};

    #[test]
    pub fn move_empty_spaces() {
        let knight = Piece::new_white(PieceType::Knight, (4, 4).into());

        let legal_moves = knight.get_legal_moves(Default::default(), Default::default());

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

        let legal_moves = knight.get_legal_moves(Default::default(), Default::default());

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
