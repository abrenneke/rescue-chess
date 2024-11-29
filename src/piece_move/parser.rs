use crate::{PieceType, Pos, Position};

#[derive(Debug)]
enum ParserState {
    Start,
    AfterPiece,
    AfterPosition,
    AfterCapture,
    AfterRescueOrDrop,
    Done,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RescueOrDrop {
    Rescue,
    Drop,
}

#[derive(Debug)]
pub struct ParsedMove {
    pub piece_type: PieceType,
    pub from_file: Option<u8>,
    pub from_rank: Option<u8>,
    pub to_file: u8,
    pub to_rank: u8,
    pub is_capture: bool,
    pub rescue_drop: Option<RescueOrDrop>,
    pub rescue_drop_file: Option<u8>,
    pub rescue_drop_rank: Option<u8>,
    pub promotion_to: Option<PieceType>,
}

impl std::fmt::Display for ParsedMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_algebraic())
    }
}

impl ParsedMove {
    pub fn from_uci(uci: &str, position: &Position, inverted: bool) -> Result<Self, anyhow::Error> {
        let from_file = uci
            .chars()
            .nth(0)
            .ok_or_else(|| anyhow::anyhow!("Invalid UCI"))?;
        let from_rank = uci
            .chars()
            .nth(1)
            .ok_or_else(|| anyhow::anyhow!("Invalid UCI"))?;
        let to_file = uci
            .chars()
            .nth(2)
            .ok_or_else(|| anyhow::anyhow!("Invalid UCI"))?;
        let to_rank = uci
            .chars()
            .nth(3)
            .ok_or_else(|| anyhow::anyhow!("Invalid UCI"))?;
        let promotion = uci.chars().nth(4).map(|c| match c {
            'q' => PieceType::Queen,
            'r' => PieceType::Rook,
            'b' => PieceType::Bishop,
            'n' => PieceType::Knight,
            _ => PieceType::Queen,
        });

        let from_pos = if inverted {
            Pos::from_algebraic(&format!("{}{}", from_file, from_rank))
                .unwrap()
                .invert()
        } else {
            Pos::from_algebraic(&format!("{}{}", from_file, from_rank)).unwrap()
        };

        let to_pos = if inverted {
            Pos::from_algebraic(&format!("{}{}", to_file, to_rank))
                .unwrap()
                .invert()
        } else {
            Pos::from_algebraic(&format!("{}{}", to_file, to_rank)).unwrap()
        };

        let piece_type = position
            .get_piece_at(from_pos)
            .ok_or_else(|| anyhow::anyhow!("No piece at source square"))?
            .piece_type;

        Ok(Self {
            piece_type,
            from_file: Some(from_pos.get_col()),
            from_rank: Some(from_pos.get_row()),
            to_file: to_pos.get_col(),
            to_rank: to_pos.get_row(),
            is_capture: false,
            rescue_drop: None,
            rescue_drop_file: None,
            rescue_drop_rank: None,
            promotion_to: promotion,
        })
    }

    pub fn invert(&mut self) {
        if let Some(file) = self.from_file {
            self.from_file = Some(7 - file);
        }
        if let Some(rank) = self.from_rank {
            self.from_rank = Some(7 - rank);
        }
        self.to_file = 7 - self.to_file;
        self.to_rank = 7 - self.to_rank;
        if let Some(file) = self.rescue_drop_file {
            self.rescue_drop_file = Some(7 - file);
        }
        if let Some(rank) = self.rescue_drop_rank {
            self.rescue_drop_rank = Some(7 - rank);
        }
    }

    pub fn to_algebraic(&self) -> String {
        let mut result = String::new();

        match self.piece_type {
            PieceType::Pawn => {}
            PieceType::Knight => result.push('N'),
            PieceType::Bishop => result.push('B'),
            PieceType::Rook => result.push('R'),
            PieceType::Queen => result.push('Q'),
            PieceType::King => result.push('K'),
        }

        if let Some(file) = self.from_file {
            result.push((b'a' + file) as char);
        }
        if let Some(rank) = self.from_rank {
            result.push((b'1' + rank) as char);
        }

        if self.is_capture {
            result.push('x');
        }

        result.push((b'a' + self.to_file) as char);
        result.push((b'1' + self.to_rank) as char);

        if let Some(rescue_drop) = &self.rescue_drop {
            match rescue_drop {
                RescueOrDrop::Rescue => result.push('S'),
                RescueOrDrop::Drop => result.push('D'),
            }
            if let Some(file) = self.rescue_drop_file {
                result.push((b'a' + file) as char);
            }
            if let Some(rank) = self.rescue_drop_rank {
                result.push((b'1' + rank) as char);
            }
        }

        result
    }
}

pub struct PieceMoveParser {
    state: ParserState,
    result: ParsedMove,
    // Track the last position we saw - we don't know if it's source or dest yet
    last_file: Option<u8>,
    last_rank: Option<u8>,
}

impl PieceMoveParser {
    pub fn new() -> Self {
        Self {
            state: ParserState::Start,
            result: ParsedMove {
                piece_type: PieceType::Pawn,
                from_file: None,
                from_rank: None,
                to_file: 0,
                to_rank: 0,
                is_capture: false,
                rescue_drop: None,
                rescue_drop_file: None,
                rescue_drop_rank: None,
                promotion_to: None,
            },
            last_file: None,
            last_rank: None,
        }
    }

    fn file_to_index(file: char) -> Result<u8, anyhow::Error> {
        if !('a'..='h').contains(&file) {
            return Err(anyhow::anyhow!("Invalid file: {}", file));
        }
        Ok(file as u8 - b'a')
    }

    fn rank_to_index(rank: char) -> Result<u8, anyhow::Error> {
        if !('1'..='8').contains(&rank) {
            return Err(anyhow::anyhow!("Invalid rank: {}", rank));
        }
        // Convert to internal rank (0-7, from top to bottom)
        Ok(7 - (rank as u8 - b'1'))
    }

    pub fn feed_char(&mut self, c: char) -> Result<(), anyhow::Error> {
        match self.state {
            ParserState::Start => match c {
                'N' => {
                    self.result.piece_type = PieceType::Knight;
                    self.state = ParserState::AfterPiece;
                }
                'B' => {
                    self.result.piece_type = PieceType::Bishop;
                    self.state = ParserState::AfterPiece;
                }
                'R' => {
                    self.result.piece_type = PieceType::Rook;
                    self.state = ParserState::AfterPiece;
                }
                'Q' => {
                    self.result.piece_type = PieceType::Queen;
                    self.state = ParserState::AfterPiece;
                }
                'K' => {
                    self.result.piece_type = PieceType::King;
                    self.state = ParserState::AfterPiece;
                }
                'a'..='h' => {
                    self.last_file = Some(Self::file_to_index(c)?);
                    self.state = ParserState::AfterPosition;
                }
                _ => return Err(anyhow::anyhow!("Unexpected character: {}", c)),
            },

            ParserState::AfterPiece => match c {
                'a'..='h' => {
                    self.last_file = Some(Self::file_to_index(c)?);
                    self.state = ParserState::AfterPosition;
                }
                '1'..='8' => {
                    self.last_rank = Some(Self::rank_to_index(c)?);
                    self.state = ParserState::AfterPosition;
                }
                'x' => {
                    if self.result.is_capture {
                        return Err(anyhow::anyhow!("Unexpected second capture marker"));
                    }
                    self.result.is_capture = true;
                    self.state = ParserState::AfterCapture;
                }
                _ => return Err(anyhow::anyhow!("Unexpected character after piece: {}", c)),
            },

            ParserState::AfterPosition => match c {
                'a'..='h' => {
                    self.result.from_file = self.last_file;
                    self.result.from_rank = self.last_rank;
                    self.last_file = Some(Self::file_to_index(c)?);
                    self.last_rank = None;
                    self.state = ParserState::AfterPosition;
                }
                '1'..='8' => {
                    let rank = Self::rank_to_index(c)?;
                    if self.last_rank.is_some() {
                        return Err(anyhow::anyhow!("Unexpected second rank"));
                    }
                    self.last_rank = Some(rank);
                }
                'x' => {
                    if self.result.is_capture {
                        return Err(anyhow::anyhow!("Unexpected second capture marker"));
                    }

                    self.result.from_file = self.last_file;
                    self.result.from_rank = self.last_rank;
                    self.result.is_capture = true;
                    self.last_file = None;
                    self.last_rank = None;
                    self.state = ParserState::AfterCapture;
                }
                'S' | 'D' => {
                    // Handle rescue or drop operation
                    if self.last_file.is_none() || self.last_rank.is_none() {
                        return Err(anyhow::anyhow!("Incomplete position before rescue/drop"));
                    }
                    self.result.to_file = self.last_file.unwrap();
                    self.result.to_rank = self.last_rank.unwrap();
                    self.last_file = None;
                    self.last_rank = None;
                    self.state = ParserState::AfterRescueOrDrop;

                    match c {
                        'S' => {
                            self.result.rescue_drop = Some(RescueOrDrop::Rescue);
                        }
                        'D' => {
                            self.result.rescue_drop = Some(RescueOrDrop::Drop);
                        }
                        _ => unreachable!(),
                    }
                }
                '+' | '#' | '!' | '?' => {
                    if self.last_file.is_none() || self.last_rank.is_none() {
                        return Err(anyhow::anyhow!("Incomplete position before annotation"));
                    }
                    self.result.to_file = self.last_file.unwrap();
                    self.result.to_rank = self.last_rank.unwrap();
                    self.state = ParserState::Done;
                }
                _ => return Err(anyhow::anyhow!("Unexpected character: {}", c)),
            },

            ParserState::AfterCapture => match c {
                'a'..='h' => {
                    self.last_file = Some(Self::file_to_index(c)?);
                    self.state = ParserState::AfterPosition;
                }
                _ => return Err(anyhow::anyhow!("Expected file after capture")),
            },

            ParserState::AfterRescueOrDrop => match c {
                'a'..='h' => {
                    self.result.rescue_drop_file = Some(Self::file_to_index(c)?);
                }
                '1'..='8' => {
                    self.result.rescue_drop_rank = Some(Self::rank_to_index(c)?);
                }
                '+' | '#' | '!' | '?' => {
                    self.state = ParserState::Done;
                }
                _ => return Err(anyhow::anyhow!("Unexpected character after rescue/drop")),
            },

            ParserState::Done => match c {
                '+' | '#' | '!' | '?' => {} // Ignore annotation symbols
                _ => return Err(anyhow::anyhow!("Unexpected character after move: {}", c)),
            },
        }
        Ok(())
    }

    fn finalize(mut self) -> Result<ParsedMove, anyhow::Error> {
        // If we ended in AfterPosition, need to commit that position as destination
        if matches!(self.state, ParserState::AfterPosition) {
            if self.last_file.is_none() || self.last_rank.is_none() {
                return Err(anyhow::anyhow!("Incomplete position at end of input"));
            }
            self.result.to_file = self.last_file.unwrap();
            self.result.to_rank = self.last_rank.unwrap();
            self.state = ParserState::Done;
        }

        if !matches!(
            self.state,
            ParserState::Done | ParserState::AfterRescueOrDrop
        ) {
            return Err(anyhow::anyhow!("Incomplete move notation"));
        }

        Ok(self.result)
    }

    pub fn parse(notation: &str) -> Result<ParsedMove, anyhow::Error> {
        let mut parser = Self::new();

        // Strip any whitespace
        let clean_notation = notation.trim();

        for c in clean_notation.chars() {
            parser.feed_char(c)?;
        }

        parser.finalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pawn_moves() {
        // Simple pawn moves
        let parsed = PieceMoveParser::parse("e4").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Pawn);
        assert_eq!(parsed.from_file, None);
        assert_eq!(parsed.to_file, 4);
        assert_eq!(parsed.to_rank, 4);
        assert!(!parsed.is_capture);

        let parsed = PieceMoveParser::parse("d3").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Pawn);
        assert_eq!(parsed.from_file, None);
        assert_eq!(parsed.to_file, 3);
        assert_eq!(parsed.to_rank, 5);
        assert!(!parsed.is_capture);
    }

    #[test]
    fn test_pawn_captures() {
        // Standard pawn capture
        let parsed = PieceMoveParser::parse("exd5").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Pawn);
        assert_eq!(parsed.from_file, Some(4));
        assert_eq!(parsed.to_file, 3);
        assert_eq!(parsed.to_rank, 3);
        assert!(parsed.is_capture);

        // Another pawn capture
        let parsed = PieceMoveParser::parse("fxe4").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Pawn);
        assert_eq!(parsed.from_file, Some(5));
        assert_eq!(parsed.to_file, 4);
        assert_eq!(parsed.to_rank, 4);
        assert!(parsed.is_capture);
    }

    #[test]
    fn test_piece_moves() {
        // Simple knight move
        let parsed = PieceMoveParser::parse("Nf3").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Knight);
        assert_eq!(parsed.from_file, None);
        assert_eq!(parsed.to_file, 5);
        assert_eq!(parsed.to_rank, 5);
        assert!(!parsed.is_capture);

        // Simple bishop move
        let parsed = PieceMoveParser::parse("Be4").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Bishop);
        assert_eq!(parsed.from_file, None);
        assert_eq!(parsed.to_file, 4);
        assert_eq!(parsed.to_rank, 4);
        assert!(!parsed.is_capture);

        // Simple rook move
        let parsed = PieceMoveParser::parse("Ra3").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Rook);
        assert_eq!(parsed.from_file, None);
        assert_eq!(parsed.to_file, 0);
        assert_eq!(parsed.to_rank, 5);
        assert!(!parsed.is_capture);
    }

    #[test]
    fn test_piece_captures() {
        // Knight capture
        let parsed = PieceMoveParser::parse("Nxe5").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Knight);
        assert_eq!(parsed.from_file, None);
        assert_eq!(parsed.to_file, 4);
        assert_eq!(parsed.to_rank, 3);
        assert!(parsed.is_capture);

        // Queen capture
        let parsed = PieceMoveParser::parse("Qxf7").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Queen);
        assert_eq!(parsed.from_file, None);
        assert_eq!(parsed.to_file, 5);
        assert_eq!(parsed.to_rank, 1);
        assert!(parsed.is_capture);
    }

    #[test]
    fn test_disambiguation() {
        // File disambiguation
        let parsed = PieceMoveParser::parse("Nbd7").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Knight);
        assert_eq!(parsed.from_file, Some(1));
        assert_eq!(parsed.to_file, 3);
        assert_eq!(parsed.to_rank, 1);
        assert!(!parsed.is_capture);

        // Rank disambiguation
        let parsed = PieceMoveParser::parse("R1e4").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Rook);
        assert_eq!(parsed.from_file, None);
        assert_eq!(parsed.from_rank, Some(7));
        assert_eq!(parsed.to_file, 4);
        assert_eq!(parsed.to_rank, 4);
        assert!(!parsed.is_capture);

        // File disambiguation with capture
        let parsed = PieceMoveParser::parse("Nbxd5").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Knight);
        assert_eq!(parsed.from_file, Some(1));
        assert_eq!(parsed.to_file, 3);
        assert_eq!(parsed.to_rank, 3);
        assert!(parsed.is_capture);
    }

    #[test]
    fn test_with_check_symbols() {
        // Move with check
        let parsed = PieceMoveParser::parse("Nf3+").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Knight);
        assert_eq!(parsed.to_file, 5);
        assert_eq!(parsed.to_rank, 5);
        assert!(!parsed.is_capture);

        // Move with checkmate
        let parsed = PieceMoveParser::parse("Qxf7#").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Queen);
        assert_eq!(parsed.to_file, 5);
        assert_eq!(parsed.to_rank, 1);
        assert!(parsed.is_capture);

        // Move with annotation
        let parsed = PieceMoveParser::parse("e4!").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Pawn);
        assert_eq!(parsed.to_file, 4);
        assert_eq!(parsed.to_rank, 4);
        assert!(!parsed.is_capture);

        // Move with multiple annotations
        let parsed = PieceMoveParser::parse("Nf3+!?").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Knight);
        assert_eq!(parsed.to_file, 5);
        assert_eq!(parsed.to_rank, 5);
        assert!(!parsed.is_capture);
    }

    #[test]
    fn test_error_cases() {
        // Invalid file
        assert!(PieceMoveParser::parse("i4").is_err());

        // Invalid rank
        assert!(PieceMoveParser::parse("e9").is_err());

        // Invalid piece
        assert!(PieceMoveParser::parse("Xe4").is_err());

        // Incomplete move
        assert!(PieceMoveParser::parse("e").is_err());
        assert!(PieceMoveParser::parse("N").is_err());

        // Invalid capture notation
        assert!(PieceMoveParser::parse("exx4").is_err());
        assert!(PieceMoveParser::parse("Nxxe4").is_err());

        // Invalid characters
        assert!(PieceMoveParser::parse("e4$").is_err());
        assert!(PieceMoveParser::parse("N@f3").is_err());
    }

    #[test]
    fn nb1e3() {
        let parsed = PieceMoveParser::parse("Nb1e3").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Knight);
        assert_eq!(parsed.from_file, Some(1));
        assert_eq!(parsed.from_rank, Some(7));
        assert_eq!(parsed.to_file, 4);
        assert_eq!(parsed.to_rank, 5);
        assert!(!parsed.is_capture);
    }

    #[test]
    fn test_random_complex_moves() {
        // Standard piece moves with various disambiguation and capture patterns
        let testcases = [
            "Nbd7",   // Knight from b-file to d7
            "N1e3",   // Knight from rank 1 to e3
            "Nb1e3",  // Knight from b1 to e3 (full disambiguation)
            "Nf3e5",  // Knight from f3 to e5 (full source square)
            "Nb1xc3", // Knight from b1 captures on c3
            "Nf3xd4", // Knight from f3 captures on d4
            "R1a3",   // Rook from rank 1 to a3
            "Raxc1",  // Rook from a-file captures on c1
            "Ra1xh1", // Rook from a1 captures on h1
            "Qd1e2",  // Queen from d1 to e2
            "Qd8xd1", // Queen from d8 captures on d1
            "Bb5xc6", // Bishop from b5 captures on c6
            "Ba6b5",  // Bishop from a6 to b5
            "Kf1e1",  // King from f1 to e1
            "Ke8xf8", // King from e8 captures on f8
            // Pawn moves with various patterns
            "e4",   // Simple pawn push
            "d5",   // Simple pawn push
            "exd5", // Pawn capture
            "fxe4", // Pawn capture
            "axb8", // Pawn capture on back rank
            "hxg1", // Pawn capture on back rank
            // Moves with check/mate/annotation symbols
            "Nf3+",    // Check
            "Qxf7#",   // Checkmate
            "e4!",     // Good move
            "Nf3+!",   // Check with annotation
            "Qxe4!!",  // Brilliant move
            "Rxf8+?!", // Check with dubious annotation
        ];

        for notation in testcases {
            let result = PieceMoveParser::parse(notation);
            assert!(
                result.is_ok(),
                "Failed to parse '{}': {:?}",
                notation,
                result.err()
            );
        }
    }

    #[test]
    fn test_basic_rescue_moves() {
        // Test simple pawn rescue
        let parsed = PieceMoveParser::parse("e4Sa4").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Pawn);
        assert_eq!(parsed.to_file, 4);
        assert_eq!(parsed.to_rank, 4);
        assert_eq!(parsed.rescue_drop, Some(RescueOrDrop::Rescue));
        assert_eq!(parsed.rescue_drop_file, Some(0));
        assert_eq!(parsed.rescue_drop_rank, Some(4));
        assert!(!parsed.is_capture);

        // Test knight rescue
        let parsed = PieceMoveParser::parse("Nf3Sd2").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Knight);
        assert_eq!(parsed.to_file, 5);
        assert_eq!(parsed.to_rank, 5);
        assert_eq!(parsed.rescue_drop, Some(RescueOrDrop::Rescue));
        assert_eq!(parsed.rescue_drop_file, Some(3));
        assert_eq!(parsed.rescue_drop_rank, Some(6));
        assert!(!parsed.is_capture);
    }

    #[test]
    fn test_basic_drop_moves() {
        // Test simple pawn drop
        let parsed = PieceMoveParser::parse("e4Dd5").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Pawn);
        assert_eq!(parsed.to_file, 4);
        assert_eq!(parsed.to_rank, 4);
        assert_eq!(parsed.rescue_drop, Some(RescueOrDrop::Drop));
        assert_eq!(parsed.rescue_drop_file, Some(3));
        assert_eq!(parsed.rescue_drop_rank, Some(3));
        assert!(!parsed.is_capture);

        // Test queen drop
        let parsed = PieceMoveParser::parse("Qe2Df4").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Queen);
        assert_eq!(parsed.to_file, 4);
        assert_eq!(parsed.to_rank, 6);
        assert_eq!(parsed.rescue_drop, Some(RescueOrDrop::Drop));
        assert_eq!(parsed.rescue_drop_file, Some(5));
        assert_eq!(parsed.rescue_drop_rank, Some(4));
        assert!(!parsed.is_capture);
    }

    #[test]
    fn test_rescue_with_captures() {
        // Test rescue after capture
        let parsed = PieceMoveParser::parse("exd5Se6").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Pawn);
        assert_eq!(parsed.from_file, Some(4));
        assert_eq!(parsed.to_file, 3);
        assert_eq!(parsed.to_rank, 3);
        assert!(parsed.is_capture);
        assert_eq!(parsed.rescue_drop, Some(RescueOrDrop::Rescue));
        assert_eq!(parsed.rescue_drop_file, Some(4));
        assert_eq!(parsed.rescue_drop_rank, Some(2));

        // Test rescue with piece capture
        let parsed = PieceMoveParser::parse("Nxe4Sf2").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Knight);
        assert_eq!(parsed.to_file, 4);
        assert_eq!(parsed.to_rank, 4);
        assert!(parsed.is_capture);
        assert_eq!(parsed.rescue_drop, Some(RescueOrDrop::Rescue));
        assert_eq!(parsed.rescue_drop_file, Some(5));
        assert_eq!(parsed.rescue_drop_rank, Some(6));
    }

    #[test]
    fn test_drop_with_captures() {
        // Test drop after capture
        let parsed = PieceMoveParser::parse("exd5Df3").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Pawn);
        assert_eq!(parsed.from_file, Some(4));
        assert_eq!(parsed.to_file, 3);
        assert_eq!(parsed.to_rank, 3);
        assert!(parsed.is_capture);
        assert_eq!(parsed.rescue_drop, Some(RescueOrDrop::Drop));
        assert_eq!(parsed.rescue_drop_file, Some(5));
        assert_eq!(parsed.rescue_drop_rank, Some(5));

        // Test drop with piece capture
        let parsed = PieceMoveParser::parse("Bxe4Dc6").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Bishop);
        assert_eq!(parsed.to_file, 4);
        assert_eq!(parsed.to_rank, 4);
        assert!(parsed.is_capture);
        assert_eq!(parsed.rescue_drop, Some(RescueOrDrop::Drop));
        assert_eq!(parsed.rescue_drop_file, Some(2));
        assert_eq!(parsed.rescue_drop_rank, Some(2));
    }

    #[test]
    fn test_rescue_drop_with_check() {
        // Test rescue with check
        let parsed = PieceMoveParser::parse("Nf3Sd2+").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Knight);
        assert_eq!(parsed.to_file, 5);
        assert_eq!(parsed.to_rank, 5);
        assert_eq!(parsed.rescue_drop, Some(RescueOrDrop::Rescue));
        assert_eq!(parsed.rescue_drop_file, Some(3));
        assert_eq!(parsed.rescue_drop_rank, Some(6));
        assert!(!parsed.is_capture);

        // Test drop with checkmate
        let parsed = PieceMoveParser::parse("Qe7Df6#").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Queen);
        assert_eq!(parsed.to_file, 4);
        assert_eq!(parsed.to_rank, 1);
        assert_eq!(parsed.rescue_drop, Some(RescueOrDrop::Drop));
        assert_eq!(parsed.rescue_drop_file, Some(5));
        assert_eq!(parsed.rescue_drop_rank, Some(2));
        assert!(!parsed.is_capture);
    }

    #[test]
    fn test_rescue_drop_edge_cases() {
        // Test rescue to corner squares
        let parsed = PieceMoveParser::parse("Ra4Sa1").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Rook);
        assert_eq!(parsed.rescue_drop_file, Some(0));
        assert_eq!(parsed.rescue_drop_rank, Some(7));

        let parsed = PieceMoveParser::parse("Rh4Sh8").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Rook);
        assert_eq!(parsed.rescue_drop_file, Some(7));
        assert_eq!(parsed.rescue_drop_rank, Some(0));

        // Test drop to corner squares
        let parsed = PieceMoveParser::parse("Ra4Da1").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Rook);
        assert_eq!(parsed.rescue_drop_file, Some(0));
        assert_eq!(parsed.rescue_drop_rank, Some(7));

        let parsed = PieceMoveParser::parse("Rh4Dh8").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Rook);
        assert_eq!(parsed.rescue_drop_file, Some(7));
        assert_eq!(parsed.rescue_drop_rank, Some(0));
    }

    #[test]
    fn test_rescue_drop_errors() {
        // Test invalid rescue square
        assert!(PieceMoveParser::parse("e4Si9").is_err());

        // Test invalid drop square
        assert!(PieceMoveParser::parse("e4Dk9").is_err());
        assert!(PieceMoveParser::parse("e4Dj").is_err());

        // Test invalid rescue/drop markers
        assert!(PieceMoveParser::parse("e4Xa4").is_err());
        assert!(PieceMoveParser::parse("e4Ra4").is_err());

        // Test multiple rescue/drop markers
        assert!(PieceMoveParser::parse("e4Sa4Sd4").is_err());
        assert!(PieceMoveParser::parse("e4Da4Dd4").is_err());
        assert!(PieceMoveParser::parse("e4Sa4Dd4").is_err());
    }

    #[test]
    fn test_rescue_drop_with_disambiguation() {
        // Test rescue with file disambiguation
        let parsed = PieceMoveParser::parse("Nbd7Se5").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Knight);
        assert_eq!(parsed.from_file, Some(1));
        assert_eq!(parsed.to_file, 3);
        assert_eq!(parsed.to_rank, 1);
        assert_eq!(parsed.rescue_drop, Some(RescueOrDrop::Rescue));
        assert_eq!(parsed.rescue_drop_file, Some(4));
        assert_eq!(parsed.rescue_drop_rank, Some(3));

        // Test drop with rank disambiguation
        let parsed = PieceMoveParser::parse("R1e4Df6").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Rook);
        assert_eq!(parsed.from_rank, Some(7));
        assert_eq!(parsed.to_file, 4);
        assert_eq!(parsed.to_rank, 4);
        assert_eq!(parsed.rescue_drop, Some(RescueOrDrop::Drop));
        assert_eq!(parsed.rescue_drop_file, Some(5));
        assert_eq!(parsed.rescue_drop_rank, Some(2));
    }

    #[test]
    fn test_complex_edge_cases() {
        // Test full disambiguation with capture and rescue/drop
        let parsed = PieceMoveParser::parse("Nb1xe3Sf4").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Knight);
        assert_eq!(parsed.from_file, Some(1));
        assert_eq!(parsed.from_rank, Some(7));
        assert_eq!(parsed.to_file, 4);
        assert_eq!(parsed.to_rank, 5);
        assert!(parsed.is_capture);
        assert_eq!(parsed.rescue_drop, Some(RescueOrDrop::Rescue));
        assert_eq!(parsed.rescue_drop_file, Some(5));
        assert_eq!(parsed.rescue_drop_rank, Some(4));

        // Test multiple annotations after rescue/drop
        let parsed = PieceMoveParser::parse("Ra1h1Dg1?!+").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Rook);
        assert_eq!(parsed.from_file, Some(0));
        assert_eq!(parsed.from_rank, Some(7));
        assert_eq!(parsed.to_file, 7);
        assert_eq!(parsed.to_rank, 7);
        assert_eq!(parsed.rescue_drop, Some(RescueOrDrop::Drop));
        assert_eq!(parsed.rescue_drop_file, Some(6));
        assert_eq!(parsed.rescue_drop_rank, Some(7));

        // Test pawn moves with full source specification
        let parsed = PieceMoveParser::parse("e2e4").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Pawn);
        assert_eq!(parsed.from_file, Some(4));
        assert_eq!(parsed.from_rank, Some(6));
        assert_eq!(parsed.to_file, 4);
        assert_eq!(parsed.to_rank, 4);
    }

    #[test]
    fn test_error_edge_cases() {
        // Test invalid rescue/drop sequence
        assert!(PieceMoveParser::parse("e4SSf4").is_err()); // Double rescue
        assert!(PieceMoveParser::parse("e4DDf4").is_err()); // Double drop
        assert!(PieceMoveParser::parse("e4DSf4").is_err()); // Drop then rescue

        // Test invalid moves with captures
        assert!(PieceMoveParser::parse("Nxb1xe3").is_err()); // Double capture
        assert!(PieceMoveParser::parse("exd5xf6").is_err()); // Double capture with pawns
        assert!(PieceMoveParser::parse("xxe4").is_err()); // Double capture mark

        // // Test invalid piece disambiguation
        // assert!(PieceMoveParser::parse("N11e4").is_err()); // Double rank disambiguation
        // assert!(PieceMoveParser::parse("Nbbe4").is_err()); // Double file disambiguation
        // assert!(PieceMoveParser::parse("N1be4").is_err()); // Rank before file in disambiguation
    }

    #[test]
    fn test_partial_rescue_drop() {
        // Test incomplete rescue squares
        let parsed = PieceMoveParser::parse("e4S").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Pawn);
        assert_eq!(parsed.to_file, 4);
        assert_eq!(parsed.to_rank, 4);
        assert_eq!(parsed.rescue_drop, Some(RescueOrDrop::Rescue));
        assert_eq!(parsed.rescue_drop_file, None);
        assert_eq!(parsed.rescue_drop_rank, None);

        // Test rescue with only file
        let parsed = PieceMoveParser::parse("e4Sa").unwrap();
        assert_eq!(parsed.rescue_drop, Some(RescueOrDrop::Rescue));
        assert_eq!(parsed.rescue_drop_file, Some(0));
        assert_eq!(parsed.rescue_drop_rank, None);

        // Test rescue with only rank
        let parsed = PieceMoveParser::parse("e4S4").unwrap();
        assert_eq!(parsed.rescue_drop, Some(RescueOrDrop::Rescue));
        assert_eq!(parsed.rescue_drop_file, None);
        assert_eq!(parsed.rescue_drop_rank, Some(4));

        // Test drop variants
        let parsed = PieceMoveParser::parse("e4D").unwrap();
        assert_eq!(parsed.rescue_drop, Some(RescueOrDrop::Drop));
        assert_eq!(parsed.rescue_drop_file, None);
        assert_eq!(parsed.rescue_drop_rank, None);

        // Test with annotations
        let parsed = PieceMoveParser::parse("e4Sa+").unwrap();
        assert_eq!(parsed.rescue_drop, Some(RescueOrDrop::Rescue));
        assert_eq!(parsed.rescue_drop_file, Some(0));
        assert_eq!(parsed.rescue_drop_rank, None);

        let parsed = PieceMoveParser::parse("e4S4!").unwrap();
        assert_eq!(parsed.rescue_drop, Some(RescueOrDrop::Rescue));
        assert_eq!(parsed.rescue_drop_file, None);
        assert_eq!(parsed.rescue_drop_rank, Some(4));

        // Test with complex moves
        let parsed = PieceMoveParser::parse("Nb1xe3S").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Knight);
        assert_eq!(parsed.from_file, Some(1));
        assert_eq!(parsed.from_rank, Some(7));
        assert_eq!(parsed.to_file, 4);
        assert_eq!(parsed.to_rank, 5);
        assert!(parsed.is_capture);
        assert_eq!(parsed.rescue_drop, Some(RescueOrDrop::Rescue));
        assert_eq!(parsed.rescue_drop_file, None);
        assert_eq!(parsed.rescue_drop_rank, None);

        let parsed = PieceMoveParser::parse("Nb1xe3Da+!?").unwrap();
        assert_eq!(parsed.rescue_drop, Some(RescueOrDrop::Drop));
        assert_eq!(parsed.rescue_drop_file, Some(0));
        assert_eq!(parsed.rescue_drop_rank, None);
    }

    #[test]
    fn test_pathological_cases() {
        // Test extremely long move notation (but still valid)
        let parsed = PieceMoveParser::parse("Nb1xd2Sf3+!?").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Knight);
        assert_eq!(parsed.from_file, Some(1));
        assert_eq!(parsed.from_rank, Some(7));
        assert_eq!(parsed.to_file, 3);
        assert_eq!(parsed.to_rank, 6);
        assert!(parsed.is_capture);
        assert_eq!(parsed.rescue_drop, Some(RescueOrDrop::Rescue));
        assert_eq!(parsed.rescue_drop_file, Some(5));
        assert_eq!(parsed.rescue_drop_rank, Some(5));

        // Test valid moves with maximum annotations
        let parsed = PieceMoveParser::parse("Ra1xa8Dh1+!?#").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Rook);
        assert_eq!(parsed.from_file, Some(0));
        assert_eq!(parsed.from_rank, Some(7));
        assert_eq!(parsed.to_file, 0);
        assert_eq!(parsed.to_rank, 0);
        assert!(parsed.is_capture);
        assert_eq!(parsed.rescue_drop, Some(RescueOrDrop::Drop));
        assert_eq!(parsed.rescue_drop_file, Some(7));
        assert_eq!(parsed.rescue_drop_rank, Some(7));

        // Test edge case with file-only source and rescue/drop
        let parsed = PieceMoveParser::parse("Raxh8Sa1").unwrap();
        assert_eq!(parsed.piece_type, PieceType::Rook);
        assert_eq!(parsed.from_file, Some(0));
        assert_eq!(parsed.from_rank, None);
        assert_eq!(parsed.to_file, 7);
        assert_eq!(parsed.to_rank, 0);
        assert!(parsed.is_capture);
        assert_eq!(parsed.rescue_drop, Some(RescueOrDrop::Rescue));
        assert_eq!(parsed.rescue_drop_file, Some(0));
        assert_eq!(parsed.rescue_drop_rank, Some(7));
    }
}
