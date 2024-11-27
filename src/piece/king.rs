use crate::{bitboard::Bitboard, evaluation::square_bonus::SquareBonus, piece_move::CanMove};

use super::{ChessPiece, Piece, RescueChessPiece};

pub struct King;

impl RescueChessPiece for King {
    fn can_hold(_other: super::PieceType) -> bool {
        true
    }
}

impl SquareBonus for King {
    fn square_bonus(_pos: crate::Pos) -> i32 {
        0
    }
}

impl ChessPiece for King {
    fn piece_type() -> crate::PieceType {
        crate::PieceType::King
    }

    fn to_unicode() -> &'static str {
        "♚"
    }
}

impl CanMove for King {
    fn get_legal_moves(piece: &Piece, white: Bitboard, _black: Bitboard) -> Bitboard {
        let mut board = Bitboard::new();
        let pos = piece.position;

        // Up left
        if pos.can_move_left() && pos.can_move_up() {
            board.set(pos.moved_unchecked(-1, -1));
        }

        // Up
        if pos.can_move_up() {
            board.set(pos.moved_unchecked(0, -1));
        }

        // Up right
        if pos.can_move_right() && pos.can_move_up() {
            board.set(pos.moved_unchecked(1, -1));
        }

        // Right
        if pos.can_move_right() {
            board.set(pos.moved_unchecked(1, 0));
        }

        // Down right
        if pos.can_move_down() && pos.can_move_right() {
            board.set(pos.moved_unchecked(1, 1));
        }

        // Down
        if pos.can_move_down() {
            board.set(pos.moved_unchecked(0, 1));
        }

        // Down left
        if pos.can_move_left() && pos.can_move_down() {
            board.set(pos.moved_unchecked(-1, 1));
        }

        // Left
        if pos.can_move_left() {
            board.set(pos.moved_unchecked(-1, 0));
        }

        board & !white
    }
}

#[cfg(test)]
mod tests {
    use crate::{Piece, PieceType};

    #[test]
    pub fn move_king_empty_spaces() {
        let king = Piece::new_white(PieceType::King, (4, 4).into());

        let legal_moves = king.get_legal_moves(Default::default(), Default::default());

        assert_eq!(
            legal_moves,
            r#"
            00000000
            00000000
            00000000
            00011100
            00010100
            00011100
            00000000
            00000000
        "#
            .parse()
            .unwrap()
        );
    }
}
