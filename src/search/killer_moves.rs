use crate::PieceMove;

pub struct KillerMoves {
    moves: Vec<[Option<PieceMove>; 2]>, // Store 2 killer moves per ply
    max_ply: usize,
}

impl KillerMoves {
    pub fn new(max_ply: usize) -> Self {
        Self {
            moves: vec![[None, None]; max_ply],
            max_ply,
        }
    }

    pub fn add_killer(&mut self, mv: PieceMove, ply: usize) {
        if ply >= self.max_ply {
            return;
        }

        // Don't store captures as killer moves
        if mv.is_capture() {
            return;
        }

        // If this move is already a killer move at this ply, return
        if self.moves[ply][0].as_ref() == Some(&mv) || self.moves[ply][1].as_ref() == Some(&mv) {
            return;
        }

        // Shift existing killer move to second slot and store new killer move in first slot
        self.moves[ply][1] = self.moves[ply][0];
        self.moves[ply][0] = Some(mv);
    }

    pub fn get_killers(&self, ply: usize) -> [Option<PieceMove>; 2] {
        if ply >= self.max_ply {
            return [None, None];
        }
        self.moves[ply]
    }
}
