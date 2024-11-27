pub mod square_bonus;

use crate::{piece_move::MoveType, Color, PieceMove, PieceType, Pos, Position};

pub fn piece_value(piece_type: PieceType) -> i32 {
    match piece_type {
        PieceType::Pawn => 100,
        PieceType::Knight => 320,
        PieceType::Bishop => 330,
        PieceType::Rook => 500,
        PieceType::Queen => 900,
        PieceType::King => 20000,
    }
}

// Mobility bonuses (in centipawns)
fn mobility_bonus(piece_type: PieceType) -> i32 {
    match piece_type {
        PieceType::Pawn => 0, // Pawns: mobility not counted (evaluated via structure instead)
        PieceType::Knight => 4, // Knight: 4 points per legal move
        PieceType::Bishop => 3, // Bishop: 3 points per legal move
        PieceType::Rook => 2, // Rook: 2 points per legal move
        PieceType::Queen => 1, // Queen: 1 point per legal move (since they have many moves)
        PieceType::King => 0, // King (mobility not rewarded in middlegame)
    }
}

#[derive(PartialEq, Eq)]
struct ScoredMove {
    score: i32,
    mv: PieceMove,
}

impl PartialOrd for ScoredMove {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.score.cmp(&other.score))
    }
}

impl Ord for ScoredMove {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score.cmp(&other.score)
    }
}

pub fn order_moves(
    position: &Position,
    moves: Vec<PieceMove>,
    prev_best_move: Option<PieceMove>,
) -> Vec<PieceMove> {
    let mut scored_moves: Vec<ScoredMove> = moves
        .into_iter()
        .map(|mv| {
            let score = score_move(position, &mv, prev_best_move);
            ScoredMove { score, mv }
        })
        .collect();

    // Sort in descending order (highest score first)
    scored_moves.sort_by(|a, b| b.score.cmp(&a.score));

    scored_moves.into_iter().map(|sm| sm.mv).collect()
}

fn score_move(_position: &Position, mv: &PieceMove, prev_best_move: Option<PieceMove>) -> i32 {
    let mut score = 0;

    // 1. Hash table move from previous iteration
    if prev_best_move.is_some() && *mv == prev_best_move.unwrap() {
        return 20000; // Highest priority
    }

    // 2. Captures, scored by MVV-LVA (Most Valuable Victim - Least Valuable Aggressor)
    if mv.is_capture() {
        // Base capture score
        score += 10000;

        // Add MVV-LVA scoring
        // Victim value - try to capture most valuable pieces first

        if let MoveType::Capture(captured_piece) = mv.move_type {
            score += piece_value(captured_piece) * 100;
        }

        // Subtract attacker value - prefer capturing with less valuable pieces
        score -= piece_value(mv.piece_type) * 10;
    }

    // 3. Killer moves (moves that caused beta cutoffs at the same ply in other branches)
    // This would require storing killer moves in the search state
    // score += check_if_killer_move(mv) * 9000;

    // 4. Special moves
    if let MoveType::Promotion(_) = mv.move_type {
        score += 8000;
    }

    // if mv.is_check() {
    //     score += 7000;
    // }

    // 5. Piece-square table bonuses
    // score += get_piece_square_bonus(mv.piece(), mv.to_square());

    // 6. History heuristic (moves that were good in earlier positions)
    // This would require maintaining a history table
    // score += get_history_score(mv);

    score
}

pub fn evaluate_position(board: &Position) -> i32 {
    let mut score = 0;

    let inverted = board.inverted();

    // Bonuses for white pieces
    for piece in board.white_pieces.iter() {
        let value = piece_value(piece.piece_type);
        let piece_score = value + piece.square_bonus();

        score += piece_score;

        let holding_value = match piece.holding {
            Some(piece_type) => piece_value(piece_type),
            None => 0,
        };

        score += holding_value;
    }

    // Penalties for black pieces
    for piece in inverted.white_pieces.iter() {
        let value = piece_value(piece.piece_type);
        let piece_score = value + piece.square_bonus();

        score -= piece_score;

        let holding_value = match piece.holding {
            Some(piece_type) => piece_value(piece_type),
            None => 0,
        };

        score -= holding_value;
    }

    if has_bishop_pair(board, Color::White) {
        score += 50;
    }

    if has_bishop_pair(board, Color::Black) {
        score -= 50;
    }

    // Evaluate pawn structure
    let white_pawn_score = evaluate_pawn_structure(board);
    let black_pawn_score = evaluate_pawn_structure(&inverted);
    score += white_pawn_score - black_pawn_score;

    // King safety evaluation
    let white_king_safety = evaluate_king_safety(board);
    let black_king_safety = evaluate_king_safety(&inverted);
    score += white_king_safety - black_king_safety;

    // Mobility evaluation
    let white_mobility = evaluate_mobility(board);
    let black_mobility = evaluate_mobility(&inverted);
    score += white_mobility - black_mobility;

    score
}

fn has_bishop_pair(position: &Position, color: Color) -> bool {
    let mut light_square_bishop = false;
    let mut dark_square_bishop = false;

    let pieces = if color == Color::White {
        &position.white_pieces
    } else {
        &position.black_pieces
    };

    for piece in pieces.iter() {
        if piece.color == color && piece.piece_type == PieceType::Bishop {
            if (piece.position.0 + piece.position.get_row()) % 2 == 0 {
                light_square_bishop = true;
            } else {
                dark_square_bishop = true;
            }
        }
    }

    light_square_bishop && dark_square_bishop
}

fn evaluate_pawn_structure(position: &Position) -> i32 {
    let mut white_score = 0;

    let mut white_pawns = vec![];
    let mut black_pawns = vec![];

    for piece in position.white_pieces.iter() {
        if piece.piece_type == PieceType::Pawn {
            white_pawns.push(piece.position);
        }
    }

    for piece in position.black_pieces.iter() {
        if piece.piece_type == PieceType::Pawn {
            black_pawns.push(piece.position);
        }
    }

    // Evaluate doubled pawns (penalize)
    for file in 0..8 {
        let white_pawns_in_file = white_pawns.iter().filter(|p| p.get_col() == file).count();

        if white_pawns_in_file > 1 {
            white_score -= 20 * (white_pawns_in_file as i32 - 1); // -0.2 per doubled pawn
        }
    }

    // Evaluate isolated pawns (penalize)
    for pawn_pos in white_pawns.iter() {
        let file = pawn_pos.get_col();
        let has_neighbor = white_pawns.iter().any(|p| {
            (file > 0 && p.get_col() == file - 1) || (file < 7 && p.get_col() == file + 1)
        });
        if !has_neighbor {
            white_score -= 30; // -0.3 for isolated pawn
        }
    }

    // Evaluate passed pawns (bonus)
    for pawn_pos in white_pawns.iter() {
        let file = pawn_pos.get_col();
        let rank = pawn_pos.get_row();
        let is_passed = !black_pawns.iter().any(|p| {
            p.get_row() < rank  // Checks if any black pawns are ahead of this white pawn
                && (p.get_col() == file // On same file
                    || (file > 0 && p.get_col() == file - 1) // Or adjacent left file
                    || (file < 7 && p.get_col() == file + 1)) // Or adjacent right file
        });
        if is_passed {
            white_score += 50 + (7 - (rank as i32)) * 10; // Progressive bonus for advancement
        }
    }

    white_score
}

fn evaluate_king_safety(position: &Position) -> i32 {
    let mut white_score = 0;

    let white_king = position.white_king;

    if let Some(white_king) = white_king {
        // Pawn shield bonus
        let shield_positions = [
            (
                white_king.position.get_col() as i32,
                (white_king.position.get_row() as i32) - 1,
            ),
            (
                (white_king.position.get_col() as i32 - 1),
                (white_king.position.get_row() as i32 - 1),
            ),
            (
                (white_king.position.get_col() as i32) + 1,
                (white_king.position.get_row() as i32) - 1,
            ),
        ];

        for (file, rank) in shield_positions.iter() {
            if *file > 0 && *file < 8 && *rank > 0 && *rank < 8 {
                let pos = Pos::xy(*file as u8, *rank as u8);
                if let Some(piece) = position.get_piece_at(pos) {
                    if piece.piece_type == PieceType::Pawn && piece.color == Color::White {
                        white_score += 20; // Bonus for each pawn protecting the king
                    }
                }
            }
        }
    }

    white_score
}

fn evaluate_mobility(position: &Position) -> i32 {
    let mut white_score = 0;

    // Calculate legal moves for each piece
    for piece in position.white_pieces.iter() {
        let moves = piece.get_legal_moves(position.white_map, position.black_map);
        let move_count = moves.count() as i32;

        let mobility_bonus = mobility_bonus(piece.piece_type) * move_count;

        white_score += mobility_bonus;
    }

    white_score
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Color, Piece, PieceType, Pos, Position};

    #[test]
    fn test_has_bishop_pair_starting_position() {
        let position = Position::start_position();

        // In starting position, both colors have bishop pairs
        assert!(has_bishop_pair(&position, Color::White));
        assert!(has_bishop_pair(&position, Color::Black));
    }

    #[test]
    fn test_has_bishop_pair_one_bishop() {
        // Create a position with only one bishop for each color
        let mut position = Position::new(
            vec![
                // White pieces
                Piece::new(
                    PieceType::Bishop,
                    Color::White,
                    Pos::from_algebraic("c1").unwrap(),
                ),
                // Black pieces
                Piece::new(
                    PieceType::Bishop,
                    Color::Black,
                    Pos::from_algebraic("c8").unwrap(),
                ),
            ],
            Default::default(), // castling rights
            None,               // en passant
            0,                  // halfmove clock
            1,                  // fullmove number
        );
        position.calc_changes(true);

        // Neither color should have a bishop pair
        assert!(!has_bishop_pair(&position, Color::White));
        assert!(!has_bishop_pair(&position, Color::Black));
    }

    #[test]
    fn test_has_bishop_pair_different_colored_squares() {
        // Create a position with bishops on different colored squares
        let mut position = Position::new(
            vec![
                // White bishops on light and dark squares
                Piece::new(
                    PieceType::Bishop,
                    Color::White,
                    Pos::from_algebraic("c1").unwrap(),
                ),
                Piece::new(
                    PieceType::Bishop,
                    Color::White,
                    Pos::from_algebraic("f1").unwrap(),
                ),
            ],
            Default::default(),
            None,
            0,
            1,
        );
        position.calc_changes(true);

        // White should have a bishop pair
        assert!(has_bishop_pair(&position, Color::White));
        // Black should not have any bishops
        assert!(!has_bishop_pair(&position, Color::Black));
    }

    #[test]
    fn test_has_bishop_pair_same_colored_squares() {
        // Create a position with bishops on same colored squares
        let mut position = Position::new(
            vec![
                // White bishops both on light squares
                Piece::new(
                    PieceType::Bishop,
                    Color::White,
                    Pos::from_algebraic("c1").unwrap(),
                ),
                Piece::new(
                    PieceType::Bishop,
                    Color::White,
                    Pos::from_algebraic("e3").unwrap(),
                ),
            ],
            Default::default(),
            None,
            0,
            1,
        );
        position.calc_changes(true);

        // White should not have a bishop pair (same colored squares)
        assert!(!has_bishop_pair(&position, Color::White));
        // Black should not have any bishops
        assert!(!has_bishop_pair(&position, Color::Black));
    }

    #[test]
    fn test_has_bishop_pair_mixed_pieces() {
        // Create a position with other pieces to ensure we only count bishops
        let mut position = Position::new(
            vec![
                // White pieces
                Piece::new(
                    PieceType::Bishop,
                    Color::White,
                    Pos::from_algebraic("c1").unwrap(),
                ),
                Piece::new(
                    PieceType::Bishop,
                    Color::White,
                    Pos::from_algebraic("f1").unwrap(),
                ),
                Piece::new(
                    PieceType::Knight,
                    Color::White,
                    Pos::from_algebraic("d4").unwrap(),
                ),
                // Black pieces
                Piece::new(
                    PieceType::Bishop,
                    Color::Black,
                    Pos::from_algebraic("c8").unwrap(),
                ),
                Piece::new(
                    PieceType::Queen,
                    Color::Black,
                    Pos::from_algebraic("d8").unwrap(),
                ),
            ],
            Default::default(),
            None,
            0,
            1,
        );
        position.calc_changes(true);

        // White should have a bishop pair
        assert!(has_bishop_pair(&position, Color::White));
        // Black should not have a bishop pair
        assert!(!has_bishop_pair(&position, Color::Black));
    }

    #[test]
    fn test_evaluate_pawn_structure_starting_position() {
        let position = Position::start_position();
        let white_score = evaluate_pawn_structure(&position);
        let black_score = evaluate_pawn_structure(&position.inverted());

        // Starting position should be equal and have no penalties
        assert_eq!(white_score, 0);
        assert_eq!(black_score, 0);
    }

    #[test]
    fn test_evaluate_pawn_structure_doubled_pawns() {
        // Position with doubled pawns for white on e-file
        let position = Position::parse_from_fen(
            "r1bqkbnr/ppp1pppp/2n5/3P4/3P4/8/PPP2PPP/RNBQKBNR w KQkq - 0 1",
        )
        .unwrap();
        let white_score = evaluate_pawn_structure(&position);
        let black_score = evaluate_pawn_structure(&position.inverted());

        // White should be penalized for doubled pawns
        assert!(white_score < 0);
        // Penalty should be -20 for one doubled pawn
        assert_eq!(white_score, -20);
        // Black should not have doubled pawns
        assert_eq!(black_score, 0);
    }

    #[test]
    fn test_evaluate_pawn_structure_isolated_pawns() {
        // Position with isolated d-pawn for white
        let position =
            Position::parse_from_fen("rnbqkbnr/pppppppp/8/8/3P4/8/PP3PPP/RNBQKBNR w KQkq - 0 1")
                .unwrap();
        let white_score = evaluate_pawn_structure(&position);
        let black_score = evaluate_pawn_structure(&position.inverted());

        // White should be penalized for isolated pawn
        assert_eq!(white_score, -30); // -30 for isolated pawn
        assert_eq!(black_score, 0);
    }

    #[test]
    fn test_evaluate_pawn_structure_passed_pawn() {
        // Position with passed d-pawn for white
        let position =
            Position::parse_from_fen("rnbqkbnr/p3pppp/8/8/1pPp4/8/PP1PPPPP/RNBQKBNR w KQkq - 0 1")
                .unwrap();
        let white_score = evaluate_pawn_structure(&position);
        let black_score = evaluate_pawn_structure(&position.inverted());

        // White should get a bonus for the passed pawn
        // Base bonus of 50 plus rank bonus (pawn on 5th rank = 3 ranks advanced = 30)
        assert_eq!(white_score, 80);
        assert_eq!(black_score, 0);
    }

    #[test]
    fn test_evaluate_pawn_structure_multiple_issues() {
        // Position with both doubled and isolated pawns for both sides
        let position = Position::parse_from_fen(
            "rnbqkbnr/p1p2p1p/3p4/3p4/3P4/3P4/P1P2P1P/RNBQKBNR w KQkq - 0 1",
        )
        .unwrap();
        let white_score = evaluate_pawn_structure(&position);
        let black_score = evaluate_pawn_structure(&position.inverted());

        // Both sides should have penalties for doubled and isolated pawns
        // -20 for doubled pawns, -30 for isolated pawns = -50
        assert_eq!(white_score, -110);
        assert_eq!(black_score, -110);
    }

    #[test]
    fn test_evaluate_king_safety_starting_position() {
        let position = Position::start_position();
        let white_score = evaluate_king_safety(&position);
        let black_score = evaluate_king_safety(&position.inverted());

        // In starting position, both kings have full pawn shield (3 pawns)
        // Each pawn gives 20 points bonus
        assert_eq!(white_score, 60);
        assert_eq!(black_score, 60);
    }

    #[test]
    fn test_evaluate_king_safety_exposed_king() {
        // Position with exposed white king (no pawn shield)
        let mut position = Position::new(
            vec![
                Piece::new(
                    PieceType::King,
                    Color::White,
                    Pos::from_algebraic("e1").unwrap(),
                ),
                // Some other pieces but no pawns in front of king
                Piece::new(
                    PieceType::Queen,
                    Color::White,
                    Pos::from_algebraic("d1").unwrap(),
                ),
                Piece::new(
                    PieceType::Bishop,
                    Color::White,
                    Pos::from_algebraic("c1").unwrap(),
                ),
            ],
            Default::default(),
            None,
            0,
            1,
        );
        position.calc_changes(true);

        let white_score = evaluate_king_safety(&position);
        // No pawn shield means no bonus
        assert_eq!(white_score, 0);
    }

    #[test]
    fn test_evaluate_king_safety_partial_shield() {
        // Position with partial pawn shield (2 pawns)
        let mut position = Position::new(
            vec![
                Piece::new(
                    PieceType::King,
                    Color::White,
                    Pos::from_algebraic("e1").unwrap(),
                ),
                Piece::new(
                    PieceType::Pawn,
                    Color::White,
                    Pos::from_algebraic("e2").unwrap(),
                ),
                Piece::new(
                    PieceType::Pawn,
                    Color::White,
                    Pos::from_algebraic("f2").unwrap(),
                ),
            ],
            Default::default(),
            None,
            0,
            1,
        );
        position.calc_changes(true);

        let white_score = evaluate_king_safety(&position);
        // Two pawns in shield = 40 points
        assert_eq!(white_score, 40);
    }

    #[test]
    fn test_evaluate_king_safety_full_shield() {
        // Position with full pawn shield (3 pawns)
        let mut position = Position::new(
            vec![
                Piece::new(
                    PieceType::King,
                    Color::White,
                    Pos::from_algebraic("e1").unwrap(),
                ),
                Piece::new(
                    PieceType::Pawn,
                    Color::White,
                    Pos::from_algebraic("d2").unwrap(),
                ),
                Piece::new(
                    PieceType::Pawn,
                    Color::White,
                    Pos::from_algebraic("e2").unwrap(),
                ),
                Piece::new(
                    PieceType::Pawn,
                    Color::White,
                    Pos::from_algebraic("f2").unwrap(),
                ),
            ],
            Default::default(),
            None,
            0,
            1,
        );
        position.calc_changes(true);

        let white_score = evaluate_king_safety(&position);
        // Three pawns in shield = 60 points
        assert_eq!(white_score, 60);
    }

    #[test]
    fn test_evaluate_king_safety_edge_case() {
        // Test king safety evaluation when king is on the edge of the board
        let mut position = Position::new(
            vec![
                Piece::new(
                    PieceType::King,
                    Color::White,
                    Pos::from_algebraic("h1").unwrap(),
                ),
                Piece::new(
                    PieceType::Pawn,
                    Color::White,
                    Pos::from_algebraic("g2").unwrap(),
                ),
                Piece::new(
                    PieceType::Pawn,
                    Color::White,
                    Pos::from_algebraic("h2").unwrap(),
                ),
            ],
            Default::default(),
            None,
            0,
            1,
        );
        position.calc_changes(true);

        let white_score = evaluate_king_safety(&position);
        // Two pawns possible at edge = 40 points
        assert_eq!(white_score, 40);
    }

    #[test]
    fn test_evaluate_king_safety_no_king() {
        // Test position with no white king (edge case)
        let mut position = Position::new(
            vec![
                Piece::new(
                    PieceType::Queen,
                    Color::White,
                    Pos::from_algebraic("d1").unwrap(),
                ),
                Piece::new(
                    PieceType::Pawn,
                    Color::White,
                    Pos::from_algebraic("e2").unwrap(),
                ),
            ],
            Default::default(),
            None,
            0,
            1,
        );
        position.calc_changes(true);

        let white_score = evaluate_king_safety(&position);
        // No king means no safety score
        assert_eq!(white_score, 0);
    }

    #[test]
    fn test_evaluate_mobility_starting_position() {
        let position = Position::start_position();
        let white_score = evaluate_mobility(&position);
        let black_score = evaluate_mobility(&position.inverted());

        // In starting position:
        // Knights: 2 possible moves each (4 total) * 4 points = 16
        // No legal moves for other pieces
        assert_eq!(white_score, 16);
        assert_eq!(black_score, 16);
    }

    #[test]
    fn test_evaluate_mobility_single_knight() {
        // Position with single knight in center with maximum mobility
        let mut position = Position::new(
            vec![Piece::new(
                PieceType::Knight,
                Color::White,
                Pos::from_algebraic("e4").unwrap(),
            )],
            Default::default(),
            None,
            0,
            1,
        );
        position.calc_changes(true);

        let white_score = evaluate_mobility(&position);
        // Knight in center has 8 possible moves * 4 points = 32
        assert_eq!(white_score, 32);
    }

    #[test]
    fn test_evaluate_mobility_single_bishop() {
        // Position with single bishop in center
        let mut position = Position::new(
            vec![Piece::new(
                PieceType::Bishop,
                Color::White,
                Pos::from_algebraic("e4").unwrap(),
            )],
            Default::default(),
            None,
            0,
            1,
        );
        position.calc_changes(true);

        let white_score = evaluate_mobility(&position);
        // Bishop in center has 13 possible moves * 3 points = 39
        assert_eq!(white_score, 39);
    }

    #[test]
    fn test_evaluate_mobility_single_rook() {
        // Position with single rook in center
        let mut position = Position::new(
            vec![Piece::new(
                PieceType::Rook,
                Color::White,
                Pos::from_algebraic("e4").unwrap(),
            )],
            Default::default(),
            None,
            0,
            1,
        );
        position.calc_changes(true);

        let white_score = evaluate_mobility(&position);
        // Rook in center has 14 possible moves * 2 points = 28
        assert_eq!(white_score, 28);
    }

    #[test]
    fn test_evaluate_mobility_single_queen() {
        // Position with single queen in center
        let mut position = Position::new(
            vec![Piece::new(
                PieceType::Queen,
                Color::White,
                Pos::from_algebraic("e4").unwrap(),
            )],
            Default::default(),
            None,
            0,
            1,
        );
        position.calc_changes(true);

        let white_score = evaluate_mobility(&position);
        // Queen in center has 27 possible moves * 1 point = 27
        assert_eq!(white_score, 27);
    }

    #[test]
    fn test_evaluate_mobility_blocked_pieces() {
        // Position with pieces blocked by friendly pieces
        let position = Position::parse_from_fen("8/8/8/8/8/8/PPPPP3/RPBQP3 w - - 0 1").unwrap();

        let white_score = evaluate_mobility(&position);
        // Only pawns can move (1 square each) * 0 points = 0
        assert_eq!(white_score, 0);
    }

    #[test]
    fn test_evaluate_mobility_mixed_pieces() {
        // Position with multiple pieces in open positions
        let mut position = Position::new(
            vec![
                // Central knight and bishop
                Piece::new(
                    PieceType::Knight,
                    Color::White,
                    Pos::from_algebraic("e4").unwrap(),
                ),
                Piece::new(
                    PieceType::Bishop,
                    Color::White,
                    Pos::from_algebraic("d4").unwrap(),
                ),
            ],
            Default::default(),
            None,
            0,
            1,
        );
        position.calc_changes(true);

        let white_score = evaluate_mobility(&position);
        // Knight: 8 moves * 4 points = 32
        // Bishop: 13 moves * 3 points = 39
        // Total = 71
        assert_eq!(white_score, 71);
    }

    #[test]
    fn test_evaluate_mobility_king_pawns() {
        // Test that kings and pawns don't contribute to mobility score
        let mut position = Position::new(
            vec![
                Piece::new(
                    PieceType::King,
                    Color::White,
                    Pos::from_algebraic("e4").unwrap(),
                ),
                Piece::new(
                    PieceType::Pawn,
                    Color::White,
                    Pos::from_algebraic("d4").unwrap(),
                ),
            ],
            Default::default(),
            None,
            0,
            1,
        );
        position.calc_changes(true);

        let white_score = evaluate_mobility(&position);
        // King: 8 moves * 0 points = 0
        // Pawn: 1 move * 0 points = 0
        // Total = 0
        assert_eq!(white_score, 0);
    }

    #[test]
    fn bad_opening_knight() {
        // Opening with the knight against the edge of the board
        let position =
            Position::parse_from_fen("rnbqkbnr/pppppppp/8/8/8/N7/PPPPPPPP/R1BQKBNR w KQkq - 0 1")
                .unwrap();

        let score = evaluate_position(&position);

        assert!(score < 20);
    }

    #[test]
    fn good_opening_pawn() {
        // Opening with e4, a strong move
        let position =
            Position::parse_from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1")
                .unwrap();

        let score = evaluate_position(&position);

        dbg!(score);
        assert!(score > 20);

        // Opening with d4, another strong move
        let position =
            Position::parse_from_fen("rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 1")
                .unwrap();

        let score = evaluate_position(&position);

        assert!(score > 20);
    }
}
