use std::sync::LazyLock;

use crate::{
    bitboard::Bitboard, evaluation::square_bonus::SquareBonus, piece_move::CanMove, pos, Pos,
    Position,
};

use super::{pawn, ChessPiece, Color, Piece, PieceType, RescueChessPiece};

pub struct King;

impl RescueChessPiece for King {
    fn can_hold(_other: super::PieceType) -> bool {
        true
    }
}

static ATTACK_MAPS: LazyLock<[Bitboard; 64]> = LazyLock::new(|| {
    let mut maps = [Bitboard::new(); 64];

    for i in 0..64 {
        let mut board = Bitboard::new();
        let pos = Pos(i as u8);

        if pos.can_move_left() && pos.can_move_up() {
            board.set(pos.moved_unchecked(-1, -1));
        }

        if pos.can_move_up() {
            board.set(pos.moved_unchecked(0, -1));
        }

        if pos.can_move_right() && pos.can_move_up() {
            board.set(pos.moved_unchecked(1, -1));
        }

        if pos.can_move_right() {
            board.set(pos.moved_unchecked(1, 0));
        }

        if pos.can_move_right() && pos.can_move_down() {
            board.set(pos.moved_unchecked(1, 1));
        }

        if pos.can_move_down() {
            board.set(pos.moved_unchecked(0, 1));
        }

        if pos.can_move_left() && pos.can_move_down() {
            board.set(pos.moved_unchecked(-1, 1));
        }

        if pos.can_move_left() {
            board.set(pos.moved_unchecked(-1, 0));
        }

        maps[i as usize] = board;
    }

    maps
});

#[inline(always)]
pub fn attack_map(pos: Pos) -> &'static Bitboard {
    &ATTACK_MAPS[pos.0 as usize]
}

#[rustfmt::skip]
const KING_MIDDLEGAME_TABLE: [i32; 64] = [
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -10,-20,-20,-20,-20,-20,-20,-10,
     20, 20,  0,  0,  0,  0, 20, 20,
     20, 30, 10,  0,  0, 10, 30, 20
];

impl SquareBonus for King {
    fn square_bonus(pos: crate::Pos) -> i32 {
        KING_MIDDLEGAME_TABLE[pos.0 as usize]
    }
}

impl ChessPiece for King {
    fn piece_type() -> crate::PieceType {
        crate::PieceType::King
    }

    fn to_unicode() -> &'static str {
        "♚"
    }
}

static WHITE_QUEEN_SIDE: LazyLock<Bitboard> = LazyLock::new(|| {
    let mut board = Bitboard::new();
    board.set(pos::B1);
    board.set(pos::C1);
    board.set(pos::D1);
    board
});

static WHITE_KING_SIDE: LazyLock<Bitboard> = LazyLock::new(|| {
    let mut board = Bitboard::new();
    board.set(pos::F1);
    board.set(pos::G1);
    board
});

static BLACK_QUEEN_SIDE: LazyLock<Bitboard> = LazyLock::new(|| {
    let mut board = Bitboard::new();
    board.set(pos::E1);
    board.set(pos::F1);
    board.set(pos::G1);
    board
});

static BLACK_KING_SIDE: LazyLock<Bitboard> = LazyLock::new(|| {
    let mut board = Bitboard::new();
    board.set(pos::B1);
    board.set(pos::C1);
    board
});

impl CanMove for King {
    fn get_legal_moves(piece: &Piece, position: &Position, exclude_white: bool) -> Bitboard {
        let pos = piece.position;

        let mut board = ATTACK_MAPS[pos.0 as usize] & !position.white_map;
        let all = position.all_map;

        let maps = position.get_piece_maps();

        if position.true_active_color == Color::White {
            if position.castling_rights.white_queen_side {
                if !all.intersects(*WHITE_QUEEN_SIDE)
                    && maps.white_rooks.get(pos::A1)
                    && !King::is_white_king_in_check(&position)
                {
                    board.set(pos::C1);
                }
            }

            if position.castling_rights.white_king_side {
                if !all.intersects(*WHITE_KING_SIDE)
                    && maps.white_rooks.get(pos::H1)
                    && !King::is_white_king_in_check(&position)
                {
                    board.set(pos::G1);
                }
            }
        } else {
            // Black, but piece.color is white and we're on the first row, so we invert
            // the queen side and king side castling squares because the chess board is mirrored,
            // not rotationally symmetrical
            if position.castling_rights.black_queen_side {
                // Black queen side is e1 + f1 + g1
                if !all.intersects(*BLACK_QUEEN_SIDE)
                    && maps.white_rooks.get(pos::H1)
                    && !King::is_white_king_in_check(&position)
                {
                    board.set(pos::F1);
                }
            }

            if position.castling_rights.black_king_side {
                // Black king side is b1 + c1
                if !all.intersects(*BLACK_KING_SIDE)
                    && maps.white_rooks.get(pos::A1)
                    && !King::is_white_king_in_check(&position)
                {
                    board.set(pos::B1);
                }
            }
        }

        if exclude_white {
            board & !position.white_map
        } else {
            board
        }
    }
}

impl King {
    pub fn is_white_king_in_check(position: &Position) -> bool {
        let king = position.white_king;
        let maps = position.get_piece_maps();

        match king {
            Some(king) => {
                // Pawns
                if maps.black_pawns.intersects(*pawn::attack_map(king)) {
                    return true;
                }
                // Use magic bitboards to get sliding piece attacks
                let blockers = position.all_map;

                // Get all possible rook attacks from king's position
                let rook_attacks = super::rook::magic::get_rook_moves_magic(king, blockers);
                let rook_attackers = rook_attacks & position.black_map;

                // Check if any of those squares have a rook or queen
                if rook_attackers.into_iter().any(|pos| {
                    position.is_piece_at(pos, &[PieceType::Rook, PieceType::Queen], Color::Black)
                }) {
                    return true;
                }

                // Get all possible bishop attacks from king's position
                let bishop_attacks = super::bishop::magic::get_bishop_moves_magic(king, blockers);
                let bishop_attackers = bishop_attacks & position.black_map;

                // Check if any of those squares have a bishop or queen
                if bishop_attackers.into_iter().any(|pos| {
                    position.is_piece_at(pos, &[PieceType::Bishop, PieceType::Queen], Color::Black)
                }) {
                    return true;
                }

                let knight_attackers = *super::knight::attack_map(king) & position.black_map;
                if knight_attackers
                    .into_iter()
                    .any(|pos| position.is_piece_at(pos, &[PieceType::Knight], Color::Black))
                {
                    return true;
                }

                let king_attackers = *attack_map(king) & position.black_map;
                if king_attackers
                    .into_iter()
                    .any(|pos| position.is_piece_at(pos, &[PieceType::King], Color::Black))
                {
                    return true;
                }

                false
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Bitboard, Piece, PieceType, Pos, Position};

    #[test]
    pub fn move_king_empty_spaces() {
        let king = Piece::new_white(PieceType::King, (4, 4).into());

        let position = Position {
            white_map: Default::default(),
            black_map: Default::default(),
            ..Default::default()
        };

        let legal_moves = king.get_legal_moves(&position, true);

        assert_eq!(
            legal_moves,
            r#"
            00000000
            00000000
            00000000
            00011100
            00010100
            00011100
            00000000
            00000000
        "#
            .parse()
            .unwrap()
        );
    }
    #[test]
    fn test_white_kingside_castling() {
        // Setup initial position with king and rook in starting positions
        let king = Piece::new_white(PieceType::King, Pos::from_algebraic("e1").unwrap());
        let mut position = Position::start_position();

        position
            .remove_piece_at(Pos::from_algebraic("f1").unwrap())
            .unwrap();
        position
            .remove_piece_at(Pos::from_algebraic("g1").unwrap())
            .unwrap();

        let legal_moves = king.get_legal_moves(&position, true);

        // Verify that g1 (kingside castle square) is a legal move
        assert!(legal_moves.get(Pos::from_algebraic("g1").unwrap()));
    }

    #[test]
    fn test_white_kingside_castling_blocked() {
        // Setup position with piece blocking castling
        let king = Piece::new_white(PieceType::King, Pos::from_algebraic("e1").unwrap());
        let mut white_map: Bitboard = Default::default();
        white_map.set(Pos::from_algebraic("f1").unwrap()); // Place blocking piece

        let mut position = Position::start_position();

        position
            .remove_piece_at(Pos::from_algebraic("f1").unwrap())
            .unwrap();

        let legal_moves = king.get_legal_moves(&position, true);

        // Verify that g1 is not a legal move when blocked
        assert!(!legal_moves.get(Pos::from_algebraic("g1").unwrap()));
    }

    #[test]
    fn test_white_queenside_castling() {
        let king = Piece::new_white(PieceType::King, Pos::from_algebraic("e1").unwrap());
        let mut position = Position::start_position();

        position
            .remove_piece_at(Pos::from_algebraic("b1").unwrap())
            .unwrap();
        position
            .remove_piece_at(Pos::from_algebraic("c1").unwrap())
            .unwrap();
        position
            .remove_piece_at(Pos::from_algebraic("d1").unwrap())
            .unwrap();

        let legal_moves = king.get_legal_moves(&position, true);

        // Verify that c1 (queenside castle square) is a legal move
        assert!(legal_moves.get(Pos::from_algebraic("c1").unwrap()));
    }

    #[test]
    fn test_white_queenside_castling_blocked() {
        let king = Piece::new_white(PieceType::King, Pos::from_algebraic("e1").unwrap());
        let mut position = Position::start_position();

        position
            .remove_piece_at(Pos::from_algebraic("b1").unwrap())
            .unwrap();
        position
            .remove_piece_at(Pos::from_algebraic("c1").unwrap())
            .unwrap();

        let legal_moves = king.get_legal_moves(&position, true);

        // Verify that c1 (queenside castle square) is a legal move
        assert!(!legal_moves.get(Pos::from_algebraic("c1").unwrap()));
    }

    #[test]
    fn test_castling_rights_disabled() {
        let king = Piece::new_white(PieceType::King, Pos::from_algebraic("e1").unwrap());
        let mut position = Position::start_position();

        position
            .remove_piece_at(Pos::from_algebraic("b1").unwrap())
            .unwrap();

        let legal_moves = king.get_legal_moves(&position, true);

        // Verify neither castling move is legal when rights are disabled
        assert!(!legal_moves.get(Pos::from_algebraic("g1").unwrap()));
        assert!(!legal_moves.get(Pos::from_algebraic("c1").unwrap()));
    }

    #[test]
    fn test_black_kingside_castling() {
        let mut position = Position::start_position();
        position.invert();

        position
            .remove_piece_at(Pos::from_algebraic("b1").unwrap())
            .unwrap();
        position
            .remove_piece_at(Pos::from_algebraic("c1").unwrap())
            .unwrap();

        let legal_moves = position
            .get_piece_at(position.white_king.unwrap())
            .unwrap()
            .get_legal_moves(&position, true);

        assert!(legal_moves.get(Pos::from_algebraic("b1").unwrap()));
    }

    #[test]
    fn test_black_kingside_castling_blocked() {
        let mut position = Position::start_position();
        position.invert();

        position
            .remove_piece_at(Pos::from_algebraic("b1").unwrap())
            .unwrap();

        let legal_moves = position
            .get_piece_at(position.white_king.unwrap())
            .unwrap()
            .get_legal_moves(&position, true);

        assert!(!legal_moves.get(Pos::from_algebraic("b1").unwrap()));
    }

    #[test]
    fn test_black_queenside_castling() {
        let mut position = Position::start_position();
        position.invert();

        position
            .remove_piece_at(Pos::from_algebraic("e1").unwrap())
            .unwrap();
        position
            .remove_piece_at(Pos::from_algebraic("f1").unwrap())
            .unwrap();
        position
            .remove_piece_at(Pos::from_algebraic("g1").unwrap())
            .unwrap();

        let legal_moves = position
            .get_piece_at(position.white_king.unwrap())
            .unwrap()
            .get_legal_moves(&position, true);

        assert!(legal_moves.get(Pos::from_algebraic("f1").unwrap()));
    }

    #[test]
    fn test_black_queenside_castling_blocked() {
        let mut position = Position::start_position();
        position.invert();

        position
            .remove_piece_at(Pos::from_algebraic("e1").unwrap())
            .unwrap();
        position
            .remove_piece_at(Pos::from_algebraic("f1").unwrap())
            .unwrap();

        let legal_moves = position
            .get_piece_at(position.white_king.unwrap())
            .unwrap()
            .get_legal_moves(&position, true);

        assert!(!legal_moves.get(Pos::from_algebraic("f1").unwrap()));
    }

    #[test]
    fn castle_through_bishop() {
        let position = Position::parse_from_fen(
            "r2qkbnr/ppp1pppp/2n5/3p2B1/3PP3/2N5/PPP2PPP/R2bKBNR w KQkq - 0 1",
        )
        .unwrap();

        let legal_moves = position
            .get_piece_at(position.white_king.unwrap())
            .unwrap()
            .get_legal_moves(&position, true);

        println!("{}", legal_moves);
    }
}
