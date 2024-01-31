use crate::bitboard::Bitboard;

use super::Piece;

pub fn get_legal_moves(piece: &Piece, white: Bitboard, _black: Bitboard) -> Bitboard {
    let mut board = Bitboard::new();

    // Up
    if piece.position < 56 && !white.get(piece.position + 8) {
        board.set(piece.position + 8);
    }

    // Up left
    if piece.position < 56 && piece.position % 8 != 0 && !white.get(piece.position + 7) {
        board.set(piece.position + 7);
    }

    // Up right
    if piece.position < 56 && piece.position % 8 != 7 && !white.get(piece.position + 9) {
        board.set(piece.position + 9);
    }

    // Right
    if piece.position % 8 != 7 && !white.get(piece.position + 1) {
        board.set(piece.position + 1);
    }

    // Down
    if piece.position > 7 && !white.get(piece.position - 8) {
        board.set(piece.position - 8);
    }

    // Down right
    if piece.position > 7 && piece.position % 8 != 7 && !white.get(piece.position - 7) {
        board.set(piece.position - 7);
    }

    // Down left
    if piece.position > 7 && piece.position % 8 != 0 && !white.get(piece.position - 9) {
        board.set(piece.position - 9);
    }

    // Left
    if piece.position % 8 != 0 && !white.get(piece.position - 1) {
        board.set(piece.position - 1);
    }

    board
}
