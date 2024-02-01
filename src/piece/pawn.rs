use crate::{bitboard::Bitboard, piece::Piece};

// Pawn
pub fn get_legal_moves(piece: &Piece, white: Bitboard, black: Bitboard) -> Bitboard {
    let mut board = Bitboard::new();
    let pos = piece.position;

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

    // TODO En passant

    board & !white
}

#[cfg(test)]
mod tests {
    use crate::{Bitboard, Piece, PieceType};

    #[test]
    pub fn move_from_starting_position() {
        let pawn = Piece::new_white(PieceType::Pawn, (4, 6).into());
        let legal_moves = super::get_legal_moves(&pawn, Default::default(), Default::default());
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

        let legal_moves = super::get_legal_moves(&pawn, white, Default::default());
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

        let legal_moves = super::get_legal_moves(&pawn, Default::default(), black);
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

        let legal_moves = super::get_legal_moves(&pawn, white, Default::default());
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

        let legal_moves = super::get_legal_moves(&pawn, Default::default(), black);
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
        let legal_moves = super::get_legal_moves(&pawn, Default::default(), Default::default());
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
