mod fen;

use std::{hash::Hash, mem};

use colored::Colorize;

use crate::{
    bitboard::Bitboard,
    piece::{Color, King, PieceType},
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

impl Default for CastlingRights {
    fn default() -> Self {
        CastlingRights {
            white_king_side: true,
            white_queen_side: true,
            black_king_side: true,
            black_queen_side: true,
        }
    }
}

/// A game position in chess. Contains all state to represent a single position
/// in a game of chess.
#[derive(Debug, Clone, Eq)]
pub struct Position {
    /// The pieces on the board
    pub white_pieces: arrayvec::ArrayVec<Piece, 16>,

    /// The pieces on the board
    pub black_pieces: arrayvec::ArrayVec<Piece, 16>,

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

    /// Optimized bitboard for quick lookups of if a position is occupied by any piece.
    pub all_map: Bitboard,

    /// Quick lookup of the index of a piece in the pieces vector
    pub position_lookup: [Option<u8>; 64],

    pub white_king: Option<Piece>,

    pub true_active_color: Color,
}

/// Given a list of pieces, returns a bitboard with the positions of the pieces for the given color.
#[inline]
fn to_bitboard(pieces: &[Piece]) -> Bitboard {
    let mut bb = Bitboard::new();
    for piece in pieces.iter() {
        bb.set(piece.position);
    }
    bb
}

/// Calculates the position_lookup by iterating over the pieces and setting the
/// index of the piece in the pieces vector at the position of the piece.
fn calc_position_lookup(white_pieces: &[Piece], black_pieces: &[Piece]) -> [Option<u8>; 64] {
    let mut position_lookup = [None; 64];

    for i in 0..white_pieces.len() {
        let piece = &white_pieces[i];
        position_lookup[piece.position.0 as usize] = Some(i as u8);
    }

    for i in 0..black_pieces.len() {
        let piece = &black_pieces[i];
        position_lookup[piece.position.0 as usize] = Some((i as u8) + 16);
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
        let white_pieces: arrayvec::ArrayVec<Piece, 16> = pieces
            .iter()
            .filter(|piece| piece.color == Color::White)
            .cloned()
            .collect();

        let black_pieces: arrayvec::ArrayVec<Piece, 16> = pieces
            .iter()
            .filter(|piece| piece.color == Color::Black)
            .cloned()
            .collect();

        let white_map = to_bitboard(&white_pieces);
        let black_map = to_bitboard(&black_pieces);
        let all_map = white_map | black_map;

        let position_lookup = calc_position_lookup(&white_pieces, &black_pieces);

        let white_king = white_pieces
            .iter()
            .find(|piece| piece.piece_type == PieceType::King && piece.color == Color::White)
            .cloned();

        Position {
            white_pieces,
            black_pieces,
            castling_rights,
            en_passant,
            halfmove_clock,
            fullmove_number,
            white_map,
            black_map,
            position_lookup,
            white_king,
            all_map,
            true_active_color: Color::White,
        }
    }

    /// Returns the start position of a chess game.
    pub fn start_position() -> Self {
        return Self::parse_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .unwrap();
    }

    pub fn empty() -> Self {
        Self::new(Vec::new(), Default::default(), None, 0, 1)
    }

    /// Inverts the position, i.e. makes the black pieces white and vice versa.
    /// The board will be flipped as well, i.e. a1 will become h8 and so on.
    pub fn invert(&mut self) {
        mem::swap(&mut self.white_pieces, &mut self.black_pieces);

        for piece in self.black_pieces.iter_mut() {
            piece.color = Color::Black;
            piece.position = piece.position.invert();
        }

        for piece in self.white_pieces.iter_mut() {
            piece.color = Color::White;
            piece.position = piece.position.invert();
        }

        self.true_active_color = self.true_active_color.invert();

        if let Some(en_passant) = self.en_passant {
            self.en_passant = Some(en_passant.invert());
        }

        self.calc_changes(true);
    }

    /// When any piece has changed, this function should be called to
    /// recalculate the bitboards and position lookup.
    pub fn calc_changes(&mut self, do_calc_position_lookup: bool) {
        if do_calc_position_lookup {
            self.white_map = to_bitboard(&self.white_pieces);
            self.black_map = to_bitboard(&self.black_pieces);
            self.all_map = self.white_map | self.black_map;

            self.position_lookup = calc_position_lookup(&self.white_pieces, &self.black_pieces);
        }

        self.white_king = self
            .white_pieces
            .iter()
            .find(|piece| piece.piece_type == PieceType::King && piece.color == Color::White)
            .cloned();
    }

    /// Returns a new GamePosition with the colors and board flipped.
    pub fn inverted(&self) -> Position {
        let mut position = self.clone();
        position.invert();
        position
    }

    /// Gets the piece at a specific position, if any.
    #[inline(always)]
    pub fn get_piece_at(&self, position: Pos) -> Option<&Piece> {
        if !self.all_map.get(position) {
            return None;
        }

        if let Some(index) = self.position_lookup[position.0 as usize] {
            if index >= 16 {
                return Some(&self.black_pieces[(index - 16) as usize]);
            } else {
                return Some(&self.white_pieces[index as usize]);
            }
        } else {
            None
        }
    }

    /// Gets the piece at a specific position, if any, mutably.
    pub fn get_piece_at_mut(&mut self, position: Pos) -> Option<&mut Piece> {
        if let Some(index) = self.position_lookup[position.0 as usize] {
            if index >= 16 {
                return Some(&mut self.black_pieces[(index - 16) as usize]);
            } else {
                return Some(&mut self.white_pieces[index as usize]);
            }
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

        if rescuer_piece.holding.is_some() {
            return Err(anyhow::anyhow!(
                "Rescuer at {} already holding a piece!",
                rescuer.to_algebraic()
            ));
        }

        if rescued_piece.holding.is_some() {
            return Err(anyhow::anyhow!(
                "Piece at {} cannot rescue, rescued piece at {} already holding a piece!",
                rescuer.to_algebraic(),
                rescued.to_algebraic()
            ));
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
            None => {
                return Err(anyhow::anyhow!(
                    "Holder at {} not holding a piece, cannot drop at {}",
                    rescuer_pos.to_algebraic(),
                    drop_pos.to_algebraic()
                ));
            }
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
            return Err(anyhow::anyhow!(
                "Position occupied {}, board state:\n{}",
                to.to_algebraic(),
                self.to_board_string_with_rank_file(false)
            ));
        }

        match self.position_lookup[from.0 as usize] {
            Some(piece_idx) => {
                let is_black = piece_idx >= 16;

                let piece = if is_black {
                    &mut self.black_pieces[(piece_idx - 16) as usize]
                } else {
                    &mut self.white_pieces[piece_idx as usize]
                };

                piece.position = to;

                self.position_lookup.swap(from.0 as usize, to.0 as usize);

                if is_black {
                    self.black_map.clear(from);
                    self.black_map.set(to);
                } else {
                    self.white_map.clear(from);
                    self.white_map.set(to);
                }
                self.all_map = self.white_map | self.black_map;
            }
            None => {
                return Err(anyhow::anyhow!(
                    "No piece at position {} to move to {}, board state:\n{}",
                    from.to_algebraic(),
                    to.to_algebraic(),
                    self.to_board_string_with_rank_file(false)
                ));
            }
        }

        self.calc_changes(false);
        Ok(())
    }

    /// Removes the piece at a specific position.
    pub fn remove_piece_at(&mut self, position: Pos) -> Result<(), anyhow::Error> {
        if let Some(index) = self.position_lookup[position.0 as usize] {
            if index >= 16 {
                self.black_pieces.remove((index - 16) as usize);
            } else {
                self.white_pieces.remove(index as usize);
            }

            self.calc_changes(true);
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

        if piece.color == Color::White {
            self.white_pieces.push(piece);
        } else {
            self.black_pieces.push(piece);
        }

        self.calc_changes(true);

        Ok(())
    }

    /// Returns true if white is in checkmate. Returns an error if the position is invalid (no king)
    pub fn is_checkmate(&self, game_type: GameType) -> Result<bool, anyhow::Error> {
        Ok(self.is_king_in_check()? && self.get_all_legal_moves(game_type)?.is_empty())
    }

    /// Returns true if the white king is currently in check. Returns an error if there is no king.
    pub fn is_king_in_check(&self) -> Result<bool, anyhow::Error> {
        Ok(King::is_in_check(self))
    }

    pub fn is_piece_at(&self, position: Pos, piece_type: &[PieceType], color: Color) -> bool {
        if !self.all_map.get(position) {
            return false;
        }

        match self.get_piece_at(position) {
            Some(piece) => piece.color == color && piece_type.contains(&piece.piece_type),
            None => false,
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

        let mut position = self.clone();

        for mv in possible_moves.into_iter() {
            // let orig = position.clone();

            let prev_en_passant = position.en_passant;
            position.apply_move(mv)?;

            if !position.is_king_in_check()? {
                moves.push(mv);
            }

            position.unapply_move(mv)?;
            position.en_passant = prev_en_passant;

            // if orig.to_board_string_with_rank_file_holding()
            //     != position.to_board_string_with_rank_file_holding()
            // {
            //     println!("Unapplied move: {}", mv);
            //     assert_eq!(
            //         orig.to_board_string_with_rank_file_holding(),
            //         position.to_board_string_with_rank_file_holding()
            //     );
            // }
        }

        Ok(moves)
    }

    /// Gets all moves that are possible by white, without checking for
    /// check, use this to check whether a king is in check, etc.
    pub fn get_all_moves_unchecked(&self, game_type: GameType) -> Vec<PieceMove> {
        let mut moves = Vec::with_capacity(self.white_pieces.len() * 8);

        for piece in self.white_pieces.iter() {
            let legal_moves = piece.get_legal_moves(self);

            // Don't move, must rescue or drop
            if game_type == GameType::Rescue {
                for dir in piece.position.get_cardinal_adjacent().into_iter() {
                    if let Some(dir) = dir {
                        match piece.holding {
                            Some(holding) => {
                                if let None = self.get_piece_at(dir) {
                                    if dir.is_row(0) && holding == PieceType::Pawn {
                                        // Drop pawn on promotion row
                                        for promoted_to in [
                                            PieceType::Queen,
                                            PieceType::Rook,
                                            PieceType::Bishop,
                                            PieceType::Knight,
                                        ] {
                                            moves.push(PieceMove {
                                                from: piece.position,
                                                to: piece.position,
                                                move_type: MoveType::NormalAndDrop {
                                                    pos: dir,
                                                    promoted_to: Some(promoted_to),
                                                },
                                                piece_type: piece.piece_type,
                                            });
                                        }
                                    } else {
                                        // Drop anything else anywhere
                                        moves.push(PieceMove {
                                            from: piece.position,
                                            to: piece.position,
                                            move_type: MoveType::NormalAndDrop {
                                                pos: dir,
                                                promoted_to: None,
                                            },
                                            piece_type: piece.piece_type,
                                        });
                                    }
                                }
                            }
                            None => {
                                if let Some(piece_at_pos) = self.get_piece_at(dir) {
                                    if piece_at_pos.color == Color::White
                                        && piece.piece_type.can_hold(piece_at_pos.piece_type)
                                        && piece_at_pos.holding.is_none()
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
                    if to.is_row(0) && piece.piece_type == PieceType::Pawn {
                        // Capture and promote
                        for promoted_to in [
                            PieceType::Queen,
                            PieceType::Rook,
                            PieceType::Bishop,
                            PieceType::Knight,
                        ] {
                            moves.push(PieceMove {
                                from: piece.position,
                                to,
                                move_type: MoveType::CapturePromotion {
                                    captured: self.get_piece_at(to).unwrap().piece_type,
                                    promoted_to,
                                    captured_holding: self.get_piece_at(to).unwrap().holding,
                                },
                                piece_type: piece.piece_type,
                            });
                        }
                    } else {
                        moves.push(PieceMove {
                            from: piece.position,
                            to,
                            move_type: MoveType::Capture {
                                captured: self.get_piece_at(to).unwrap().piece_type,
                                captured_holding: self.get_piece_at(to).unwrap().holding,
                            },
                            piece_type: piece.piece_type,
                        });
                    }
                } else {
                    if piece.piece_type == PieceType::King
                        && piece.position == Pos::xy(4, 7)
                        && to == Pos::xy(6, 7)
                    {
                        // White kingside castle
                        moves.push(PieceMove {
                            from: piece.position,
                            to,
                            move_type: MoveType::Castle {
                                king: Pos::xy(4, 7),
                                rook: Pos::xy(7, 7),
                            },
                            piece_type: piece.piece_type,
                        });
                    } else if piece.piece_type == PieceType::King
                        && piece.position == Pos::xy(4, 7)
                        && to == Pos::xy(2, 7)
                    {
                        // White queenside castle
                        moves.push(PieceMove {
                            from: piece.position,
                            to,
                            move_type: MoveType::Castle {
                                king: Pos::xy(4, 7),
                                rook: Pos::xy(0, 7),
                            },
                            piece_type: piece.piece_type,
                        });
                    } else if piece.piece_type == PieceType::King
                        && piece.position == Pos::xy(3, 7)
                        && to == Pos::xy(1, 7)
                    {
                        // Black queenside castle
                        moves.push(PieceMove {
                            from: piece.position,
                            to,
                            move_type: MoveType::Castle {
                                king: Pos::xy(3, 7),
                                rook: Pos::xy(0, 7),
                            },
                            piece_type: piece.piece_type,
                        });
                    } else if piece.piece_type == PieceType::King
                        && piece.position == Pos::xy(3, 7)
                        && to == Pos::xy(5, 7)
                    {
                        // Black kingside castle
                        moves.push(PieceMove {
                            from: piece.position,
                            to,
                            move_type: MoveType::Castle {
                                king: Pos::xy(3, 7),
                                rook: Pos::xy(7, 7),
                            },
                            piece_type: piece.piece_type,
                        });
                    } else if piece.piece_type == PieceType::Pawn && to.is_row(0) {
                        for promoted_to in [
                            PieceType::Queen,
                            PieceType::Rook,
                            PieceType::Bishop,
                            PieceType::Knight,
                        ]
                        .iter()
                        {
                            moves.push(PieceMove {
                                from: piece.position,
                                to,
                                move_type: MoveType::Promotion(*promoted_to),
                                piece_type: piece.piece_type,
                            });
                        }
                    } else {
                        if piece.piece_type == PieceType::Pawn {
                            if let Some(en_passant) = self.en_passant {
                                if to == en_passant {
                                    moves.push(PieceMove {
                                        from: piece.position,
                                        to,
                                        move_type: MoveType::EnPassant {
                                            captured: PieceType::Pawn,
                                            captured_pos: Pos::xy(en_passant.get_col(), 3),
                                            captured_holding: self
                                                .get_piece_at(Pos::xy(en_passant.get_col(), 3))
                                                .unwrap()
                                                .holding,
                                        },
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
                            } else {
                                moves.push(PieceMove {
                                    from: piece.position,
                                    to,
                                    move_type: MoveType::Normal,
                                    piece_type: piece.piece_type,
                                });
                            }
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

                if game_type == GameType::Rescue {
                    match piece.holding {
                        // Drop a rescued piece at an adjacent position
                        Some(holding) => {
                            for dir in to.get_cardinal_adjacent().into_iter() {
                                if let Some(dir) = dir {
                                    if let None = self.get_piece_at(dir) {
                                        if self.black_map.get(to) {
                                            if holding == PieceType::Pawn && dir.is_row(0) {
                                                // Capture and drop pawn on promotion row
                                                for promoted_to in [
                                                    PieceType::Queen,
                                                    PieceType::Rook,
                                                    PieceType::Bishop,
                                                    PieceType::Knight,
                                                ] {
                                                    moves.push(PieceMove {
                                                        from: piece.position,
                                                        to,
                                                        move_type: MoveType::CaptureAndDrop {
                                                            captured_type: self
                                                                .get_piece_at(to)
                                                                .unwrap()
                                                                .piece_type,
                                                            drop_pos: dir,
                                                            promoted_to: Some(promoted_to),
                                                            captured_holding: self
                                                                .get_piece_at(to)
                                                                .unwrap()
                                                                .holding,
                                                        },
                                                        piece_type: piece.piece_type,
                                                    });
                                                }
                                            } else {
                                                moves.push(PieceMove {
                                                    from: piece.position,
                                                    to,
                                                    move_type: MoveType::CaptureAndDrop {
                                                        captured_type: self
                                                            .get_piece_at(to)
                                                            .unwrap()
                                                            .piece_type,
                                                        drop_pos: dir,
                                                        promoted_to: None,
                                                        captured_holding: self
                                                            .get_piece_at(to)
                                                            .unwrap()
                                                            .holding,
                                                    },
                                                    piece_type: piece.piece_type,
                                                });
                                            }
                                        } else {
                                            if holding == PieceType::Pawn && dir.is_row(0) {
                                                // Drop pawn on promotion row
                                                for promoted_to in [
                                                    PieceType::Queen,
                                                    PieceType::Rook,
                                                    PieceType::Bishop,
                                                    PieceType::Knight,
                                                ] {
                                                    moves.push(PieceMove {
                                                        from: piece.position,
                                                        to,
                                                        move_type: MoveType::NormalAndDrop {
                                                            promoted_to: Some(promoted_to),
                                                            pos: dir,
                                                        },
                                                        piece_type: piece.piece_type,
                                                    });
                                                }
                                            } else {
                                                moves.push(PieceMove {
                                                    from: piece.position,
                                                    to,
                                                    move_type: MoveType::NormalAndDrop {
                                                        pos: dir,
                                                        promoted_to: None,
                                                    },
                                                    piece_type: piece.piece_type,
                                                });
                                            }
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
                                        if piece_at_pos == piece || piece_at_pos.holding.is_some() {
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
                                                        captured_holding: self
                                                            .get_piece_at(to)
                                                            .unwrap()
                                                            .holding,
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

        for piece in self.white_pieces.iter().chain(self.black_pieces.iter()) {
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

    pub fn to_board_string_with_rank_file(&self, unicode: bool) -> String {
        let mut board = [[None; 8]; 8];

        for piece in self.white_pieces.iter().chain(self.black_pieces.iter()) {
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
                        let piece_str = if unicode {
                            piece.to_colored_unicode()
                        } else {
                            piece.to_string().white()
                        };
                        board_string.push_str(&piece_str);
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

    pub fn to_board_string_with_rank_file_holding(&self) -> String {
        let mut board = [[None; 8]; 8];

        for piece in self.white_pieces.iter().chain(self.black_pieces.iter()) {
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
                        let piece_str = piece.piece_type.to_algebraic(piece.color);
                        board_string.push_str(&piece_str);
                        if let Some(holding) = piece.holding {
                            board_string.push_str(&holding.to_algebraic(piece.color));
                        } else {
                            board_string.push(' ');
                        }
                    }
                    None => {
                        board_string.push_str(". ");
                    }
                }
                board_string.push(' ');
            }

            board_string.push('\n');
        }

        board_string.push_str("  a  b  c  d  e  f  g  h\n");

        board_string
    }

    pub fn apply_move(&mut self, mv: PieceMove) -> Result<(), anyhow::Error> {
        let piece = self.get_piece_at(mv.from).ok_or_else(|| {
            anyhow::anyhow!(
                "No piece at position {}, board state:\n{}",
                mv.from.to_algebraic(),
                self.to_board_string_with_rank_file(false)
            )
        })?;

        let legal_moves = piece.get_legal_moves(self);

        let is_rescue_or_drop = match mv.move_type {
            MoveType::NormalAndRescue(_)
            | MoveType::NormalAndDrop { .. }
            | MoveType::CaptureAndDrop { .. }
            | MoveType::CaptureAndRescue { .. } => true,
            _ => false,
        };

        if !legal_moves.get(mv.to) && !is_rescue_or_drop {
            return Err(anyhow::anyhow!(
                "Illegal move {}! Board state:\n{}\n{}, legal moves: {}",
                mv.to_string(),
                self.to_fen(),
                self.to_board_string_with_rank_file_holding(),
                legal_moves
                    .iter()
                    .map(|pos| pos.to_algebraic())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
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
            MoveType::NormalAndDrop { pos, promoted_to } => {
                if mv.from != mv.to {
                    self.move_piece(mv.from, mv.to)?;
                }
                self.drop_piece(mv.to, pos)?;

                if let Some(promoted_to) = promoted_to {
                    self.get_piece_at_mut(pos).unwrap().piece_type = promoted_to;
                }
            }
            MoveType::CaptureAndRescue {
                captured_type: _,
                rescued_pos,
                captured_holding: _,
            } => {
                self.remove_piece_at(mv.to)?;
                self.move_piece(mv.from, mv.to)?;
                self.rescue_piece(mv.to, rescued_pos)?;
            }
            MoveType::CaptureAndDrop {
                captured_type: _,
                drop_pos,
                promoted_to,
                captured_holding: _,
            } => {
                self.remove_piece_at(mv.to)?;
                self.move_piece(mv.from, mv.to)?;
                self.drop_piece(mv.to, drop_pos)?;

                if let Some(promoted_to) = promoted_to {
                    self.get_piece_at_mut(drop_pos).unwrap().piece_type = promoted_to;
                }
            }
            MoveType::Capture {
                captured: _,
                captured_holding: _,
            } => {
                self.remove_piece_at(mv.to)?;
                self.move_piece(mv.from, mv.to)?;
            }
            MoveType::EnPassant {
                captured_pos,
                captured: _,
                captured_holding: _,
            } => {
                self.remove_piece_at(captured_pos)?;
                self.move_piece(mv.from, mv.to)?;
            }
            MoveType::Castle { king: _, rook: _ } => {
                self.move_piece(mv.from, mv.to)?;

                if mv.from == Pos::xy(4, 7) {
                    // White castle
                    if mv.to == Pos::xy(6, 7) {
                        self.move_piece(Pos::xy(7, 7), Pos::xy(5, 7))?;
                    } else {
                        self.move_piece(Pos::xy(0, 7), Pos::xy(3, 7))?;
                    }
                } else if mv.from == Pos::xy(3, 7) {
                    // Black castle
                    if mv.to == Pos::xy(1, 7) {
                        self.move_piece(Pos::xy(0, 7), Pos::xy(2, 7))?;
                    } else {
                        self.move_piece(Pos::xy(7, 7), Pos::xy(4, 7))?;
                    }
                } else {
                    panic!("Illegal castle move");
                }
            }
            MoveType::Promotion(piece_type) => {
                self.move_piece(mv.from, mv.to)?;
                self.get_piece_at_mut(mv.to).unwrap().piece_type = piece_type;
            }
            MoveType::CapturePromotion {
                captured: _,
                promoted_to,
                captured_holding: _,
            } => {
                self.remove_piece_at(mv.to)?;
                self.move_piece(mv.from, mv.to)?;
                self.get_piece_at_mut(mv.to).unwrap().piece_type = promoted_to;
            }
        }

        self.try_remove_castling_rights(mv);
        self.try_en_passant_set(mv);

        Ok(())
    }

    fn try_remove_castling_rights(&mut self, mv: PieceMove) {
        // If a rook moved from a corner, remove, if a king moved from its start position, remove both
        if mv.piece_type == PieceType::Rook && mv.from == Pos::from_algebraic("a1").unwrap() {
            self.castling_rights.white_queen_side = false;
        } else if mv.piece_type == PieceType::Rook && mv.from == Pos::from_algebraic("h1").unwrap()
        {
            self.castling_rights.white_king_side = false;
        } else if mv.piece_type == PieceType::King && mv.from == Pos::from_algebraic("e1").unwrap()
        {
            self.castling_rights.white_king_side = false;
            self.castling_rights.white_queen_side = false;
        } else if mv.piece_type == PieceType::Rook && mv.from == Pos::from_algebraic("a8").unwrap()
        {
            self.castling_rights.black_queen_side = false;
        } else if mv.piece_type == PieceType::Rook && mv.from == Pos::from_algebraic("h8").unwrap()
        {
            self.castling_rights.black_king_side = false;
        } else if mv.piece_type == PieceType::King && mv.from == Pos::from_algebraic("e8").unwrap()
        {
            self.castling_rights.black_king_side = false;
            self.castling_rights.black_queen_side = false;
        }
    }

    pub fn unapply_move(&mut self, mv: PieceMove) -> Result<(), anyhow::Error> {
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
            MoveType::NormalAndDrop { pos, promoted_to } => {
                if promoted_to.is_some() {
                    self.unpromote_piece(pos)?;
                }

                let piece = self.get_piece_at(mv.to).ok_or_else(|| {
                    anyhow::anyhow!(
                        "Cannot unapply move from {} to {} and drop at {}. No piece at position {}",
                        mv.from.to_algebraic(),
                        mv.to.to_algebraic(),
                        pos.to_algebraic(),
                        mv.to.to_algebraic()
                    )
                })?;

                self.rescue_piece(piece.position, pos)?;

                if mv.from != mv.to {
                    self.move_piece(mv.to, mv.from)?;
                }
            }
            MoveType::Capture {
                captured,
                captured_holding,
            } => {
                let color = self
                    .get_piece_at(mv.to)
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "Cannot unapply move from {} to {}. No piece at position {} to capture.",
                            mv.from.to_algebraic(),
                            mv.to.to_algebraic(),
                            mv.to.to_algebraic()
                        )
                    })?
                    .color;

                self.move_piece(mv.to, mv.from)?;
                self.add_piece(Piece::new(captured, color.invert(), mv.to))?;

                if let Some(captured_holding) = captured_holding {
                    self.get_piece_at_mut(mv.to).unwrap().holding = Some(captured_holding);
                }
            }
            MoveType::CaptureAndRescue {
                captured_type,
                rescued_pos,
                captured_holding,
            } => {
                let color = self
                    .get_piece_at(mv.to)
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "Cannot unapply move from {} to {}. No piece at position {}",
                            mv.from.to_algebraic(),
                            mv.to.to_algebraic(),
                            mv.to.to_algebraic()
                        )
                    })?
                    .color;

                self.drop_piece(mv.to, rescued_pos)?;
                self.move_piece(mv.to, mv.from)?;

                self.add_piece(Piece::new(captured_type, color.invert(), mv.to))?;

                if let Some(captured_holding) = captured_holding {
                    self.get_piece_at_mut(mv.to).unwrap().holding = Some(captured_holding);
                }
            }
            MoveType::CaptureAndDrop {
                captured_type,
                drop_pos,
                promoted_to,
                captured_holding,
            } => {
                if promoted_to.is_some() {
                    self.unpromote_piece(drop_pos)?;
                }

                let piece = self.get_piece_at(mv.to).ok_or_else(|| {
                    anyhow::anyhow!(
                        "Cannot unapply move from {} to {}. No piece at position {}.",
                        mv.from.to_algebraic(),
                        mv.to.to_algebraic(),
                        mv.to.to_algebraic(),
                    )
                })?;
                let position = piece.position;
                let color = piece.color;

                self.rescue_piece(position, drop_pos)?;
                self.move_piece(mv.to, mv.from)?;

                self.add_piece(Piece::new(captured_type, color.invert(), mv.to))?;

                if let Some(captured_holding) = captured_holding {
                    self.get_piece_at_mut(mv.to).unwrap().holding = Some(captured_holding);
                }
            }
            MoveType::EnPassant {
                captured,
                captured_pos,
                captured_holding,
            } => {
                let color = self
                    .get_piece_at(mv.to)
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "Cannot unapply move from {} to {}. No piece at position {}.",
                            mv.from.to_algebraic(),
                            mv.to.to_algebraic(),
                            mv.to.to_algebraic()
                        )
                    })?
                    .color;

                self.move_piece(mv.to, mv.from)?;
                self.add_piece(Piece::new(captured, color.invert(), captured_pos))?;

                if let Some(captured_holding) = captured_holding {
                    self.get_piece_at_mut(mv.to).unwrap().holding = Some(captured_holding);
                }
            }
            MoveType::Castle { king: _, rook: _ } => {
                self.move_piece(mv.to, mv.from)?;
                if mv.from == Pos::xy(4, 7) {
                    if mv.to == Pos::xy(6, 7) {
                        self.move_piece(Pos::xy(5, 7), Pos::xy(7, 7))?;
                    } else {
                        self.move_piece(Pos::xy(3, 7), Pos::xy(0, 7))?;
                    }
                } else if mv.from == Pos::xy(3, 7) {
                    if mv.to == Pos::xy(1, 7) {
                        self.move_piece(Pos::xy(2, 7), Pos::xy(0, 7))?;
                    } else {
                        self.move_piece(Pos::xy(4, 7), Pos::xy(7, 7))?;
                    }
                } else {
                    panic!("Illegal castle move");
                }
            }
            MoveType::Promotion(_) => {
                self.unpromote_piece(mv.to)?;
                self.move_piece(mv.to, mv.from)?;
            }
            MoveType::CapturePromotion {
                captured,
                promoted_to: _,
                captured_holding,
            } => {
                let color = self
                    .get_piece_at(mv.to)
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "Cannot unapply move from {} to {}. No piece at position {}",
                            mv.from.to_algebraic(),
                            mv.to.to_algebraic(),
                            mv.to.to_algebraic(),
                        )
                    })?
                    .color;

                self.unpromote_piece(mv.to)?;
                self.move_piece(mv.to, mv.from)?;

                self.add_piece(Piece::new(captured, color.invert(), mv.to))?;

                if let Some(captured_holding) = captured_holding {
                    self.get_piece_at_mut(mv.to).unwrap().holding = Some(captured_holding);
                }
            }
        }

        self.try_readd_castling_rights(mv);

        Ok(())
    }

    fn try_en_passant_set(&mut self, mv: PieceMove) {
        let piece = self.get_piece_at(mv.to).unwrap();

        if piece.piece_type == PieceType::Pawn && mv.from.get_row() == 6 && mv.to.get_row() == 4 {
            self.en_passant = Some(Pos::xy(mv.from.get_col(), 5));
        } else {
            self.en_passant = None;
        }
    }

    fn try_readd_castling_rights(&mut self, mv: PieceMove) {
        // If a rook moved from a corner, remove, if a king moved from its start position, remove both
        if mv.piece_type == PieceType::Rook && mv.from == Pos::from_algebraic("a1").unwrap() {
            self.castling_rights.white_queen_side = true;
        } else if mv.piece_type == PieceType::Rook && mv.from == Pos::from_algebraic("h1").unwrap()
        {
            self.castling_rights.white_king_side = true;
        } else if mv.piece_type == PieceType::King && mv.from == Pos::from_algebraic("e1").unwrap()
        {
            self.castling_rights.white_king_side = true;
            self.castling_rights.white_queen_side = true;
        } else if mv.piece_type == PieceType::Rook && mv.from == Pos::from_algebraic("a8").unwrap()
        {
            self.castling_rights.black_queen_side = true;
        } else if mv.piece_type == PieceType::Rook && mv.from == Pos::from_algebraic("h8").unwrap()
        {
            self.castling_rights.black_king_side = true;
        } else if mv.piece_type == PieceType::King && mv.from == Pos::from_algebraic("e8").unwrap()
        {
            self.castling_rights.black_king_side = true;
            self.castling_rights.black_queen_side = true;
        }
    }

    pub fn promote_piece(&mut self, pos: Pos, promoted_to: PieceType) -> Result<(), anyhow::Error> {
        let piece = self
            .get_piece_at_mut(pos)
            .ok_or_else(|| anyhow::anyhow!("No piece at pos"))?;

        piece.piece_type = promoted_to;

        Ok(())
    }

    pub fn unpromote_piece(&mut self, pos: Pos) -> Result<(), anyhow::Error> {
        let piece = self
            .get_piece_at_mut(pos)
            .ok_or_else(|| anyhow::anyhow!("No piece at pos"))?;

        piece.piece_type = PieceType::Pawn;

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

        for piece in self.white_pieces.iter() {
            board[piece.position.0 as usize] = Some((piece.piece_type, piece.color));
        }

        for piece in self.black_pieces.iter() {
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

impl Default for Position {
    fn default() -> Self {
        Position::start_position()
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
                .unwrap()
                .inverted();

        assert_eq!(position, expected);
    }

    #[test]
    fn test_from_moves_multiple_moves() {
        let position = Position::from_moves(&["e4", "e5", "Nf3"], GameType::Rescue).unwrap();

        // Verify sequence of moves
        let expected = Position::parse_from_fen(
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 1",
        )
        .unwrap()
        .inverted();

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
            .get_piece_at(Pos::from_algebraic("e2").unwrap().invert())
            .unwrap()
            .holding
            .is_some());

        assert!(position
            .get_piece_at(Pos::from_algebraic("f2").unwrap().invert())
            .is_none());
    }

    #[test]
    fn test_from_moves_invalid_move() {
        // Try to make an invalid move
        assert!(Position::from_moves(&["e5"], GameType::Rescue).is_err()); // Pawn can't move two squares from e2
        assert!(Position::from_moves(&["d4", "d5", "Ke2"], GameType::Rescue).is_err());
        // King can't move through pawn
    }

    #[test]
    fn test_unapply_normal_move() {
        // Test simple pawn move
        let mut position = Position::start_position();
        let original = position.clone();

        let mv = PieceMove::from_algebraic(&position, "e4", GameType::Rescue).unwrap();
        position.apply_move(mv.clone()).unwrap();
        position.unapply_move(mv).unwrap();

        assert_eq!(
            position, original,
            "Position should be identical after applying and unapplying move"
        );
    }

    #[test]
    fn test_unapply_capture() {
        // Set up a position where white can capture black's pawn
        let mut position: Position =
            "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1".into();
        let original = position.clone();

        let mv = PieceMove::from_algebraic(&position, "exd5", GameType::Rescue).unwrap();
        position.apply_move(mv.clone()).unwrap();
        position.unapply_move(mv).unwrap();

        assert_eq!(
            position, original,
            "Position should be identical after applying and unapplying capture"
        );
    }

    #[test]
    fn test_unapply_rescue() {
        // Test rescuing a piece
        let mut position: Position = "8/8/8/8/8/8/PP6/8 w - - 0 1".into();
        let original = position.clone();

        let mv = PieceMove {
            from: "a2".into(),
            to: "a2".into(),
            move_type: MoveType::NormalAndRescue("b2".into()),
            piece_type: PieceType::Pawn,
        };

        position.apply_move(mv.clone()).unwrap();
        position.unapply_move(mv).unwrap();

        assert_eq!(
            position, original,
            "Position should be identical after applying and unapplying rescue"
        );
    }

    #[test]
    fn test_unapply_drop() {
        // Test dropping a rescued piece
        let mut position: Position = "8/8/8/8/8/8/PP6/8 w - - 0 1".into();
        let original = position.clone();

        // First rescue a piece
        let rescue_mv = PieceMove {
            from: "a2".into(),
            to: "a2".into(),
            move_type: MoveType::NormalAndRescue("b2".into()),
            piece_type: PieceType::Pawn,
        };
        position.apply_move(rescue_mv).unwrap();

        // Then drop it
        let drop_mv = PieceMove {
            from: "a2".into(),
            to: "a2".into(),
            move_type: MoveType::NormalAndDrop {
                pos: "a3".into(),
                promoted_to: None,
            },
            piece_type: PieceType::Pawn,
        };

        position.apply_move(drop_mv.clone()).unwrap();
        position.unapply_move(drop_mv).unwrap();

        // Unapply the rescue move to get back to original position
        position.unapply_move(rescue_mv).unwrap();

        assert_eq!(
            position, original,
            "Position should be identical after sequence of moves"
        );
    }

    #[test]
    fn test_unapply_move_and_rescue() {
        // Test moving to a square and rescuing an adjacent piece
        let mut position: Position = "8/8/8/8/8/8/PP6/8 w - - 0 1".into();
        let original = position.clone();

        let mv = PieceMove {
            from: "a2".into(),
            to: "a3".into(),
            move_type: MoveType::NormalAndRescue("b2".into()),
            piece_type: PieceType::Pawn,
        };

        position.apply_move(mv.clone()).unwrap();
        position.unapply_move(mv).unwrap();

        assert_eq!(
            position, original,
            "Position should be identical after applying and unapplying move with rescue"
        );
    }

    #[test]
    fn test_unapply_capture_and_rescue() {
        // Test capturing a piece and rescuing an adjacent piece
        let mut position: Position = "8/8/8/8/8/1p6/PP6/8 w - - 0 1".into();
        let original = position.clone();

        let mv = PieceMove {
            from: "a2".into(),
            to: "b3".into(),
            move_type: MoveType::CaptureAndRescue {
                captured_type: PieceType::Pawn,
                rescued_pos: "b2".into(),
                captured_holding: None,
            },
            piece_type: PieceType::Pawn,
        };

        position.apply_move(mv.clone()).unwrap();
        position.unapply_move(mv).unwrap();

        assert_eq!(
            position, original,
            "Position should be identical after applying and unapplying capture with rescue"
        );
    }

    #[test]
    fn test_unapply_move_sequence() {
        // Test a sequence of moves
        let mut position = Position::start_position();
        let original = position.clone();

        let moves = [
            PieceMove::from_algebraic(&position, "e4", GameType::Rescue).unwrap(),
            PieceMove::from_algebraic(&position.inverted(), "e4", GameType::Rescue).unwrap(),
            PieceMove::from_algebraic(&position, "Nf3", GameType::Rescue).unwrap(),
        ];

        // Apply moves
        for mv in moves.iter() {
            position.apply_move(mv.clone()).unwrap();
            position.invert();
        }

        // Unapply moves in reverse
        for mv in moves.iter().rev() {
            position.invert();
            position.unapply_move(mv.clone()).unwrap();
        }

        assert_eq!(
            position, original,
            "Position should be identical after sequence of moves"
        );
    }

    #[test]
    fn test_unapply_promotion() {
        // Test pawn promotion
        let mut position: Position = "8/P7/8/8/8/8/8/8 w - - 0 1".into();
        let original = position.clone();

        let mv = PieceMove {
            from: "a7".into(),
            to: "a8".into(),
            move_type: MoveType::Promotion(PieceType::Queen),
            piece_type: PieceType::Pawn,
        };

        position.apply_move(mv.clone()).unwrap();
        position.unapply_move(mv).unwrap();

        assert_eq!(
            position, original,
            "Position should be identical after applying and unapplying promotion"
        );
    }

    #[test]
    fn test_unapply_errors() {
        let mut position = Position::start_position();

        // Test unapplying a move with no piece at destination
        let mv = PieceMove {
            from: "e2".into(),
            to: "e4".into(),
            move_type: MoveType::Normal,
            piece_type: PieceType::Pawn,
        };

        assert!(
            position.unapply_move(mv.clone()).is_err(),
            "Should error when no piece at destination"
        );
    }

    #[test]
    fn cannot_do_kg1() {
        let position = Position::parse_from_fen(
            "r2qkb1r/ppp2ppp/2n2n2/1B1p1b2/1P1P4/2P2N2/P4PPP/RNBQK2R b KQkq - 2 7",
        )
        .unwrap();

        println!("{}", position.to_board_string_with_rank_file(false));

        let moves = position.get_all_legal_moves(GameType::Classic).unwrap();

        for mv in moves.iter() {
            println!("{}", mv);
        }

        assert!(
            moves.iter().any(|mv| mv.piece_type == PieceType::King
                && mv.to == Pos::from_algebraic("g1").unwrap())
                == false
        );
    }

    #[test]
    fn black_castle() {
        let mut position =
            Position::parse_from_fen("4k2r/6pp/2NPpn2/5p2/3P4/8/PP1B1PPP/R3K2R b Qkq - 0 1")
                .unwrap();

        println!("{}", position.to_board_string_with_rank_file(false));

        let mv = PieceMove::from_uci_inverted(&position, "e8g8", GameType::Classic).unwrap();

        println!("{}", mv);

        position.apply_move(mv).unwrap();

        println!("{}", position.to_board_string_with_rank_file(false));
    }

    #[test]
    fn en_passant_capture() {
        let position = Position::parse_from_fen("8/8/8/3pP3/8/8/8/8 w - d6 0 1").unwrap();

        let moves = position.get_all_legal_moves(GameType::Classic).unwrap();

        for mv in moves.iter() {
            println!("{}", mv);
        }

        assert!(moves.iter().any(|mv| mv.move_type
            == MoveType::EnPassant {
                captured_pos: Pos::from_algebraic("d5").unwrap(),
                captured: PieceType::Pawn,
                captured_holding: None,
            }));
    }
}
