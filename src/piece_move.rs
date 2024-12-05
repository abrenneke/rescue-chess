use parser::RescueOrDrop;
use serde::{Deserialize, Serialize};

use crate::{Bitboard, Color, Piece, PieceType, Pos, Position};

mod parser;

pub trait CanMove {
    fn get_legal_moves(piece: &Piece, position: &Position) -> Bitboard;
}

/// A move that a chess piece can make.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct PieceMove {
    /// The type of piece that is moving
    pub piece_type: PieceType,

    /// The position the piece is moving from
    pub from: Pos,

    /// The position the piece is moving to. May be the same position if the pieces is only picking up another piece.
    pub to: Pos,

    /// Additional information about the move
    pub move_type: MoveType,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GameType {
    Classic,
    Rescue,
}

/// The type of move a piece can make. Non-normal moves can store additional information, such as captured piece.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum MoveType {
    /// A move that does not capture an enemy piece
    Normal {
        captured: Option<PieceType>,
        captured_pos: Option<Pos>,
        captured_holding: Option<PieceType>,

        rescued_pos: Option<Pos>,
        dropped_pos: Option<Pos>,

        promoted_to: Option<PieceType>,
        dropped_promoted_to: Option<PieceType>,
    },

    /// A move that castles the king
    /// The first Pos is the position of the king, the second is the position of the rook
    Castle { king: Pos, rook: Pos },
}

impl PieceMove {
    /// Inverts the move, so that it can be applied from the other player's perspective.
    pub fn invert(&mut self) {
        self.from = self.from.invert();
        self.to = self.to.invert();

        match &mut self.move_type {
            MoveType::Normal {
                captured_pos,
                rescued_pos,
                dropped_pos,
                captured: _,
                captured_holding: _,
                dropped_promoted_to: _,
                promoted_to: _,
            } => {
                *captured_pos = captured_pos.map(|pos| pos.invert());
                *rescued_pos = rescued_pos.map(|pos| pos.invert());
                *dropped_pos = dropped_pos.map(|pos| pos.invert());
            }
            MoveType::Castle { king, rook } => {
                *king = king.invert();
                *rook = rook.invert();
            }
        }
    }

    /// Returns a new move that is the inverse of this move, so that it can be applied from the other player's perspective.
    pub fn inverted(&self) -> PieceMove {
        let mut inverted = *self;
        inverted.invert();
        inverted
    }

    pub fn is_capture(&self) -> bool {
        match self.move_type {
            MoveType::Normal {
                captured: Some(_), ..
            } => true,
            _ => false,
        }
    }

    pub fn is_rescue_or_drop(&self) -> bool {
        match self.move_type {
            MoveType::Normal {
                rescued_pos: Some(_),
                ..
            } => true,
            MoveType::Normal {
                dropped_pos: Some(_),
                ..
            } => true,
            _ => false,
        }
    }

    pub fn is_rescue(&self) -> bool {
        match self.move_type {
            MoveType::Normal {
                rescued_pos: Some(_),
                ..
            } => true,
            _ => false,
        }
    }

    pub fn is_drop(&self) -> bool {
        match self.move_type {
            MoveType::Normal {
                dropped_pos: Some(_),
                ..
            } => true,
            _ => false,
        }
    }

    pub fn from_algebraic(
        position: &Position,
        notation: &str,
        game_type: GameType,
    ) -> Result<PieceMove, anyhow::Error> {
        let parsed = parser::PieceMoveParser::parse(notation)?;
        Self::from_algebraic_impl(position, parsed, game_type)
    }

    pub fn from_algebraic_inverted(
        position: &Position,
        notation_inverted: &str,
        game_type: GameType,
    ) -> Result<PieceMove, anyhow::Error> {
        let mut parsed = parser::PieceMoveParser::parse(notation_inverted)?;
        parsed.invert();
        Self::from_algebraic_impl(position, parsed, game_type)
    }

    pub fn from_uci(
        position: &Position,
        notation: &str,
        game_type: GameType,
    ) -> Result<PieceMove, anyhow::Error> {
        let parsed = parser::ParsedMove::from_uci(notation, position, false)?;
        Self::from_algebraic_impl(position, parsed, game_type)
    }

    pub fn from_uci_inverted(
        position: &Position,
        notation_inverted: &str,
        game_type: GameType,
    ) -> Result<PieceMove, anyhow::Error> {
        let parsed = parser::ParsedMove::from_uci(notation_inverted, position, true)?;
        Self::from_algebraic_impl(position, parsed, game_type)
    }

    fn from_algebraic_impl(
        position: &Position,
        parsed: parser::ParsedMove,
        game_type: GameType,
    ) -> Result<PieceMove, anyhow::Error> {
        // Get all legal moves for pieces of this type
        let legal_moves = position.get_all_legal_moves(game_type)?;

        let mut matching_moves: Vec<PieceMove> = legal_moves
            .into_iter()
            .filter(|mv| {
                // Helper function to check if move matches rescue/drop pattern
                let rescue_drop_matches = match (&parsed.rescue_drop, &mv.move_type) {
                    // No rescue/drop specified in notation
                    (
                        None,
                        MoveType::Normal {
                            dropped_pos: None,
                            rescued_pos: None,
                            ..
                        },
                    ) => true,
                    (None, MoveType::Castle { .. }) => true,

                    // Rescue specified in notation
                    (
                        Some(RescueOrDrop::Rescue),
                        MoveType::Normal {
                            rescued_pos: Some(pos),
                            ..
                        },
                    ) => {
                        // If position specified in notation, must match
                        match (parsed.rescue_drop_file, parsed.rescue_drop_rank) {
                            (Some(file), Some(rank)) => {
                                pos.get_col() == file && pos.get_row() == rank
                            }
                            (Some(file), None) => pos.get_col() == file,
                            (None, Some(rank)) => pos.get_row() == rank,
                            (None, None) => true,
                        }
                    }

                    // Drop specified in notation
                    (
                        Some(RescueOrDrop::Drop),
                        MoveType::Normal {
                            dropped_pos: Some(pos),
                            ..
                        },
                    ) => {
                        // If position specified in notation, must match
                        match (parsed.rescue_drop_file, parsed.rescue_drop_rank) {
                            (Some(file), Some(rank)) => {
                                pos.get_col() == file && pos.get_row() == rank
                            }
                            (Some(file), None) => pos.get_col() == file,
                            (None, Some(rank)) => pos.get_row() == rank,
                            (None, None) => true,
                        }
                    }

                    // Mismatch in rescue/drop
                    _ => false,
                };

                // Full move matching logic
                let res = mv.piece_type == parsed.piece_type &&  // Match piece type
                        mv.to == Pos::xy(parsed.to_file, parsed.to_rank) &&  // Match destination square
                        (!parsed.is_capture || mv.is_capture()) &&  // Match capture flag if specified
                        parsed.from_file.map_or(true, |file| mv.from.get_col() == file) &&  // Match source file if specified
                        parsed.from_rank.map_or(true, |rank| mv.from.get_row() == rank) &&  // Match source rank if specified
                        rescue_drop_matches; // Match rescue/drop pattern if specified

                res
            })
            .collect();

        // If no moves match, the notation is invalid
        if matching_moves.is_empty() {
            return Err(anyhow::anyhow!(
                "No piece can make this move: {}. Possible moves are: {}. Board state\n{}\n{}",
                parsed.to_algebraic(),
                position
                    .get_all_legal_moves(game_type)?
                    .iter()
                    .map(|mv| mv.to_string())
                    .collect::<Vec<String>>()
                    .join(", "),
                position.to_board_string_with_rank_file_holding(),
                position.to_fen()
            ));
        }

        // If exactly one move matches, that's our move
        if matching_moves.len() == 1 {
            return Ok(matching_moves.remove(0));
        }

        // If we have multiple matches but no disambiguation was provided,
        // this is ambiguous and invalid
        if parsed.from_file.is_none() && parsed.from_rank.is_none() {
            return Err(anyhow::anyhow!(
                "Ambiguous move - need disambiguation. Possible moves: {}",
                matching_moves
                    .into_iter()
                    .map(|mv| mv.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ));
        }

        // Return the first matching move (we've already filtered by disambiguation)
        Ok(matching_moves.remove(0))
    }

    pub fn to_uci(&self) -> String {
        match self.move_type {
            MoveType::Normal { promoted_to, .. } => {
                if let Some(promoted_to) = promoted_to {
                    format!(
                        "{}{}{}",
                        self.from.to_algebraic(),
                        self.to.to_algebraic(),
                        promoted_to.to_algebraic(Color::White),
                    )
                } else {
                    format!("{}{}", self.from.to_algebraic(), self.to.to_algebraic())
                }
            }
            MoveType::Castle { .. } => {
                format!("{}{}", self.from.to_algebraic(), self.to.to_algebraic())
            }
        }
    }
}

/// Displays the move in algebraic notation.
impl std::fmt::Display for PieceMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let MoveType::Castle { king: _, rook } = self.move_type {
            if rook.get_col() == 0 {
                write!(f, "O-O-O")?;
            } else {
                write!(f, "O-O")?;
            }

            return Ok(());
        }

        write!(
            f,
            "{}",
            match self.piece_type {
                PieceType::Pawn => "",
                PieceType::Knight => "N",
                PieceType::Bishop => "B",
                PieceType::Rook => "R",
                PieceType::Queen => "Q",
                PieceType::King => "K",
            }
        )?;

        if let MoveType::Normal {
            captured: Some(_), ..
        } = &self.move_type
        {
            write!(f, "x")?;
        }

        write!(f, "{}", self.to.to_algebraic())?;

        if let MoveType::Normal {
            promoted_to: Some(promoted_to),
            ..
        } = &self.move_type
        {
            write!(f, "={}", promoted_to.to_algebraic(Color::White))?;
        }

        if let MoveType::Normal {
            rescued_pos: Some(rescued_pos),
            ..
        } = &self.move_type
        {
            write!(f, "S{}", rescued_pos.to_algebraic())?;
        }

        if let MoveType::Normal {
            dropped_pos: Some(dropped_pos),
            dropped_promoted_to,
            ..
        } = &self.move_type
        {
            write!(f, "D{}", dropped_pos.to_algebraic())?;
            if let Some(dropped_promoted_to) = dropped_promoted_to {
                write!(f, "{}", dropped_promoted_to.to_algebraic(Color::White))?;
            }
        }

        Ok(())
    }
}

impl std::fmt::Debug for PieceMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pawn_move() {
        let position =
            Position::parse_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                .unwrap();
        let mv = PieceMove::from_algebraic(&position, "e4", GameType::Rescue).unwrap();
        assert_eq!(mv.piece_type, PieceType::Pawn);
        assert_eq!(mv.from, Pos::from_algebraic("e2").unwrap());
        assert_eq!(mv.to, Pos::from_algebraic("e4").unwrap());
        assert!(matches!(mv.move_type, MoveType::Normal { .. }));
    }

    #[test]
    fn test_knight_move() {
        let position =
            Position::parse_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                .unwrap();
        let mv = PieceMove::from_algebraic(&position, "Nf3", GameType::Rescue).unwrap();
        assert_eq!(mv.piece_type, PieceType::Knight);
        assert_eq!(mv.from, Pos::from_algebraic("g1").unwrap());
        assert_eq!(mv.to, Pos::from_algebraic("f3").unwrap());
        assert!(matches!(mv.move_type, MoveType::Normal { .. }));
    }

    #[test]
    fn test_pawn_capture() {
        let position = Position::parse_from_fen(
            "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1",
        )
        .unwrap();

        let mv = PieceMove::from_algebraic(&position, "exd5", GameType::Rescue).unwrap();
        assert_eq!(mv.piece_type, PieceType::Pawn);
        assert_eq!(mv.from, Pos::from_algebraic("e4").unwrap());
        assert_eq!(mv.to, Pos::from_algebraic("d5").unwrap());
        assert!(matches!(
            mv.move_type,
            MoveType::Normal {
                captured: Some(PieceType::Pawn),
                captured_holding: None,
                ..
            }
        ));
    }

    #[test]
    fn test_disambiguation_file() {
        let position = Position::parse_from_fen(
            "r1bqkbnr/ppp1pppp/2n5/3p4/3P4/2N5/PPP2PPP/R1BQKBNR w KQkq - 0 1",
        )
        .unwrap();

        let mv = PieceMove::from_algebraic(&position, "Nce2", GameType::Rescue).unwrap();
        assert_eq!(mv.piece_type, PieceType::Knight);
        assert_eq!(mv.from, Pos::from_algebraic("c3").unwrap());
        assert_eq!(mv.to, Pos::from_algebraic("e2").unwrap());
    }

    #[test]
    fn test_disambiguation_rank() {
        let position = Position::parse_from_fen(
            "r1bqkbnr/ppp1pppp/2n5/3p4/3P4/2N5/PPP2PPP/R1BQKBNR w KQkq - 0 1",
        )
        .unwrap();

        let mv = PieceMove::from_algebraic(&position, "N3e2", GameType::Rescue).unwrap();
        assert_eq!(mv.piece_type, PieceType::Knight);
        assert_eq!(mv.from, Pos::from_algebraic("c3").unwrap());
        assert_eq!(mv.to, Pos::from_algebraic("e2").unwrap());
    }

    #[test]
    fn test_with_check_symbol() {
        let position = Position::parse_from_fen(
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1",
        )
        .unwrap();
        let mv = PieceMove::from_algebraic(&position, "Qh5+", GameType::Rescue).unwrap();
        assert_eq!(mv.piece_type, PieceType::Queen);
        assert_eq!(mv.to, Pos::from_algebraic("h5").unwrap());
        assert!(matches!(mv.move_type, MoveType::Normal { .. }));
    }

    // #[test]
    // fn test_en_passant() {
    //     let position =
    //         Position::parse_from_fen("rnbqkbnr/ppp2ppp/8/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq e6 0 1")
    //             .unwrap();
    //     let mv = PieceMove::from_algebraic(&position, "dxe6").unwrap();
    //     assert_eq!(mv.piece_type, PieceType::Pawn);
    //     assert_eq!(mv.from, Pos::from_algebraic("d5").unwrap());
    //     assert_eq!(mv.to, Pos::from_algebraic("e6").unwrap());
    //     assert!(matches!(mv.move_type, MoveType::EnPassant(_)));
    // }

    #[test]
    fn test_invalid_move() {
        let position =
            Position::parse_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                .unwrap();
        assert!(PieceMove::from_algebraic(&position, "e5", GameType::Rescue).is_err());
        // Invalid move, pawn can't move two squares
    }

    #[test]
    fn test_basic_pawn_moves() {
        // Starting position
        let position =
            Position::parse_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                .unwrap();

        // Test e4
        let mv = PieceMove::from_algebraic(&position, "e4", GameType::Rescue).unwrap();
        assert_eq!(mv.piece_type, PieceType::Pawn);
        assert_eq!(mv.from, Pos::from_algebraic("e2").unwrap());
        assert_eq!(mv.to, Pos::from_algebraic("e4").unwrap());
        assert!(matches!(mv.move_type, MoveType::Normal { .. }));

        // Test d3
        let mv = PieceMove::from_algebraic(&position, "d3", GameType::Rescue).unwrap();
        assert_eq!(mv.piece_type, PieceType::Pawn);
        assert_eq!(mv.from, Pos::from_algebraic("d2").unwrap());
        assert_eq!(mv.to, Pos::from_algebraic("d3").unwrap());
        assert!(matches!(mv.move_type, MoveType::Normal { .. }));
    }

    #[test]
    fn test_pawn_captures() {
        // Position with potential pawn captures
        let position = Position::parse_from_fen(
            "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1",
        )
        .unwrap();

        // Test exd5
        let mv = PieceMove::from_algebraic(&position, "exd5", GameType::Rescue).unwrap();
        assert_eq!(mv.piece_type, PieceType::Pawn);
        assert_eq!(mv.from, Pos::from_algebraic("e4").unwrap());
        assert_eq!(mv.to, Pos::from_algebraic("d5").unwrap());
        assert!(mv.is_capture());
    }

    #[test]
    fn test_knight_move_2() {
        // Starting position
        let position =
            Position::parse_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                .unwrap();

        // Test Nf3
        let mv = PieceMove::from_algebraic(&position, "Nf3", GameType::Rescue).unwrap();
        assert_eq!(mv.piece_type, PieceType::Knight);
        assert_eq!(mv.from, Pos::from_algebraic("g1").unwrap());
        assert_eq!(mv.to, Pos::from_algebraic("f3").unwrap());
        assert!(matches!(mv.move_type, MoveType::Normal { .. }));
    }

    #[test]
    fn test_knight_captures() {
        // Position with potential piece capture
        let position = Position::parse_from_fen(
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 1",
        )
        .unwrap();

        // Test Nxe5
        let mv = PieceMove::from_algebraic(&position, "Nxe5", GameType::Rescue).unwrap();
        assert_eq!(mv.piece_type, PieceType::Knight);
        assert_eq!(mv.from, Pos::from_algebraic("f3").unwrap());
        assert_eq!(mv.to, Pos::from_algebraic("e5").unwrap());
        assert!(mv.is_capture());
    }

    #[test]
    fn test_error_cases() {
        let position =
            Position::parse_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                .unwrap();

        // Test invalid pawn move (too far)
        assert!(PieceMove::from_algebraic(&position, "e5", GameType::Rescue).is_err());

        // Test invalid capture (no piece to capture)
        assert!(PieceMove::from_algebraic(&position, "Nxe4", GameType::Rescue).is_err());

        // Test invalid piece move (blocked)
        assert!(PieceMove::from_algebraic(&position, "Nc4", GameType::Rescue).is_err());
    }

    #[test]
    fn test_check_and_mate_notation() {
        // Position with check possibility
        let position = Position::parse_from_fen(
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1",
        )
        .unwrap();

        // Test move with check symbol
        let mv = PieceMove::from_algebraic(&position, "Qh5+", GameType::Rescue).unwrap();
        assert_eq!(mv.piece_type, PieceType::Queen);
        assert_eq!(mv.from, Pos::from_algebraic("d1").unwrap());
        assert_eq!(mv.to, Pos::from_algebraic("h5").unwrap());

        // Test move with checkmate symbol (same move, different notation)
        let mv2 = PieceMove::from_algebraic(&position, "Qh5#", GameType::Rescue).unwrap();
        assert_eq!(mv, mv2);
    }

    #[test]
    fn test_ambiguous_moves() {
        // Position with two knights that could move to the same square
        let position = Position::parse_from_fen(
            "r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/2N5/PPPP1PPP/R1BQKBNR w KQkq - 0 1",
        )
        .unwrap();

        // Test ambiguous move (should fail)
        assert!(PieceMove::from_algebraic(&position, "Ne2", GameType::Rescue).is_err());

        // Test with proper disambiguation
        let mv = PieceMove::from_algebraic(&position, "Nce2", GameType::Rescue).unwrap();
        assert_eq!(mv.from, Pos::from_algebraic("c3").unwrap());
        assert_eq!(mv.to, Pos::from_algebraic("e2").unwrap());
    }

    #[test]
    fn test_queen_moves() {
        // Position with queen moves
        let position = Position::parse_from_fen(
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1",
        )
        .unwrap();

        // Test diagonal queen move
        let mv = PieceMove::from_algebraic(&position, "Qh5", GameType::Rescue).unwrap();
        assert_eq!(mv.piece_type, PieceType::Queen);
        assert_eq!(mv.from, Pos::from_algebraic("d1").unwrap());
        assert_eq!(mv.to, Pos::from_algebraic("h5").unwrap());
    }

    #[test]
    fn test_bishop_moves() {
        // Position with bishop moves
        let position = Position::parse_from_fen(
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1",
        )
        .unwrap();

        // Test bishop move
        let mv = PieceMove::from_algebraic(&position, "Bc4", GameType::Rescue).unwrap();
        assert_eq!(mv.piece_type, PieceType::Bishop);
        assert_eq!(mv.from, Pos::from_algebraic("f1").unwrap());
        assert_eq!(mv.to, Pos::from_algebraic("c4").unwrap());
    }

    #[test]
    fn test_king_moves() {
        // Position with king move
        let position = Position::parse_from_fen(
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1",
        )
        .unwrap();

        // Test king move
        let mv = PieceMove::from_algebraic(&position, "Ke2", GameType::Rescue).unwrap();
        assert_eq!(mv.piece_type, PieceType::King);
        assert_eq!(mv.from, Pos::from_algebraic("e1").unwrap());
        assert_eq!(mv.to, Pos::from_algebraic("e2").unwrap());
    }

    #[test]
    fn test_rescue_moves() {
        let position = Position::start_position();

        let mv = PieceMove::from_algebraic(&position, "e2Sf2", GameType::Rescue).unwrap();
        assert_eq!(mv.piece_type, PieceType::Pawn);
        assert_eq!(mv.from, Pos::from_algebraic("e2").unwrap());
        assert_eq!(mv.to, Pos::from_algebraic("e2").unwrap()); // Piece stays in place for rescue
        assert!(
            matches!(mv.move_type, MoveType::Normal{ rescued_pos: Some(pos), .. } if pos == Pos::from_algebraic("f2").unwrap())
        );

        // Test partial specifications
        let mv = PieceMove::from_algebraic(&position, "e2Sf", GameType::Rescue).unwrap();
        assert!(
            matches!(mv.move_type, MoveType::Normal{ rescued_pos: Some(pos), .. } if pos == Pos::from_algebraic("f2").unwrap())
        );

        let mv = PieceMove::from_algebraic(&position, "e2S2", GameType::Rescue);
        assert!(mv.is_err()); // Ambiguous move, could rescue d2 or f2

        // Test error case - trying to rescue a piece that's not adjacent
        assert!(PieceMove::from_algebraic(&position, "e4Sa4", GameType::Rescue).is_err());
    }

    #[test]
    fn test_move_and_rescue_moves() {
        let position = Position::start_position();

        let mv = PieceMove::from_algebraic(&position, "Nf3Sf2", GameType::Rescue).unwrap();
        assert_eq!(mv.piece_type, PieceType::Knight);
        assert_eq!(mv.from, Pos::from_algebraic("g1").unwrap());
        assert_eq!(mv.to, Pos::from_algebraic("f3").unwrap());
        assert!(
            matches!(mv.move_type, MoveType::Normal{ rescued_pos: Some(pos), .. } if pos == Pos::from_algebraic("f2").unwrap())
        );
    }
}
