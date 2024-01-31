use crate::{bitboard::Bitboard, piece::Piece};

// Pawn
pub fn get_legal_moves(piece: &Piece, white: Bitboard, black: Bitboard) -> Bitboard {
    let mut board = Bitboard::new();

    if piece.position > 55 {
        return board;
    }

    // Single move
    if !white.get(piece.position + 8) && !black.get(piece.position + 8) {
        board.set(piece.position + 8);
    }

    // Double move
    if piece.position < 16 && !white.get(piece.position + 16) && !black.get(piece.position + 16) {
        board.set(piece.position + 16);
    }

    // Capture left
    if black.get(piece.position + 7) && piece.position % 8 != 0 {
        board.set(piece.position + 7);
    }

    // Capture right
    if black.get(piece.position + 9) && piece.position % 8 != 7 {
        board.set(piece.position + 9);
    }

    // TODO En passant

    board
}
