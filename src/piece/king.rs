use crate::{
    bitboard::Bitboard, evaluation::square_bonus::SquareBonus, piece_move::CanMove, Position,
};

use super::{ChessPiece, Color, Piece, PieceType, RescueChessPiece};

pub struct King;

impl RescueChessPiece for King {
    fn can_hold(_other: super::PieceType) -> bool {
        true
    }
}

#[rustfmt::skip]
const KING_MIDDLEGAME_TABLE: [i32; 64] = [
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -10,-20,-20,-20,-20,-20,-20,-10,
     20, 20,  0,  0,  0,  0, 20, 20,
     20, 30, 10,  0,  0, 10, 30, 20
];

impl SquareBonus for King {
    fn square_bonus(pos: crate::Pos) -> i32 {
        KING_MIDDLEGAME_TABLE[pos.0 as usize]
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

impl King {
    pub fn is_in_check(position: &Position) -> bool {
        let king = position.white_king;

        match king {
            Some(king) => {
                // Pawns
                if let Some(pos) = king.position.moved(-1, -1) {
                    if position.is_piece_at(pos, &[PieceType::Pawn], Color::Black) {
                        return true;
                    }
                }

                if let Some(pos) = king.position.moved(1, -1) {
                    if position.is_piece_at(pos, &[PieceType::Pawn], Color::Black) {
                        return true;
                    }
                }

                // Ranks and files
                let mut pos = king.position;

                // Up
                while pos.can_move_up() {
                    pos = pos.moved_unchecked(0, -1);

                    if position.white_map.get(pos) {
                        break;
                    }

                    if position.is_piece_at(pos, &[PieceType::Rook, PieceType::Queen], Color::Black)
                    {
                        return true;
                    }

                    if position.black_map.get(pos) {
                        break;
                    }
                }

                pos = king.position;

                // Down
                while pos.can_move_down() {
                    pos = pos.moved_unchecked(0, 1);

                    if position.white_map.get(pos) {
                        break;
                    }

                    if position.is_piece_at(pos, &[PieceType::Rook, PieceType::Queen], Color::Black)
                    {
                        return true;
                    }

                    if position.black_map.get(pos) {
                        break;
                    }
                }

                pos = king.position;

                // Left
                while pos.can_move_left() {
                    pos = pos.moved_unchecked(-1, 0);

                    if position.white_map.get(pos) {
                        break;
                    }

                    if position.is_piece_at(pos, &[PieceType::Rook, PieceType::Queen], Color::Black)
                    {
                        return true;
                    }

                    if position.black_map.get(pos) {
                        break;
                    }
                }

                pos = king.position;

                // Right
                while pos.can_move_right() {
                    pos = pos.moved_unchecked(1, 0);

                    if position.white_map.get(pos) {
                        break;
                    }

                    if position.is_piece_at(pos, &[PieceType::Rook, PieceType::Queen], Color::Black)
                    {
                        return true;
                    }

                    if position.black_map.get(pos) {
                        break;
                    }
                }

                // Diagonals

                pos = king.position;

                // Up left
                while pos.can_move_up() && pos.can_move_left() {
                    pos = pos.moved_unchecked(-1, -1);

                    if position.white_map.get(pos) {
                        break;
                    }

                    if position.is_piece_at(
                        pos,
                        &[PieceType::Bishop, PieceType::Queen],
                        Color::Black,
                    ) {
                        return true;
                    }

                    if position.black_map.get(pos) {
                        break;
                    }
                }

                pos = king.position;

                // Up right
                while pos.can_move_up() && pos.can_move_right() {
                    pos = pos.moved_unchecked(1, -1);

                    if position.white_map.get(pos) {
                        break;
                    }

                    if position.is_piece_at(
                        pos,
                        &[PieceType::Bishop, PieceType::Queen],
                        Color::Black,
                    ) {
                        return true;
                    }

                    if position.black_map.get(pos) {
                        break;
                    }
                }

                pos = king.position;

                // Down right
                while pos.can_move_down() && pos.can_move_right() {
                    pos = pos.moved_unchecked(1, 1);

                    if position.white_map.get(pos) {
                        break;
                    }

                    if position.is_piece_at(
                        pos,
                        &[PieceType::Bishop, PieceType::Queen],
                        Color::Black,
                    ) {
                        return true;
                    }

                    if position.black_map.get(pos) {
                        break;
                    }
                }

                pos = king.position;

                // Down left
                while pos.can_move_down() && pos.can_move_left() {
                    pos = pos.moved_unchecked(-1, 1);

                    if position.white_map.get(pos) {
                        break;
                    }

                    if position.is_piece_at(
                        pos,
                        &[PieceType::Bishop, PieceType::Queen],
                        Color::Black,
                    ) {
                        return true;
                    }

                    if position.black_map.get(pos) {
                        break;
                    }
                }

                // Knights
                if let Some(pos) = king.position.moved(-1, -2) {
                    if position.is_piece_at(pos, &[PieceType::Knight], Color::Black) {
                        return true;
                    }
                }

                if let Some(pos) = king.position.moved(1, -2) {
                    if position.is_piece_at(pos, &[PieceType::Knight], Color::Black) {
                        return true;
                    }
                }

                if let Some(pos) = king.position.moved(2, -1) {
                    if position.is_piece_at(pos, &[PieceType::Knight], Color::Black) {
                        return true;
                    }
                }

                if let Some(pos) = king.position.moved(2, 1) {
                    if position.is_piece_at(pos, &[PieceType::Knight], Color::Black) {
                        return true;
                    }
                }

                if let Some(pos) = king.position.moved(1, 2) {
                    if position.is_piece_at(pos, &[PieceType::Knight], Color::Black) {
                        return true;
                    }
                }

                if let Some(pos) = king.position.moved(-1, 2) {
                    if position.is_piece_at(pos, &[PieceType::Knight], Color::Black) {
                        return true;
                    }
                }

                if let Some(pos) = king.position.moved(-2, 1) {
                    if position.is_piece_at(pos, &[PieceType::Knight], Color::Black) {
                        return true;
                    }
                }

                if let Some(pos) = king.position.moved(-2, -1) {
                    if position.is_piece_at(pos, &[PieceType::Knight], Color::Black) {
                        return true;
                    }
                }

                // Kings
                if let Some(pos) = king.position.moved(-1, -1) {
                    if position.is_piece_at(pos, &[PieceType::King], Color::Black) {
                        return true;
                    }
                }

                if let Some(pos) = king.position.moved(0, -1) {
                    if position.is_piece_at(pos, &[PieceType::King], Color::Black) {
                        return true;
                    }
                }

                if let Some(pos) = king.position.moved(1, -1) {
                    if position.is_piece_at(pos, &[PieceType::King], Color::Black) {
                        return true;
                    }
                }

                if let Some(pos) = king.position.moved(1, 0) {
                    if position.is_piece_at(pos, &[PieceType::King], Color::Black) {
                        return true;
                    }
                }

                if let Some(pos) = king.position.moved(1, 1) {
                    if position.is_piece_at(pos, &[PieceType::King], Color::Black) {
                        return true;
                    }
                }

                if let Some(pos) = king.position.moved(0, 1) {
                    if position.is_piece_at(pos, &[PieceType::King], Color::Black) {
                        return true;
                    }
                }

                if let Some(pos) = king.position.moved(-1, 1) {
                    if position.is_piece_at(pos, &[PieceType::King], Color::Black) {
                        return true;
                    }
                }

                if let Some(pos) = king.position.moved(-1, 0) {
                    if position.is_piece_at(pos, &[PieceType::King], Color::Black) {
                        return true;
                    }
                }

                false
            }
            None => false,
        }
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
