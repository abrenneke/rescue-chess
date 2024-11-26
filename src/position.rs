mod fen;

use std::hash::Hash;

use crate::{
    bitboard::Bitboard,
    piece::{Color, PieceType},
    piece_move::{GameType, MoveType, PieceMove},
    pos::Pos,
};

use super::piece::Piece;

/// Records the castling rights that each player has at a point in the game. Once
/// a player moves their king, or the rook that is involved in castling, the
/// castling rights are removed.
#[derive(Debug, Clone, PartialEq, Copy, Hash, Eq)]
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
#[derive(Debug, Clone, Eq)]
pub struct Position {
    /// The pieces on the board
    pub pieces: arrayvec::ArrayVec<Piece, 32>,

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
            pieces: pieces.into_iter().collect(),
            castling_rights,
            en_passant,
            halfmove_clock,
            fullmove_number,
            white_map,
            black_map,
            position_lookup,
        }
    }

    /// Returns the start position of a chess game.
    pub fn start_position() -> Self {
        return Self::parse_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .unwrap();
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
    pub fn calc_changes(&mut self) {
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

    pub fn rescue_piece(&mut self, rescuer: Pos, rescued: Pos) -> Result<(), anyhow::Error> {
        let rescuer_piece = match self.get_piece_at(rescuer) {
            Some(rescuer) => rescuer,
            None => {
                return Err(anyhow::anyhow!(
                    "No piece at rescuer position {}",
                    rescuer.to_algebraic()
                ))
            }
        };

        let rescued_piece = match self.get_piece_at(rescued) {
            Some(rescued) => rescued,
            None => {
                return Err(anyhow::anyhow!(
                    "No piece at rescued position {}",
                    rescued.to_algebraic()
                ))
            }
        };

        if let Some(_) = rescuer_piece.holding {
            return Err(anyhow::anyhow!("Rescuer already holding a piece"));
        }

        if rescuer_piece.piece_type.can_hold(rescued_piece.piece_type) {
            self.get_piece_at_mut(rescuer).unwrap().holding = Some(rescued_piece.piece_type);
            self.remove_piece_at(rescued)?;

            Ok(())
        } else {
            Err(anyhow::anyhow!("Rescuer cannot hold rescued piece"))
        }
    }

    pub fn drop_piece(&mut self, rescuer_pos: Pos, drop_pos: Pos) -> Result<(), anyhow::Error> {
        let rescuer = match self.get_piece_at(rescuer_pos) {
            Some(rescuer) => rescuer,
            None => return Err(anyhow::anyhow!("No piece at rescuer position")),
        };

        let holding_type = match rescuer.holding {
            Some(holding_type) => holding_type,
            None => return Err(anyhow::anyhow!("Rescuer not holding a piece")),
        };

        if self.black_map.get(drop_pos) || self.white_map.get(drop_pos) {
            return Err(anyhow::anyhow!("Position occupied"));
        }

        self.add_piece(Piece::new(holding_type, rescuer.color, drop_pos))?;

        self.get_piece_at_mut(rescuer_pos).unwrap().holding = None;

        Ok(())
    }

    /// Moves a piece from one position to another.
    pub fn move_piece(&mut self, from: Pos, to: Pos) -> Result<(), anyhow::Error> {
        if self.white_map.get(to) || self.black_map.get(to) {
            return Err(anyhow::anyhow!("Position occupied"));
        }

        self.get_piece_at_mut(from).unwrap().position = to;
        self.calc_changes();

        Ok(())
    }

    /// Removes the piece at a specific position.
    pub fn remove_piece_at(&mut self, position: Pos) -> Result<(), anyhow::Error> {
        if let Some(index) = self.position_lookup[position.0 as usize] {
            self.pieces.remove(index as usize);
            self.calc_changes();
            Ok(())
        } else {
            Err(anyhow::anyhow!("No piece at position"))
        }
    }

    /// Adds a piece to the board.
    pub fn add_piece(&mut self, piece: Piece) -> Result<(), anyhow::Error> {
        if self.white_map.get(piece.position) || self.black_map.get(piece.position) {
            return Err(anyhow::anyhow!("Position occupied"));
        }

        self.pieces.push(piece);
        self.calc_changes();

        Ok(())
    }

    /// Returns true if white is in checkmate. Returns an error if the position is invalid (no king)
    pub fn is_checkmate(&self, game_type: GameType) -> Result<bool, anyhow::Error> {
        Ok(self.is_king_in_check(game_type)? && self.get_all_legal_moves(game_type)?.is_empty())
    }

    /// Returns true if the white king is currently in check. Returns an error if there is no king.
    pub fn is_king_in_check(&self, game_type: GameType) -> Result<bool, anyhow::Error> {
        // TODO probably more efficient to check from the king position
        let king = self
            .pieces
            .iter()
            .find(|piece| piece.piece_type == PieceType::King && piece.color == Color::White);

        match king {
            Some(king) => {
                let king_position = king.position;

                let black_moves = self.inverted().get_all_moves_unchecked(game_type);

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
    pub fn get_all_legal_moves(
        &self,
        game_type: GameType,
    ) -> Result<Vec<PieceMove>, anyhow::Error> {
        let possible_moves = self.get_all_moves_unchecked(game_type);
        let mut moves = Vec::with_capacity(possible_moves.len());

        for mv in possible_moves.into_iter() {
            let mut new_position = self.clone();
            new_position.apply_move(mv)?;

            if !new_position.is_king_in_check(game_type)? {
                moves.push(mv);
            }
        }

        Ok(moves)
    }

    /// Gets all moves that are possible by white, without checking for
    /// check, use this to check whether a king is in check, etc.
    pub fn get_all_moves_unchecked(&self, game_type: GameType) -> Vec<PieceMove> {
        let mut moves = Vec::with_capacity(self.pieces.len() * 8);

        for piece in self.pieces.iter().filter(|p| p.color == Color::White) {
            let legal_moves = piece.get_legal_moves(self.white_map, self.black_map);

            // Don't move, must rescue or drop
            if game_type == GameType::Rescue {
                for dir in piece.position.get_cardinal_adjacent().into_iter() {
                    if let Some(dir) = dir {
                        match piece.holding {
                            Some(_) => {
                                if let None = self.get_piece_at(dir) {
                                    moves.push(PieceMove {
                                        from: piece.position,
                                        to: piece.position,
                                        move_type: MoveType::NormalAndDrop(dir),
                                        piece_type: piece.piece_type,
                                    });
                                }
                            }
                            None => {
                                if let Some(piece_at_pos) = self.get_piece_at(dir) {
                                    if piece_at_pos.color == Color::White
                                        && piece.piece_type.can_hold(piece_at_pos.piece_type)
                                    {
                                        moves.push(PieceMove {
                                            from: piece.position,
                                            to: piece.position,
                                            move_type: MoveType::NormalAndRescue(dir),
                                            piece_type: piece.piece_type,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Move to a spot, maybe capture, maybe rescue, maybe drop
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

                if game_type == GameType::Rescue {
                    match piece.holding {
                        // Drop a rescued piece at an adjacent position
                        Some(_) => {
                            for dir in to.get_cardinal_adjacent().into_iter() {
                                if let Some(dir) = dir {
                                    if let None = self.get_piece_at(dir) {
                                        if self.black_map.get(to) {
                                            moves.push(PieceMove {
                                                from: piece.position,
                                                to,
                                                move_type: MoveType::CaptureAndDrop {
                                                    captured_type: self
                                                        .get_piece_at(to)
                                                        .unwrap()
                                                        .piece_type,
                                                    drop_pos: dir,
                                                },
                                                piece_type: piece.piece_type,
                                            });
                                        } else {
                                            moves.push(PieceMove {
                                                from: piece.position,
                                                to,
                                                move_type: MoveType::NormalAndDrop(dir),
                                                piece_type: piece.piece_type,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        // Rescue adjacent pieces of the same color
                        None => {
                            for dir in to.get_cardinal_adjacent().into_iter() {
                                if let Some(dir) = dir {
                                    if let Some(piece_at_pos) = self.get_piece_at(dir) {
                                        if piece_at_pos == piece {
                                            continue;
                                        }

                                        if piece_at_pos.color == Color::White
                                            && piece.piece_type.can_hold(piece_at_pos.piece_type)
                                        {
                                            if self.black_map.get(to) {
                                                moves.push(PieceMove {
                                                    from: piece.position,
                                                    to,
                                                    move_type: MoveType::CaptureAndRescue {
                                                        captured_type: self
                                                            .get_piece_at(to)
                                                            .unwrap()
                                                            .piece_type,
                                                        rescued_pos: dir,
                                                    },
                                                    piece_type: piece.piece_type,
                                                });
                                            } else {
                                                moves.push(PieceMove {
                                                    from: piece.position,
                                                    to,
                                                    move_type: MoveType::NormalAndRescue(dir),
                                                    piece_type: piece.piece_type,
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
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

    pub fn to_board_string_with_rank_file(&self) -> String {
        let mut board = [[None; 8]; 8];

        for piece in self.pieces.iter() {
            let (x, y) = piece.position.as_tuple();
            board[y as usize][x as usize] = Some(piece);
        }

        let mut board_string = String::new();

        for rank in 0..8 {
            board_string.push_str(&(8 - rank).to_string());
            board_string.push(' ');

            for file in 0..8 {
                match board[rank][file] {
                    Some(piece) => {
                        board_string.push_str(&piece.to_string());
                    }
                    None => {
                        board_string.push('.');
                    }
                }
                board_string.push(' ');
            }

            board_string.push('\n');
        }

        board_string.push_str("  a b c d e f g h\n");

        board_string
    }

    pub fn apply_move(&mut self, mv: PieceMove) -> Result<(), anyhow::Error> {
        let piece = self
            .get_piece_at(mv.from)
            .ok_or(anyhow::anyhow!("No piece at position"))?;

        let legal_moves = piece.get_legal_moves(self.white_map, self.black_map);

        let is_rescue_or_drop = match mv.move_type {
            MoveType::NormalAndRescue(_) | MoveType::NormalAndDrop(_) => true,
            _ => false,
        };

        if !legal_moves.get(mv.to) && !is_rescue_or_drop {
            return Err(anyhow::anyhow!("Illegal move"));
        }

        match mv.move_type {
            MoveType::Unknown => {
                return Err(anyhow::anyhow!("Unknown move type"));
            }
            MoveType::Normal => {
                self.move_piece(mv.from, mv.to)?;
            }
            MoveType::NormalAndRescue(pos) => {
                if mv.from != mv.to {
                    self.move_piece(mv.from, mv.to)?;
                }
                self.rescue_piece(mv.to, pos)?;
            }
            MoveType::NormalAndDrop(pos) => {
                if mv.from != mv.to {
                    self.move_piece(mv.from, mv.to)?;
                }
                self.drop_piece(mv.to, pos)?;
            }
            MoveType::CaptureAndRescue {
                captured_type: _,
                rescued_pos,
            } => {
                self.remove_piece_at(mv.to)?;
                self.move_piece(mv.from, mv.to)?;
                self.rescue_piece(mv.to, rescued_pos)?;
            }
            MoveType::CaptureAndDrop {
                captured_type: _,
                drop_pos,
            } => {
                self.remove_piece_at(mv.to)?;
                self.move_piece(mv.from, mv.to)?;
                self.drop_piece(mv.to, drop_pos)?;
            }
            MoveType::Capture(_) => {
                self.remove_piece_at(mv.to)?;
                self.move_piece(mv.from, mv.to)?;
            }
            MoveType::EnPassant(pos) => {
                self.remove_piece_at(pos)?;
                self.move_piece(mv.from, mv.to)?;
            }
            MoveType::Castle { king: _, rook: _ } => {
                todo!();
            }
            MoveType::Promotion(piece_type) => {
                self.move_piece(mv.from, mv.to)?;
                self.get_piece_at_mut(mv.from).unwrap().piece_type = piece_type;
            }
            MoveType::CapturePromotion {
                captured: _,
                promoted_to: _,
            } => {
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
                self.move_piece(mv.to, mv.from)?;
            }
            MoveType::NormalAndRescue(pos) => {
                self.drop_piece(mv.to, pos)?;

                if mv.from != mv.to {
                    self.move_piece(mv.to, mv.from)?;
                }
            }
            MoveType::NormalAndDrop(pos) => {
                self.rescue_piece(piece.position, pos)?;

                if mv.from != mv.to {
                    self.move_piece(mv.to, mv.from)?;
                }
            }
            MoveType::Capture(captured) => {
                self.move_piece(mv.to, mv.from)?;
                self.add_piece(Piece::new(captured, color, mv.to))?;
            }
            MoveType::CaptureAndRescue {
                captured_type,
                rescued_pos,
            } => {
                self.drop_piece(mv.to, rescued_pos)?;
                self.move_piece(mv.to, mv.from)?;
                self.add_piece(Piece::new(captured_type, color, mv.to))?;
            }
            MoveType::CaptureAndDrop {
                captured_type,
                drop_pos,
            } => {
                self.rescue_piece(piece.position, drop_pos)?;
                self.move_piece(mv.to, mv.from)?;
                self.add_piece(Piece::new(captured_type, color, mv.to))?;
            }
            MoveType::EnPassant(pos) => {
                self.move_piece(mv.to, mv.from)?;
                self.add_piece(Piece::new(PieceType::Pawn, color, pos))?;
            }
            MoveType::Castle { king: _, rook: _ } => {
                todo!();
            }
            MoveType::Promotion(_) => {
                self.move_piece(mv.to, mv.from)?;
                self.get_piece_at_mut(mv.to).unwrap().piece_type = PieceType::Pawn;
            }
            MoveType::CapturePromotion {
                captured: _,
                promoted_to: _,
            } => {
                todo!();
            }
        }

        self.calc_changes();

        Ok(())
    }

    pub fn parse_from_fen(fen: &str) -> Result<Position, anyhow::Error> {
        return fen::parse_position_from_fen(fen);
    }

    pub fn to_fen(&self) -> String {
        return fen::position_to_fen(self);
    }

    /// Creates a new position by applying a sequence of moves to the starting position.
    /// Each move should be in algebraic notation (e.g. "e4", "Nf3", etc.).
    ///
    /// # Example
    /// ```rust
    /// use rescue_chess::Position;
    ///
    /// let position = Position::from_moves(&["e4", "e5", "Nf3"]).unwrap();
    /// ```
    pub fn from_moves(moves: &[&str], game_type: GameType) -> Result<Position, anyhow::Error> {
        let mut position = Position::start_position();
        let mut is_black = false;

        for &mv_str in moves {
            // We need to invert the algebraic notation if we are playing as black,
            // so use the from_algebraic_inverted method
            let mv = if is_black {
                PieceMove::from_algebraic_inverted(&position, mv_str, game_type)?
            } else {
                PieceMove::from_algebraic(&position, mv_str, game_type)?
            };

            position.apply_move(mv)?;

            // Switch sides by inverting the position
            position.invert();
            is_black = !is_black;
        }

        Ok(position)
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.to_fen() == other.to_fen()
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
    use crate::{
        piece_move::{GameType, MoveType},
        PieceMove, PieceType, Pos, Position,
    };

    #[test]
    pub fn parse_fen_1() {
        let position: Position = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".into();

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
        assert!(!position.is_king_in_check(GameType::Rescue).unwrap());

        let position: Position = "1N3r2/4P3/2pP3p/2P2P2/3K1k2/2p1p3/3BBq2/2R5 w - - 0 1".into();
        assert!(!position.is_king_in_check(GameType::Rescue).unwrap());

        let position: Position = "8/nR2Q2P/P2P2kb/4B2b/4K3/1r2P2P/5p2/1r6 w - - 0 1".into();
        assert!(!position.is_king_in_check(GameType::Rescue).unwrap());

        let position: Position = "4B1r1/7q/rk4pP/4n3/1Np5/1p1P1R2/P1Q2K2/8 w - - 0 1".into();
        assert!(!position.is_king_in_check(GameType::Rescue).unwrap());
    }

    #[test]
    fn king_in_check() {
        let position: Position = "rnb1kbnr/pppppppp/3q4/8/3K4/8/PPPPPPPP/RNBQ1BNR w - - 0 1".into();
        assert!(position.is_king_in_check(GameType::Rescue).unwrap());

        let position: Position = "rnbqk1nr/pppppppp/5b2/8/3K4/8/PPPPPPPP/RNBQ1BNR".into();
        assert!(position.is_king_in_check(GameType::Rescue).unwrap());

        let position: Position = "rnbqkb1r/pppppppp/4n3/8/3K4/8/PPPPPPPP/RNBQ1BNR".into();
        assert!(position.is_king_in_check(GameType::Rescue).unwrap());

        let position: Position = "rnbqkbnr/pppp1ppp/8/4p3/3K4/8/PPPPPPPP/RNBQ1BNR".into();
        assert!(position.is_king_in_check(GameType::Rescue).unwrap());
    }

    #[test]
    fn legal_moves_puts_king_in_check() {
        let position: Position = "8/8/8/3r4/3R4/3K4/8/8".into();

        println!("{}", position.to_board_string());

        let moves = position.get_all_legal_moves(GameType::Rescue).unwrap();

        for mv in moves {
            println!("{}", mv);
        }
    }

    #[test]
    fn possible_moves_includes_rescue() {
        let position: Position = "8/8/8/8/8/8/8/3QK3 w - - 0 1".into();

        let moves = position.get_all_moves_unchecked(GameType::Rescue);

        for mv in moves {
            println!("{}", mv);
        }
    }

    #[test]
    fn all_possible_moves_start_position() {
        let position: Position = Position::start_position();

        let moves = position.get_all_moves_unchecked(GameType::Rescue);

        for mv in moves {
            println!("{}", mv);
        }
    }

    #[test]
    fn rescue_bug() {
        let mut position: Position = "8/8/8/8/8/8/P7/1P6 w - - 0 1".into();
        println!("{}", position.to_board_string());

        let mv = PieceMove {
            from: "b1".into(),
            to: "b2".into(),
            move_type: MoveType::NormalAndRescue("a2".into()),
            piece_type: PieceType::Pawn,
        };

        position.apply_move(mv).unwrap();

        println!("{}", position.to_board_string());
    }

    #[test]
    fn test_from_moves_empty() {
        let position = Position::from_moves(&[], GameType::Rescue).unwrap();
        assert_eq!(position, Position::start_position());
    }

    #[test]
    fn test_from_moves_single_move() {
        let position = Position::from_moves(&["e4"], GameType::Rescue).unwrap();

        // Verify pawn moved to e4
        let expected =
            Position::parse_from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1")
                .unwrap();

        assert_eq!(position, expected);
    }

    #[test]
    fn test_from_moves_multiple_moves() {
        let position = Position::from_moves(&["e4", "e5", "Nf3"], GameType::Rescue).unwrap();

        // Verify sequence of moves
        let expected = Position::parse_from_fen(
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 1",
        )
        .unwrap();

        assert_eq!(position, expected);
    }

    #[test]
    fn test_from_moves_with_captures() {
        let position =
            Position::from_moves(&["e4", "d5", "exd5", "Qxd5"], GameType::Rescue).unwrap();

        // Verify captures were handled correctly
        let expected =
            Position::parse_from_fen("rnb1kbnr/ppp1pppp/8/3q4/8/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1")
                .unwrap();

        assert_eq!(position, expected);
    }

    #[test]
    fn test_from_moves_with_rescue() {
        let position = Position::from_moves(&["e2Sf2"], GameType::Rescue).unwrap();

        // Verify the rescue operation
        assert!(position
            .get_piece_at(Pos::from_algebraic("e2").unwrap())
            .unwrap()
            .holding
            .is_some());

        assert!(position
            .get_piece_at(Pos::from_algebraic("f2").unwrap())
            .is_none());
    }

    #[test]
    fn test_from_moves_invalid_move() {
        // Try to make an invalid move
        assert!(Position::from_moves(&["e5"], GameType::Rescue).is_err()); // Pawn can't move two squares from e2
        assert!(Position::from_moves(&["d4", "d5", "Ke2"], GameType::Rescue).is_err());
        // King can't move through pawn
    }
}
