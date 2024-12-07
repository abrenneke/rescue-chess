#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Features {
    pub enable_transposition_table: bool,
    pub enable_lmr: bool,
    pub enable_window_search: bool,
    pub enable_killer_moves: bool,
    pub enable_null_move_pruning: bool,
    pub enable_history: bool,

    pub evaluate_bishop_pairs: bool,
    pub evaluate_pawn_structure: bool,
    pub evaluate_king_safety: bool,
    pub evaluate_mobility: bool,
    pub evaluate_piece_coordination: bool,
    pub evaluate_pawn_control: bool,
    pub evaluate_piece_protection: bool,
}

impl Default for Features {
    fn default() -> Self {
        Self {
            enable_transposition_table: true,
            enable_lmr: true,
            enable_window_search: true,
            enable_killer_moves: true,
            enable_null_move_pruning: true,
            enable_history: true,

            evaluate_bishop_pairs: true,
            evaluate_pawn_structure: true,
            evaluate_king_safety: true,
            evaluate_mobility: true, // slow, over doubles search time
            evaluate_piece_coordination: true,
            evaluate_pawn_control: true,
            evaluate_piece_protection: true,
        }
    }
}
