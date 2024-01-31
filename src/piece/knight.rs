use crate::{bitboard::Bitboard, piece::Piece};

pub fn get_legal_moves(piece: &Piece, white: Bitboard, _black: Bitboard) -> Bitboard {
    let mut board = Bitboard::new();

    // TODO check these

    // Up left
    if piece.position < 56 && piece.position % 8 != 0 && !white.get(piece.position + 7) {
        board.set(piece.position + 7);
    }

    // Up right
    if piece.position < 56 && piece.position % 8 != 7 && !white.get(piece.position + 9) {
        board.set(piece.position + 9);
    }

    // Right up
    if piece.position < 48 && piece.position % 8 < 6 && !white.get(piece.position + 17) {
        board.set(piece.position + 17);
    }

    // Right down
    if piece.position > 15 && piece.position % 8 < 6 && !white.get(piece.position - 15) {
        board.set(piece.position - 15);
    }

    // Down right
    if piece.position > 7 && piece.position % 8 < 7 && !white.get(piece.position - 6) {
        board.set(piece.position - 6);
    }

    // Down left
    if piece.position > 7 && piece.position % 8 > 0 && !white.get(piece.position - 9) {
        board.set(piece.position - 9);
    }

    // Left down
    if piece.position > 15 && piece.position % 8 > 1 && !white.get(piece.position - 17) {
        board.set(piece.position - 17);
    }

    // Left up
    if piece.position < 48 && piece.position % 8 > 1 && !white.get(piece.position + 15) {
        board.set(piece.position + 15);
    }

    board
}
