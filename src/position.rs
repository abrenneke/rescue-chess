use std::hash::Hash;

use crate::{
    bitboard::Bitboard,
    piece::{Color, PieceType},
    piece_move::{MoveType, PieceMove},
    pos::Pos,
};

use super::piece::Piece;

/// Records the castling rights that each player has at a point in the game. Once
/// a player moves their king, or the rook that is involved in castling, the
/// castling rights are removed.
#[derive(Clone, PartialEq, Copy, Hash, Eq)]
pub struct CastlingRights {
    /// Can white castle kingside
    pub white_king_side: bool,

    /// Can white castle queenside
    pub white_queen_side: bool,

    /// Can black castle kingside
    pub black_king_side: bool,

    /// Can black castle queenside
    pub black_queen_side: bool,
}

/// A game position in chess. Contains all state to represent a single position
/// in a game of chess.
#[derive(Clone, PartialEq, Eq)]
pub struct Position {
    /// The pieces on the board
    pub pieces: Vec<Piece>,

    // Active color is always white for our purposes
    pub castling_rights: CastlingRights,

    /// If Some, the position of the pawn that can be captured en passant
    pub en_passant: Option<Pos>,

    /// The number of halfmoves since the last capture or pawn move
    pub halfmove_clock: u8,

    /// The number of the full move. A full move is both players making a move.
    pub fullmove_number: u16,

    /// Optimized bitboard for quick lookups of if a position is occupied by a white piece.
    pub white_map: Bitboard,

    /// Optimized bitboard for quick lookups of if a position is occupied by a black piece.
    pub black_map: Bitboard,

    /// Quick lookup of the index of a piece in the pieces vector
    pub position_lookup: [Option<u8>; 64],
}

/// Given a list of pieces, returns a bitboard with the positions of the pieces for the given color.
fn color_as_bitboard(pieces: &[Piece], color: Color) -> Bitboard {
    pieces
        .iter()
        .filter(|piece| piece.color == color)
        .fold(Bitboard::new(), |acc, piece| acc.with(piece.position))
}

/// Calculates the position_lookup by iterating over the pieces and setting the
/// index of the piece in the pieces vector at the position of the piece.
fn calc_position_lookup(pieces: &[Piece]) -> [Option<u8>; 64] {
    let mut position_lookup = [None; 64];
    for (i, piece) in pieces.iter().enumerate() {
        position_lookup[piece.position.0 as usize] = Some(i as u8);
    }

    position_lookup
}

impl Position {
    /// Creates a new position by specifying all of the fields.
    pub fn new(
        pieces: Vec<Piece>,
        castling_rights: CastlingRights,
        en_passant: Option<Pos>,
        halfmove_clock: u8,
        fullmove_number: u16,
    ) -> Position {
        let white_map = color_as_bitboard(&pieces, Color::White);
        let black_map = color_as_bitboard(&pieces, Color::Black);
        let position_lookup = calc_position_lookup(&pieces);

        Position {
            pieces,
            castling_rights,
            en_passant,
            halfmove_clock,
            fullmove_number,
            white_map,
            black_map,
            position_lookup,
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

        self.calc_changes();
    }

    /// When any piece has changed, this function should be called to
    /// recalculate the bitboards and position lookup.
    fn calc_changes(&mut self) {
        self.white_map = color_as_bitboard(&self.pieces, Color::White);
        self.black_map = color_as_bitboard(&self.pieces, Color::Black);
        self.position_lookup = calc_position_lookup(&self.pieces);
    }

    /// Returns a new GamePosition with the colors and board flipped.
    pub fn inverted(&self) -> Position {
        let mut position = self.clone();
        position.invert();
        position
    }

    /// Gets the piece at a specific position, if any.
    pub fn get_piece_at(&self, position: Pos) -> Option<&Piece> {
        if let Some(index) = self.position_lookup[position.0 as usize] {
            Some(&self.pieces[index as usize])
        } else {
            None
        }
    }

    /// Gets the piece at a specific position, if any, mutably.
    pub fn get_piece_at_mut(&mut self, position: Pos) -> Option<&mut Piece> {
        if let Some(index) = self.position_lookup[position.0 as usize] {
            Some(&mut self.pieces[index as usize])
        } else {
            None
        }
    }

    /// Moves a piece from one position to another.
    pub fn move_piece(&mut self, from: Pos, to: Pos) {
        self.get_piece_at_mut(from).unwrap().position = to;
        self.calc_changes();
    }

    /// Removes the piece at a specific position.
    pub fn remove_piece_at(&mut self, position: Pos) {
        self.pieces.retain(|p| p.position != position);
        self.calc_changes();
    }

    /// Adds a piece to the board.
    pub fn add_piece(&mut self, piece: Piece) {
        self.pieces.push(piece);
        self.calc_changes();
    }

    /// Returns true if white is in checkmate. Returns an error if the position is invalid (no king)
    pub fn is_checkmate(&self) -> Result<bool, anyhow::Error> {
        Ok(self.is_king_in_check()? && self.get_all_legal_moves()?.is_empty())
    }

    /// Returns true if the white king is currently in check. Returns an error if there is no king.
    pub fn is_king_in_check(&self) -> Result<bool, anyhow::Error> {
        // TODO probably more efficient to check from the king position
        let king = self
            .pieces
            .iter()
            .find(|piece| piece.piece_type == PieceType::King && piece.color == Color::White);

        match king {
            Some(king) => {
                let king_position = king.position;

                let black_moves = self.inverted().get_all_moves_unchecked();

                for mv in black_moves {
                    if mv.to.invert() == king_position {
                        return Ok(true);
                    }
                }

                Ok(false)
            }
            None => Err(anyhow::anyhow!(
                "No white king found! \n {}",
                self.to_board_string()
            )),
        }
    }

    /// Gets all legal moves for the current position. Takes into account
    /// whether the king is in check, etc.
    pub fn get_all_legal_moves(&self) -> Result<Vec<PieceMove>, anyhow::Error> {
        let possible_moves = self.get_all_moves_unchecked();
        let mut moves = Vec::with_capacity(possible_moves.len());

        for mv in possible_moves.into_iter() {
            let mut new_position = self.clone();
            new_position.apply_move(mv)?;

            if !new_position.is_king_in_check()? {
                moves.push(mv);
            }
        }

        Ok(moves)
    }

    /// Gets all moves that are possible by white, without checking for
    /// check, use this to check whether a king is in check, etc.
    pub fn get_all_moves_unchecked(&self) -> Vec<PieceMove> {
        let mut moves = Vec::new();

        for piece in self.pieces.iter().filter(|p| p.color == Color::White) {
            let legal_moves = piece.get_legal_moves(self.white_map, self.black_map);

            for to in legal_moves.iter() {
                if self.white_map.get(to) {
                    panic!("Illegal move");
                }

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

    /// Prints the board as ASCII characters.
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

    pub fn apply_move(&mut self, mv: PieceMove) -> Result<(), anyhow::Error> {
        let piece = self
            .get_piece_at(mv.from)
            .ok_or(anyhow::anyhow!("No piece at position"))?;

        let legal_moves = piece.get_legal_moves(self.white_map, self.black_map);

        if !legal_moves.get(mv.to) {
            return Err(anyhow::anyhow!("Illegal move"));
        }

        match mv.move_type {
            MoveType::Unknown => {
                return Err(anyhow::anyhow!("Unknown move type"));
            }
            MoveType::Normal => {
                self.move_piece(mv.from, mv.to);
            }
            MoveType::Capture(_) => {
                self.remove_piece_at(mv.to);
                self.move_piece(mv.from, mv.to);
            }
            MoveType::EnPassant(pos) => {
                self.remove_piece_at(pos);
                self.move_piece(mv.from, mv.to);
            }
            MoveType::Castle(_king_pos, _rook_pos) => {
                todo!();
            }
            MoveType::Promotion(piece_type) => {
                self.move_piece(mv.from, mv.to);
                self.get_piece_at_mut(mv.from).unwrap().piece_type = piece_type;
            }
            MoveType::CapturePromotion(_promotion, _captured) => {
                todo!();
            }
        }

        Ok(())
    }

    pub fn unapply_move(&mut self, mv: PieceMove) -> Result<(), anyhow::Error> {
        let piece = self
            .get_piece_at(mv.to)
            .ok_or(anyhow::anyhow!("No piece at position"))?;

        let color = piece.color;

        match mv.move_type {
            MoveType::Unknown => {
                return Err(anyhow::anyhow!("Unknown move type"));
            }
            MoveType::Normal => {
                self.get_piece_at_mut(mv.to).unwrap().position = mv.from;
            }
            MoveType::Capture(captured) => {
                self.get_piece_at_mut(mv.to).unwrap().position = mv.from;
                self.add_piece(Piece::new(captured, color, mv.to));
            }
            MoveType::EnPassant(pos) => {
                self.get_piece_at_mut(mv.to).unwrap().position = mv.from;
                self.add_piece(Piece::new(PieceType::Pawn, color, pos));
            }
            MoveType::Castle(_king_pos, _rook_pos) => {
                todo!();
            }
            MoveType::Promotion(_) => {
                self.get_piece_at_mut(mv.to).unwrap().position = mv.from;
                self.get_piece_at_mut(mv.to).unwrap().piece_type = PieceType::Pawn;
            }
            MoveType::CapturePromotion(_, _) => {
                todo!();
            }
        }

        self.calc_changes();

        Ok(())
    }
}

impl std::hash::Hash for Position {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let mut board = [None; 64];

        for piece in self.pieces.iter() {
            board[piece.position.0 as usize] = Some((piece.piece_type, piece.color));
        }

        board.hash(state);
        self.castling_rights.hash(state);
        self.en_passant.hash(state);
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
        assert!(!position.is_king_in_check().unwrap());

        let position: Position = "1N3r2/4P3/2pP3p/2P2P2/3K1k2/2p1p3/3BBq2/2R5 w - - 0 1".into();
        assert!(!position.is_king_in_check().unwrap());

        let position: Position = "8/nR2Q2P/P2P2kb/4B2b/4K3/1r2P2P/5p2/1r6 w - - 0 1".into();
        assert!(!position.is_king_in_check().unwrap());

        let position: Position = "4B1r1/7q/rk4pP/4n3/1Np5/1p1P1R2/P1Q2K2/8 w - - 0 1".into();
        assert!(!position.is_king_in_check().unwrap());
    }

    #[test]
    fn king_in_check() {
        let position: Position = "rnb1kbnr/pppppppp/3q4/8/3K4/8/PPPPPPPP/RNBQ1BNR w - - 0 1".into();
        assert!(position.is_king_in_check().unwrap());

        let position: Position = "rnbqk1nr/pppppppp/5b2/8/3K4/8/PPPPPPPP/RNBQ1BNR".into();
        assert!(position.is_king_in_check().unwrap());

        let position: Position = "rnbqkb1r/pppppppp/4n3/8/3K4/8/PPPPPPPP/RNBQ1BNR".into();
        assert!(position.is_king_in_check().unwrap());

        let position: Position = "rnbqkbnr/pppp1ppp/8/4p3/3K4/8/PPPPPPPP/RNBQ1BNR".into();
        assert!(position.is_king_in_check().unwrap());
    }

    #[test]
    fn legal_moves_puts_king_in_check() {
        let position: Position = "8/8/8/3r4/3R4/3K4/8/8".into();

        println!("{}", position.to_board_string());

        let moves = position.get_all_legal_moves().unwrap();

        for mv in moves {
            println!("{}", mv);
        }
    }
}
