use crate::{
    bitboard::Bitboard,
    piece::{Color, PieceType},
};

use super::piece::Piece;

pub struct Board {
    pub pieces: Vec<Piece>,
}

impl Board {
    pub fn color_as_bitboard(&self, color: Color) -> Bitboard {
        self.pieces
            .iter()
            .filter(|piece| piece.color == color)
            .fold(Bitboard::new(), |acc, piece| acc.with(piece.position))
    }

    pub fn parse_from_fen(notation: &str) -> Result<Board, anyhow::Error> {
        let mut pieces = Vec::new();
        let mut position = 0;

        for character in notation.chars() {
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

        Ok(Board { pieces })
    }
}

impl std::str::FromStr for Board {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Board::parse_from_fen(s)
    }
}
