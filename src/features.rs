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
    pub evaluate_trapped_pieces: bool,
    pub evaluate_strategic_squares: bool,
    pub evaluate_piece_pressure: bool,
    pub evaluate_pawn_structure_quality: bool,
    pub evaluate_pawn_defense_quality: bool,
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
            evaluate_piece_coordination: false,
            evaluate_pawn_control: false,
            evaluate_piece_protection: false,
            evaluate_trapped_pieces: false,
            evaluate_strategic_squares: false,
            evaluate_piece_pressure: false,
            evaluate_pawn_structure_quality: false,
            evaluate_pawn_defense_quality: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvaluationWeights {
    pub material: i32,
    pub bishop_pair: i32,
    pub pawn_structure: i32,
    pub king_safety: i32,
    pub mobility: i32,
    pub piece_coordination: i32,
    pub pawn_control: i32,
    pub piece_protection: i32,
    pub trapped_pieces: i32,
    pub strategic_squares: i32,
    pub piece_pressure: i32,
    pub pawn_structure_quality: i32,
    pub pawn_defense_quality: i32,
}

impl Default for EvaluationWeights {
    fn default() -> Self {
        Self {
            material: 100,              // Base multiplier for material values
            bishop_pair: 50,            // Was hardcoded as 50
            pawn_structure: 100,        // Full weight for pawn structure
            king_safety: 100,           // Full weight for king safety
            mobility: 75,               // Slightly lower to not overshadow structure
            piece_coordination: 80,     // Important but not as much as material
            pawn_control: 70,           // Good bonus but shouldn't dominate
            piece_protection: 60,       // Moderate importance
            trapped_pieces: 90,         // Important but not as much as material
            strategic_squares: 85,      // Key for positional play
            piece_pressure: 65,         // Good bonus for long-term pressure
            pawn_structure_quality: 95, // Almost as important as basic structure
            pawn_defense_quality: 95,   // Almost as important as basic structure
        }
    }
}
