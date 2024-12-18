use crate::{position::CastlingRights, Color, Piece, PieceType, Pos, Position};

/// Parses a position from FEN notation.
///
/// Note that this can only parse when the active color is white, because
/// the engine only supports playing as white.
///
/// # Example
///
/// ```
/// use rescue_chess::Position;
///
/// let position = Position::parse_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
/// ```
pub fn parse_position_from_fen(notation: &str) -> Result<Position, anyhow::Error> {
    let mut pieces: Vec<Piece> = Vec::new();
    let mut position = Pos(0);

    let mut castling_rights = CastlingRights {
        white_king_side: false,
        white_queen_side: false,
        black_king_side: false,
        black_queen_side: false,
    };

    let mut notation = notation.split_whitespace();

    let piece_placement = notation
        .next()
        .ok_or_else(|| anyhow::anyhow!("FEN notation must contain piece placement information"))?;

    let mut holding = false;

    for character in piece_placement.chars() {
        match character {
            'P' => {
                if holding {
                    pieces.last_mut().unwrap().holding = Some(PieceType::Pawn);
                    holding = false;
                } else {
                    pieces.push(Piece {
                        piece_type: PieceType::Pawn,
                        color: Color::White,
                        position,
                        holding: None,
                    });
                }

                position += 1;
            }
            'N' => {
                if holding {
                    pieces.last_mut().unwrap().holding = Some(PieceType::Knight);
                    holding = false;
                } else {
                    pieces.push(Piece {
                        piece_type: PieceType::Knight,
                        color: Color::White,
                        position,
                        holding: None,
                    });
                }
                position += 1;
            }
            'B' => {
                if holding {
                    pieces.last_mut().unwrap().holding = Some(PieceType::Bishop);
                    holding = false;
                } else {
                    pieces.push(Piece {
                        piece_type: PieceType::Bishop,
                        color: Color::White,
                        position,
                        holding: None,
                    });
                }
                position += 1;
            }
            'R' => {
                if holding {
                    pieces.last_mut().unwrap().holding = Some(PieceType::Rook);
                    holding = false;
                } else {
                    pieces.push(Piece {
                        piece_type: PieceType::Rook,
                        color: Color::White,
                        position,
                        holding: None,
                    });
                }
                position += 1;
            }
            'Q' => {
                if holding {
                    pieces.last_mut().unwrap().holding = Some(PieceType::Queen);
                    holding = false;
                } else {
                    pieces.push(Piece {
                        piece_type: PieceType::Queen,
                        color: Color::White,
                        position,
                        holding: None,
                    });
                }
                position += 1;
            }
            'K' => {
                if holding {
                    pieces.last_mut().unwrap().holding = Some(PieceType::King);
                    holding = false;
                } else {
                    pieces.push(Piece {
                        piece_type: PieceType::King,
                        color: Color::White,
                        position,
                        holding: None,
                    });
                }
                position += 1;
            }
            'p' => {
                if holding {
                    pieces.last_mut().unwrap().holding = Some(PieceType::Pawn);
                    holding = false;
                } else {
                    pieces.push(Piece {
                        piece_type: PieceType::Pawn,
                        color: Color::Black,
                        position,
                        holding: None,
                    });
                }
                position += 1;
            }
            'n' => {
                if holding {
                    pieces.last_mut().unwrap().holding = Some(PieceType::Knight);
                    holding = false;
                } else {
                    pieces.push(Piece {
                        piece_type: PieceType::Knight,
                        color: Color::Black,
                        position,
                        holding: None,
                    });
                }
                position += 1;
            }
            'b' => {
                if holding {
                    pieces.last_mut().unwrap().holding = Some(PieceType::Bishop);
                    holding = false;
                } else {
                    pieces.push(Piece {
                        piece_type: PieceType::Bishop,
                        color: Color::Black,
                        position,
                        holding: None,
                    });
                }
                position += 1;
            }
            'r' => {
                if holding {
                    pieces.last_mut().unwrap().holding = Some(PieceType::Rook);
                    holding = false;
                } else {
                    pieces.push(Piece {
                        piece_type: PieceType::Rook,
                        color: Color::Black,
                        position,
                        holding: None,
                    });
                }
                position += 1;
            }
            'q' => {
                if holding {
                    pieces.last_mut().unwrap().holding = Some(PieceType::Queen);
                    holding = false;
                } else {
                    pieces.push(Piece {
                        piece_type: PieceType::Queen,
                        color: Color::Black,
                        position,
                        holding: None,
                    });
                }
                position += 1;
            }
            'k' => {
                if holding {
                    pieces.last_mut().unwrap().holding = Some(PieceType::King);
                    holding = false;
                } else {
                    pieces.push(Piece {
                        piece_type: PieceType::King,
                        color: Color::Black,
                        position,
                        holding: None,
                    });
                }
                position += 1;
            }
            'x' => {
                holding = true;
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
    let mut active_color = Color::White;

    if let Some(active_color_str) = notation.next() {
        active_color = match active_color_str {
            "w" => Color::White,
            "b" => Color::Black,
            _ => {
                return Err(anyhow::anyhow!("Invalid active color in FEN notation"));
            }
        };

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

        let en_passant_str = notation
            .next()
            .ok_or_else(|| anyhow::anyhow!("FEN notation must contain en passant information"))?;

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

    let mut position = Position::new(
        pieces,
        castling_rights,
        en_passant,
        halfmove_clock,
        fullmove_number,
    );

    if active_color == Color::White {
        Ok(position)
    } else {
        position.invert();
        Ok(position)
    }
}

/// Converts a piece type and color to FEN notation.
fn piece_type_to_fen(piece_type: PieceType, color: Color) -> char {
    match piece_type {
        PieceType::Pawn => match color {
            Color::White => 'P',
            Color::Black => 'p',
        },
        PieceType::Knight => match color {
            Color::White => 'N',
            Color::Black => 'n',
        },
        PieceType::Bishop => match color {
            Color::White => 'B',
            Color::Black => 'b',
        },
        PieceType::Rook => match color {
            Color::White => 'R',
            Color::Black => 'r',
        },
        PieceType::Queen => match color {
            Color::White => 'Q',
            Color::Black => 'q',
        },
        PieceType::King => match color {
            Color::White => 'K',
            Color::Black => 'k',
        },
    }
}

/// Converts the position to FEN notation.
///
/// # Example
///
/// ```
/// use rescue_chess::Position;
///
/// let position = Position::start_position();
/// let fen = position.to_fen();
/// assert_eq!(fen, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
/// ```
pub fn position_to_fen(position: &Position) -> String {
    let mut position = position.clone();

    let true_active_color = position.true_active_color;

    if true_active_color == Color::Black {
        // It's actually black's turn, so we need to invert the position
        position = position.inverted();
    }

    let mut fen = String::new();

    for rank in 0..8 {
        let mut empty = 0;
        for file in 0..8 {
            let pos = Pos::xy(file, rank);

            let piece = position.get_piece_at(pos);

            if let Some(piece) = piece {
                if empty > 0 {
                    fen.push_str(&empty.to_string());
                    empty = 0;
                }

                let character = piece_type_to_fen(piece.piece_type, piece.color);

                fen.push(character);

                if let Some(holding) = piece.holding {
                    fen.push('x');
                    fen.push(piece_type_to_fen(holding, piece.color))
                }
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

    if true_active_color == Color::White {
        fen.push('w');
    } else {
        fen.push('b');
    }

    fen.push(' ');

    if position.castling_rights.white_king_side {
        fen.push('K');
    }

    if position.castling_rights.white_queen_side {
        fen.push('Q');
    }

    if position.castling_rights.black_king_side {
        fen.push('k');
    }

    if position.castling_rights.black_queen_side {
        fen.push('q');
    }

    if !position.castling_rights.white_king_side
        && !position.castling_rights.white_queen_side
        && !position.castling_rights.black_king_side
        && !position.castling_rights.black_queen_side
    {
        fen.push('-');
    }

    fen.push(' ');

    if let Some(en_passant) = position.en_passant {
        fen.push_str(&en_passant.to_algebraic());
    } else {
        fen.push('-');
    }

    fen.push(' ');

    fen.push_str(&position.halfmove_clock.to_string());

    fen.push(' ');

    fen.push_str(&position.fullmove_number.to_string());

    fen
}
