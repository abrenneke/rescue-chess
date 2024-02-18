use std::{ops::Index, slice::SliceIndex};

use rand::seq::SliceRandom;
use serde::Serialize;

use crate::{position::print_positions, PieceMove, Position};

/// A list of moves, with a score. The moves alternate between the two players,
/// and are the move from that player's perspective.
#[derive(Clone, Debug, Serialize)]
pub struct Variation {
    pub moves: Vec<PieceMove>,
}

impl Variation {
    pub fn new() -> Variation {
        Variation { moves: Vec::new() }
    }

    pub fn new_from(mv: PieceMove) -> Variation {
        Variation { moves: vec![mv] }
    }

    pub fn prepend_move(&mut self, mv: PieceMove) {
        self.moves.insert(0, mv);
    }

    pub fn next_move(&self) -> PieceMove {
        self.moves.first().expect("No moves in variation").clone()
    }

    pub fn print_as_positions(&self, start_position: &Position) -> String {
        let mut position = start_position.clone();
        let mut other_player = false;

        let mut positions = Vec::new();
        positions.push(position.clone());

        for mv in &self.moves {
            position.apply_move(mv);

            if (other_player) {
                positions.push(position.inverted());
            } else {
                positions.push(position.clone());
            }

            position.invert();
            other_player = !other_player;
        }

        print_positions(&positions)
    }
}

impl std::fmt::Display for Variation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut moves = self.moves.iter().peekable();
        while let Some(mv) = moves.next() {
            write!(f, "{}", mv)?;
            if moves.peek().is_some() {
                write!(f, " ")?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Variations {
    pub variations: Vec<Variation>,
}

impl Variations {
    pub fn new() -> Variations {
        Variations {
            variations: Vec::new(),
        }
    }

    pub fn new_from(variation: Variation) -> Variations {
        Variations {
            variations: vec![variation],
        }
    }

    pub fn push(&mut self, variation: Variation) {
        self.variations.push(variation);
    }
}

impl std::fmt::Display for Variations {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut variations = self.variations.iter().peekable();
        let mut i = 1;
        while let Some(variation) = variations.next() {
            writeln!(f, "{}. {}", i, variation)?;
            i += 1;
        }
        Ok(())
    }
}
