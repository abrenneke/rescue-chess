use fxhash::FxHashMap;

use crate::{
    evaluation::evaluate_position,
    piece_move::{GameType, MoveType, PieceMove},
    PieceType, Position,
};

pub fn search(position: &Position, depth: u32) -> (Option<PieceMove>, i32) {
    let mut hash_map = FxHashMap::default();

    negamax_hashing(position, depth, &mut hash_map)
}

pub fn negamax_hashing(
    position: &Position,
    depth: u32,
    hash_map: &mut FxHashMap<Position, (Option<PieceMove>, i32)>,
) -> (Option<PieceMove>, i32) {
    if let Some((best_move, score)) = hash_map.get(position) {
        return (*best_move, *score);
    }

    if depth == 0 {
        let result = (None, evaluate_position(position, GameType::Rescue));
        hash_map.insert(position.clone(), result);
        return result;
    }

    let mut max = i32::MIN;
    let mut best_move = None;

    if position.is_checkmate(GameType::Rescue).unwrap() {
        return (None, -1000);
    }

    let moves = position.get_all_legal_moves(GameType::Rescue);

    let moves = match moves {
        Ok(moves) => moves,
        Err(err) => {
            println!("{}", position.to_board_string());
            panic!("Error getting legal moves: {}", err);
        }
    };

    for mv in moves {
        // Apply the move to a clone of the position
        // then switch to other player's perspective
        let mut child = position.clone();
        child.apply_move(mv).unwrap();
        child.invert();

        if let MoveType::Capture {
            captured,
            captured_holding: _,
        } = mv.move_type
        {
            if captured == PieceType::King {
                panic!("Capturing a king");
            }
        }

        let (_, score) = negamax_hashing(&child, depth - 1, hash_map);

        if -score > max {
            max = -score;
            best_move = Some(mv);
        }
    }

    hash_map.insert(position.clone(), (best_move, max));

    (best_move, max)
}
