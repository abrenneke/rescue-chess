use crate::{evaluation::evaluate_position, piece_move::PieceMove, Position};

pub fn search(position: &Position, depth: u32) -> (Option<PieceMove>, i32) {}

pub fn alpha_beta(
    position: &Position,
    alpha: i32,
    beta: i32,
    depth: u32,
) -> (Option<PieceMove>, i32) {
    if depth == 0 {
        return (None, evaluate_position(position));
    }
}