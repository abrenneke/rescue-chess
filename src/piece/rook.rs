use crate::{bitboard::Bitboard, piece::Piece};

pub fn get_legal_moves(piece: &Piece, white: Bitboard, black: Bitboard) -> Bitboard {
    let mut board = Bitboard::new();

    // Up
    let mut position = piece.position;
    while position < 56 {
        position += 8;
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
    while position > 7 {
        position -= 8;
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
    while position % 8 != 0 {
        position -= 1;
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
    while position % 8 != 7 {
        position += 1;
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
