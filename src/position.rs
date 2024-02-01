use crate::{
    bitboard::Bitboard,
    piece::{Color, PieceType},
    piece_move::{MoveType, PieceMove},
    pos::Pos,
};

use super::piece::Piece;

#[derive(Clone, PartialEq, Copy)]
pub struct CastlingRights {
    pub white_king_side: bool,
    pub white_queen_side: bool,
    pub black_king_side: bool,
    pub black_queen_side: bool,
}

#[derive(Clone, PartialEq)]
pub struct Position {
    pub pieces: Vec<Piece>,
    // Active color is always white for our purposes
    pub castling_rights: CastlingRights,
    pub en_passant: Option<Pos>,
    pub halfmove_clock: u8,
    pub fullmove_number: u16,

    pub white_map: Bitboard,
    pub black_map: Bitboard,
}

fn color_as_bitboard(pieces: &[Piece], color: Color) -> Bitboard {
    pieces
        .iter()
        .filter(|piece| piece.color == color)
        .fold(Bitboard::new(), |acc, piece| acc.with(piece.position))
}

impl Position {
    pub fn new(
        pieces: Vec<Piece>,
        castling_rights: CastlingRights,
        en_passant: Option<Pos>,
        halfmove_clock: u8,
        fullmove_number: u16,
    ) -> Position {
        let white_map = color_as_bitboard(&pieces, Color::White);
        let black_map = color_as_bitboard(&pieces, Color::Black);

        Position {
            pieces,
            castling_rights,
            en_passant,
            halfmove_clock,
            fullmove_number,
            white_map,
            black_map,
        }
    }

    /// Inverts the position, i.e. makes the black pieces white and vice versa.
    /// The board will be flipped as well, i.e. a1 will become h8 and so on.
    pub fn invert(&mut self) {
        for piece in self.pieces.iter_mut() {
            piece.color = match piece.color {
                Color::White => Color::Black,
                Color::Black => Color::White,
            };

            piece.position = piece.position.invert();
        }

        self.white_map = color_as_bitboard(&self.pieces, Color::White);
        self.black_map = color_as_bitboard(&self.pieces, Color::Black);
    }

    /// Returns a new GamePosition with the colors and board flipped.
    pub fn inverted(&self) -> Position {
        let mut position = self.clone();
        position.invert();
        position
    }

    pub fn get_piece_at(&self, position: Pos) -> Option<&Piece> {
        self.pieces.iter().find(|piece| piece.position == position)
    }

    pub fn get_piece_at_mut(&mut self, position: Pos) -> Option<&mut Piece> {
        self.pieces
            .iter_mut()
            .find(|piece| piece.position == position)
    }

    pub fn remove_piece(&mut self, piece: &Piece) {
        self.pieces.retain(|p| p != piece);
    }

    pub fn remove_piece_at(&mut self, position: Pos) {
        self.pieces.retain(|p| p.position != position);
    }

    pub fn add_piece(&mut self, piece: Piece) {
        self.pieces.push(piece);
    }

    pub fn is_checkmate(&self) -> bool {
        self.is_king_in_check() && self.get_all_legal_moves().is_empty()
    }

    pub fn is_king_in_check(&self) -> bool {
        // TODO probably more efficient to check from the king position
        let king = self
            .pieces
            .iter()
            .find(|piece| piece.piece_type == PieceType::King && piece.color == Color::White)
            .expect("King was captured!!");

        let king_position = king.position;

        let black_moves = self.inverted().get_all_moves_unchecked();

        for mv in black_moves {
            if mv.to.invert() == king_position {
                return true;
            }
        }

        false
    }

    /// Gets all legal moves for the current position. Takes into account
    /// whether the king is in check, etc.
    pub fn get_all_legal_moves(&self) -> Vec<PieceMove> {
        let possible_moves = self.get_all_moves_unchecked();

        possible_moves
            .into_iter()
            .filter(|mv| {
                let mut new_position = self.clone();
                new_position.apply_move(*mv).is_ok() && !new_position.is_king_in_check()
            })
            .collect()
    }

    /// Gets all moves that are possible by white, without checking for
    /// check, use this to check whether a king is in check, etc.
    pub fn get_all_moves_unchecked(&self) -> Vec<PieceMove> {
        let mut moves = Vec::new();

        for piece in self.pieces.iter().filter(|p| p.color == Color::White) {
            let legal_moves = piece.get_legal_moves(self.white_map, self.black_map);

            for to in legal_moves.iter() {
                if self.black_map.get(to) {
                    moves.push(PieceMove {
                        from: piece.position,
                        to,
                        move_type: MoveType::Capture(self.get_piece_at(to).unwrap().piece_type),
                        piece_type: piece.piece_type,
                    });
                } else {
                    moves.push(PieceMove {
                        from: piece.position,
                        to,
                        move_type: MoveType::Normal,
                        piece_type: piece.piece_type,
                    });
                }
            }
        }

        moves
    }

    pub fn to_board_string(&self) -> String {
        let mut board = [[None; 8]; 8];

        for piece in self.pieces.iter() {
            let (x, y) = piece.position.as_tuple();
            board[y as usize][x as usize] = Some(piece);
        }

        let mut board_string = String::new();

        for rank in 0..8 {
            for file in 0..8 {
                match board[rank][file] {
                    Some(piece) => {
                        board_string.push_str(&piece.to_string());
                    }
                    None => {
                        board_string.push('.');
                    }
                }
            }

            board_string.push('\n');
        }

        board_string
    }
}

impl std::str::FromStr for Position {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Position::parse_from_fen(s)
    }
}

impl std::convert::From<&'static str> for Position {
    fn from(s: &'static str) -> Self {
        Position::parse_from_fen(s).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::Position;

    #[test]
    pub fn parse_fen_1() {
        let position: Position = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".into();

        println!("{}", position.to_board_string());

        assert_eq!(
            position.to_fen(),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );

        let position: Position =
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 1 2".into();

        assert_eq!(
            position.to_fen(),
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 1 2",
        );
    }

    #[test]
    pub fn king_not_in_check() {
        let position: Position = "8/8/8/8/8/8/8/4K3 w - - 0 1".into();
        assert!(!position.is_king_in_check());

        let position: Position = "1N3r2/4P3/2pP3p/2P2P2/3K1k2/2p1p3/3BBq2/2R5 w - - 0 1".into();
        assert!(!position.is_king_in_check());

        let position: Position = "8/nR2Q2P/P2P2kb/4B2b/4K3/1r2P2P/5p2/1r6 w - - 0 1".into();
        assert!(!position.is_king_in_check());

        let position: Position = "4B1r1/7q/rk4pP/4n3/1Np5/1p1P1R2/P1Q2K2/8 w - - 0 1".into();
        assert!(!position.is_king_in_check());
    }

    #[test]
    fn king_in_check() {
        let position: Position = "rnb1kbnr/pppppppp/3q4/8/3K4/8/PPPPPPPP/RNBQ1BNR w - - 0 1".into();
        assert!(position.is_king_in_check());

        let position: Position = "rnbqk1nr/pppppppp/5b2/8/3K4/8/PPPPPPPP/RNBQ1BNR".into();
        assert!(position.is_king_in_check());

        let position: Position = "rnbqkb1r/pppppppp/4n3/8/3K4/8/PPPPPPPP/RNBQ1BNR".into();
        assert!(position.is_king_in_check());

        let position: Position = "rnbqkbnr/pppp1ppp/8/4p3/3K4/8/PPPPPPPP/RNBQ1BNR".into();
        assert!(position.is_king_in_check());
    }

    #[test]
    fn legal_moves_puts_king_in_check() {
        let position: Position = "8/8/8/3r4/3R4/3K4/8/8".into();

        println!("{}", position.to_board_string());

        let moves = position.get_all_legal_moves();

        for mv in moves {
            println!("{}", mv);
        }
    }
}
