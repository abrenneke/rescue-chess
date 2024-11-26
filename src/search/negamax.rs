use crate::{
    evaluation::evaluate_position,
    piece_move::{GameType, PieceMove},
    Position,
};

pub fn negamax(position: &Position, depth: u32) -> (Option<PieceMove>, i32) {
    if depth == 0 {
        return (None, evaluate_position(position));
    }

    let mut max = i32::MIN;
    let mut best_move = None;

    if position.is_checkmate(GameType::Rescue).unwrap() {
        return (None, -1000);
    }

    let moves = position.get_all_legal_moves(GameType::Rescue).unwrap();

    for mv in moves {
        // Apply the move to a clone of the position
        // then switch to other player's perspective
        let mut child = position.clone();
        child.apply_move(mv).unwrap();
        child.invert();

        let (_, score) = negamax(&child, depth - 1);

        if -score > max {
            max = -score;
            best_move = Some(mv);
        }
    }

    (best_move, max)
}

#[cfg(test)]
pub mod tests {
    use crate::Position;

    use super::negamax;

    #[test]
    pub fn negamax_1() {
        let position: Position = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR".into();
        let (best_move, score) = negamax(&position, 5);

        println!("{} ({})", best_move.unwrap(), score);
    }
}
