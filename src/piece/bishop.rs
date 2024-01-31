use crate::{bitboard::Bitboard, piece::Piece};

pub fn get_legal_moves(piece: &Piece, white: Bitboard, black: Bitboard) -> Bitboard {
    let mut board = Bitboard::new();
    let mut position = piece.position;

    // Up left
    while position < 56 && position % 8 != 0 {
        position += 7;
        if white.get(position) {
            break;
        }
        board.set(position);
        if black.get(position) {
            break;
        }
    }

    // Up right
    position = piece.position;
    while position < 56 && position % 8 != 7 {
        position += 9;
        if white.get(position) {
            break;
        }
        board.set(position);
        if black.get(position) {
            break;
        }
    }

    // Down right
    position = piece.position;
    while position > 7 && position % 8 != 7 {
        position -= 7;
        if white.get(position) {
            break;
        }
        board.set(position);
        if black.get(position) {
            break;
        }
    }

    // Down left
    position = piece.position;
    while position > 7 && position % 8 != 0 {
        position -= 9;
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
