use crate::{bitboard::Bitboard, piece::Piece};

pub fn get_legal_moves(piece: &Piece, white: Bitboard, black: Bitboard) -> Bitboard {
    let mut board = Bitboard::new();
    let mut pos = piece.position;

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

#[cfg(test)]
mod tests {
    use crate::{
        bitboard::Bitboard,
        piece::{Piece, PieceType},
        pos::Pos,
    };

    #[test]
    fn test_get_legal_moves_empty_1() {
        let bishop = Piece::new_white(PieceType::Bishop, (4, 4).into());
        let legal_moves = bishop.get_legal_moves(0.into(), 0.into());

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
        let legal_moves = bishop.get_legal_moves(0.into(), 0.into());

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

        let legal_moves = bishop.get_legal_moves(white, Default::default());

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

        let legal_moves = bishop.get_legal_moves(Default::default(), black);

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
}
