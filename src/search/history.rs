use crate::PieceMove;

#[derive(Debug, Clone)]
pub struct HistoryTable {
    // Track success of [piece_type][to_square] combinations
    pub success: [[i32; 64]; 6], // 6 piece types, 64 squares
    // Track how many times we tried each move
    pub tried: [[i32; 64]; 6],
}

impl HistoryTable {
    pub fn new() -> Self {
        Self {
            success: [[0; 64]; 6],
            tried: [[0; 64]; 6],
        }
    }

    pub fn update_history(&mut self, mv: &PieceMove, depth: u32, caused_cutoff: bool) {
        let piece_idx = mv.piece_type as usize;
        let square_idx = mv.to.0 as usize;

        self.tried[piece_idx][square_idx] += 1;
        if caused_cutoff {
            // Bonus based on depth - deeper cutoffs are more valuable
            self.success[piece_idx][square_idx] += depth as i32;
        }
    }

    pub fn get_history_score(&self, mv: &PieceMove) -> i32 {
        let piece_idx = mv.piece_type as usize;
        let square_idx = mv.to.0 as usize;

        let successes = self.success[piece_idx][square_idx];
        let attempts = self.tried[piece_idx][square_idx];

        if attempts == 0 {
            return 0;
        }

        (successes * 2000) / attempts
    }
}
