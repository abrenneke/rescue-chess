use serde::{Deserialize, Serialize};

use crate::{PieceType, Pos};

/// A move that a chess piece can make.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq)]
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

/// The type of move a piece can make. Non-normal moves can store additional information, such as captured piece.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum MoveType {
    /// A move that has not been classified yet
    Unknown,

    /// A move that does not capture an enemy piece
    Normal,

    /// A move that maybe moves to a position, then rescues and adjacent piece at pos
    NormalAndRescue(Pos),

    /// A move that maybe moves to a position, then drops a rescued piece at pos
    NormalAndDrop(Pos),

    /// A move that captures an enemy piece
    /// The PieceType is the type of piece that is captured
    Capture(PieceType),

    /// A move that captures an enemy piece and rescues a friendly piece
    CaptureAndRescue {
        captured_type: PieceType,
        rescued_pos: Pos,
    },

    /// A move that captures an enemy piece and drops a rescued piece
    CaptureAndDrop {
        captured_type: PieceType,
        drop_pos: Pos,
    },

    /// A move that captures an enemy piece en passant.
    /// The Pos is the position of the captured pawn.
    EnPassant(Pos),

    /// A move that castles the king
    /// The first Pos is the position of the king, the second is the position of the rook
    Castle { king: Pos, rook: Pos },

    /// A move that promotes a pawn
    /// The PieceType is the type of piece the pawn is promoted to
    Promotion(PieceType),

    /// A move that promotes a pawn and captures an enemy piece
    /// The first PieceType is the type of piece the pawn is promoted to, the second is the type of piece the pawn captures
    CapturePromotion {
        captured: PieceType,
        promoted_to: PieceType,
    },
}

impl PieceMove {
    /// Inverts the move, so that it can be applied from the other player's perspective.
    pub fn invert(&mut self) {
        self.from = self.from.invert();
        self.to = self.to.invert();

        match &mut self.move_type {
            MoveType::EnPassant(pos) => {
                *pos = pos.invert();
            }
            MoveType::Castle { king, rook } => {
                *king = king.invert();
                *rook = rook.invert();
            }
            MoveType::NormalAndRescue(pos) => {
                *pos = pos.invert();
            }
            MoveType::CaptureAndRescue {
                captured_type: _,
                rescued_pos,
            } => {
                *rescued_pos = rescued_pos.invert();
            }
            MoveType::NormalAndDrop(pos) => {
                *pos = pos.invert();
            }
            MoveType::CaptureAndDrop {
                captured_type: _,
                drop_pos,
            } => {
                *drop_pos = drop_pos.invert();
            }
            _ => {}
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
            MoveType::Capture(_) => true,
            MoveType::CaptureAndRescue { .. } => true,
            MoveType::CaptureAndDrop { .. } => true,
            MoveType::EnPassant(_) => true,
            MoveType::CapturePromotion { .. } => true,
            _ => false,
        }
    }
}

/// Displays the move in algebraic notation.
impl std::fmt::Display for PieceMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

        if let MoveType::Capture(_) = &self.move_type {
            write!(f, "x")?;
        } else if let MoveType::CaptureAndDrop {
            captured_type: _,
            drop_pos: _,
        } = &self.move_type
        {
            write!(f, "x")?;
        } else if let MoveType::CaptureAndRescue {
            captured_type: _,
            rescued_pos: _,
        } = &self.move_type
        {
            write!(f, "x")?;
        }

        write!(f, "{}", self.to.to_algebraic())?;

        // s for save, d for drop
        if let MoveType::NormalAndRescue(pos) = self.move_type {
            write!(f, "S{}", pos.to_algebraic())?;
        } else if let MoveType::NormalAndDrop(pos) = self.move_type {
            write!(f, "D{}", pos.to_algebraic())?;
        } else if let MoveType::CaptureAndRescue {
            captured_type: _,
            rescued_pos,
        } = self.move_type
        {
            write!(f, "S{}", rescued_pos.to_algebraic())?;
        } else if let MoveType::CaptureAndDrop {
            captured_type: _,
            drop_pos,
        } = self.move_type
        {
            write!(f, "D{}", drop_pos.to_algebraic())?;
        }

        Ok(())
    }
}

impl std::fmt::Debug for PieceMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
