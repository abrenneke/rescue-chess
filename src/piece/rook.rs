use crate::{bitboard::Bitboard, piece::Piece};

pub fn get_legal_moves(piece: &Piece, white: Bitboard, black: Bitboard) -> Bitboard {
    let mut board = Bitboard::new();

    // Up
    let mut position = piece.position;
    while position.can_move_up() {
        position = position.moved_unchecked(0, -1);
        if white.get(position) {
            break;
        }
        board.set(position);
        if black.get(position) {
            break;
        }
    }

    // Down
    position = piece.position;
    while position.can_move_down() {
        position = position.moved_unchecked(0, 1);
        if white.get(position) {
            break;
        }
        board.set(position);
        if black.get(position) {
            break;
        }
    }

    // Left
    position = piece.position;
    while position.can_move_left() {
        position = position.moved_unchecked(-1, 0);
        if white.get(position) {
            break;
        }
        board.set(position);
        if black.get(position) {
            break;
        }
    }

    // Right
    position = piece.position;
    while position.can_move_right() {
        position = position.moved_unchecked(1, 0);
        if white.get(position) {
            break;
        }
        board.set(position);
        if black.get(position) {
            break;
        }
    }

    board
}

#[cfg(test)]
mod tests {
    use crate::{Bitboard, Piece, PieceType};

    #[test]
    fn legal_moves() {
        let piece = Piece::new_white(PieceType::Rook, (4, 4).into());

        let legal_moves = piece.get_legal_moves(Default::default(), Default::default());

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

        let legal_moves = piece.get_legal_moves(white, Default::default());

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

        let legal_moves = piece.get_legal_moves(Default::default(), black);

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
