use crate::{Piece, PieceType, Pos, Position};

#[derive(Debug, Clone, Copy)]
pub struct PieceMove {
    /// The type of piece that is moving
    pub piece_type: PieceType,

    /// The position the piece is moving from
    pub from: Pos,

    /// The position the piece is moving to
    pub to: Pos,

    /// Additional information about the move
    pub move_type: MoveType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MoveType {
    /// A move that has not been classified yet
    Unknown,

    /// A move that does not capture an enemy piece
    Normal,

    /// A move that captures an enemy piece
    /// The PieceType is the type of piece that is captured
    Capture(PieceType),

    /// A move that captures an enemy piece en passant.
    /// The Pos is the position of the captured pawn.
    EnPassant(Pos),

    /// A move that castles the king
    /// The first Pos is the position of the king, the second is the position of the rook
    Castle(Pos, Pos),

    /// A move that promotes a pawn
    /// The PieceType is the type of piece the pawn is promoted to
    Promotion(PieceType),

    /// A move that promotes a pawn and captures an enemy piece
    /// The first PieceType is the type of piece the pawn is promoted to, the second is the type of piece the pawn captures
    CapturePromotion(PieceType, PieceType),
}

impl Position {
    pub fn apply_move(&mut self, mv: PieceMove) -> Result<(), anyhow::Error> {
        let piece = self
            .get_piece_at(mv.from)
            .ok_or(anyhow::anyhow!("No piece at position"))?;

        let legal_moves = piece.get_legal_moves(self.white_map, self.black_map);

        if !legal_moves.get(mv.to) {
            return Err(anyhow::anyhow!("Illegal move"));
        }

        let piece = self.get_piece_at_mut(mv.from).unwrap();

        match mv.move_type {
            MoveType::Unknown => {
                return Err(anyhow::anyhow!("Unknown move type"));
            }
            MoveType::Normal => {
                piece.position = mv.to;
            }
            MoveType::Capture(_) => {
                piece.position = mv.to;
                self.remove_piece_at(mv.to);
            }
            MoveType::EnPassant(pos) => {
                piece.position = mv.to;
                self.remove_piece_at(pos);
            }
            MoveType::Castle(_king_pos, _rook_pos) => {
                todo!();
            }
            MoveType::Promotion(piece_type) => {
                piece.position = mv.to;
                piece.piece_type = piece_type;
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

        Ok(())
    }
}

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
        }

        write!(f, "{}", self.to.to_algebraic())
    }
}
