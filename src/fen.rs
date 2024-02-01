use crate::{position::CastlingRights, Color, Piece, PieceType, Pos, Position};

impl Position {
    // TODO Extended Position Description instead?
    pub fn parse_from_fen(notation: &str) -> Result<Position, anyhow::Error> {
        let mut pieces = Vec::new();
        let mut position = Pos(0);

        let mut castling_rights = CastlingRights {
            white_king_side: false,
            white_queen_side: false,
            black_king_side: false,
            black_queen_side: false,
        };

        let mut notation = notation.split_whitespace();

        let piece_placement = notation.next().ok_or_else(|| {
            anyhow::anyhow!("FEN notation must contain piece placement information")
        })?;

        for character in piece_placement.chars() {
            match character {
                'P' => {
                    pieces.push(Piece {
                        piece_type: PieceType::Pawn,
                        color: Color::White,
                        position,
                    });
                    position += 1;
                }
                'N' => {
                    pieces.push(Piece {
                        piece_type: PieceType::Knight,
                        color: Color::White,
                        position,
                    });
                    position += 1;
                }
                'B' => {
                    pieces.push(Piece {
                        piece_type: PieceType::Bishop,
                        color: Color::White,
                        position,
                    });
                    position += 1;
                }
                'R' => {
                    pieces.push(Piece {
                        piece_type: PieceType::Rook,
                        color: Color::White,
                        position,
                    });
                    position += 1;
                }
                'Q' => {
                    pieces.push(Piece {
                        piece_type: PieceType::Queen,
                        color: Color::White,
                        position,
                    });
                    position += 1;
                }
                'K' => {
                    pieces.push(Piece {
                        piece_type: PieceType::King,
                        color: Color::White,
                        position,
                    });
                    position += 1;
                }
                'p' => {
                    pieces.push(Piece {
                        piece_type: PieceType::Pawn,
                        color: Color::Black,
                        position,
                    });
                    position += 1;
                }
                'n' => {
                    pieces.push(Piece {
                        piece_type: PieceType::Knight,
                        color: Color::Black,
                        position,
                    });
                    position += 1;
                }
                'b' => {
                    pieces.push(Piece {
                        piece_type: PieceType::Bishop,
                        color: Color::Black,
                        position,
                    });
                    position += 1;
                }
                'r' => {
                    pieces.push(Piece {
                        piece_type: PieceType::Rook,
                        color: Color::Black,
                        position,
                    });
                    position += 1;
                }
                'q' => {
                    pieces.push(Piece {
                        piece_type: PieceType::Queen,
                        color: Color::Black,
                        position,
                    });
                    position += 1;
                }
                'k' => {
                    pieces.push(Piece {
                        piece_type: PieceType::King,
                        color: Color::Black,
                        position,
                    });
                    position += 1;
                }
                '1'..='8' => {
                    position += character.to_digit(10).unwrap() as u8;
                }
                '/' => {}
                _ => {
                    return Err(anyhow::anyhow!("Invalid character in FEN notation"));
                }
            }
        }

        let mut en_passant = None;
        let mut halfmove_clock = 0;
        let mut fullmove_number = 1;

        if let Some(active_color_str) = notation.next() {
            if active_color_str != "w" {
                return Err(anyhow::anyhow!("Only white active color is supported"));
            }

            let castling = notation.next().ok_or_else(|| {
                anyhow::anyhow!("FEN notation must contain castling rights information")
            })?;

            for character in castling.chars() {
                match character {
                    'K' => castling_rights.white_king_side = true,
                    'Q' => castling_rights.white_queen_side = true,
                    'k' => castling_rights.black_king_side = true,
                    'q' => castling_rights.black_queen_side = true,
                    '-' => break,
                    _ => {
                        return Err(anyhow::anyhow!(
                            "Invalid character in castling rights information"
                        ));
                    }
                }
            }

            let en_passant_str = notation.next().ok_or_else(|| {
                anyhow::anyhow!("FEN notation must contain en passant information")
            })?;

            if en_passant_str != "-" {
                en_passant = Some(Pos::from_algebraic(en_passant_str)?);
            }

            let halfmove_clock_str = notation.next().ok_or_else(|| {
                anyhow::anyhow!("FEN notation must contain halfmove clock information")
            })?;

            halfmove_clock = halfmove_clock_str.parse()?;

            let fullmove_number_str = notation.next().ok_or_else(|| {
                anyhow::anyhow!("FEN notation must contain fullmove number information")
            })?;

            fullmove_number = fullmove_number_str.parse()?;
        }

        Ok(Position::new(
            pieces,
            castling_rights,
            en_passant,
            halfmove_clock,
            fullmove_number,
        ))
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        for rank in 0..8 {
            let mut empty = 0;
            for file in 0..8 {
                let pos = Pos::xy(file, rank);
                let piece = self.pieces.iter().find(|piece| piece.position == pos);

                if let Some(piece) = piece {
                    if empty > 0 {
                        fen.push_str(&empty.to_string());
                        empty = 0;
                    }

                    let character = match piece.piece_type {
                        PieceType::Pawn => match piece.color {
                            Color::White => 'P',
                            Color::Black => 'p',
                        },
                        PieceType::Knight => match piece.color {
                            Color::White => 'N',
                            Color::Black => 'n',
                        },
                        PieceType::Bishop => match piece.color {
                            Color::White => 'B',
                            Color::Black => 'b',
                        },
                        PieceType::Rook => match piece.color {
                            Color::White => 'R',
                            Color::Black => 'r',
                        },
                        PieceType::Queen => match piece.color {
                            Color::White => 'Q',
                            Color::Black => 'q',
                        },
                        PieceType::King => match piece.color {
                            Color::White => 'K',
                            Color::Black => 'k',
                        },
                    };

                    fen.push(character);
                } else {
                    empty += 1;
                }
            }

            if empty > 0 {
                fen.push_str(&empty.to_string());
            }

            if rank < 7 {
                fen.push('/');
            }
        }

        fen.push(' ');

        fen.push_str("w ");

        if self.castling_rights.white_king_side {
            fen.push('K');
        }

        if self.castling_rights.white_queen_side {
            fen.push('Q');
        }

        if self.castling_rights.black_king_side {
            fen.push('k');
        }

        if self.castling_rights.black_queen_side {
            fen.push('q');
        }

        if !self.castling_rights.white_king_side
            && !self.castling_rights.white_queen_side
            && !self.castling_rights.black_king_side
            && !self.castling_rights.black_queen_side
        {
            fen.push('-');
        }

        fen.push(' ');

        if let Some(en_passant) = self.en_passant {
            fen.push_str(&en_passant.to_algebraic());
        } else {
            fen.push('-');
        }

        fen.push(' ');

        fen.push_str(&self.halfmove_clock.to_string());

        fen.push(' ');

        fen.push_str(&self.fullmove_number.to_string());

        fen
    }
}
