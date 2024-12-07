pub mod ordering;
pub mod square_bonus;

use crate::{
    piece::pawn, piece_move::GameType, search::alpha_beta::SearchParams, Bitboard, Color,
    PieceType, Pos, Position,
};

pub fn evaluate_position(board: &Position, _game_type: GameType, params: &SearchParams) -> i32 {
    let mut score = 0;

    let inverted = board.inverted();

    // Bonuses for white pieces
    for piece in board.white_pieces.iter() {
        if let Some(piece) = piece {
            let value = piece_value(piece.piece_type);
            let piece_score = value + piece.square_bonus();

            score += piece_score * params.weights.material / 100;

            let holding_value = match piece.holding {
                Some(piece_type) => piece_value(piece_type),
                None => 0,
            };

            score += holding_value * params.weights.material / 100;
        }
    }

    // Penalties for black pieces
    for piece in inverted.white_pieces.iter() {
        if let Some(piece) = piece {
            let value = piece_value(piece.piece_type);
            let piece_score = value + piece.square_bonus();

            score -= piece_score * params.weights.material / 100;

            let holding_value = match piece.holding {
                Some(piece_type) => piece_value(piece_type),
                None => 0,
            };

            score -= holding_value * params.weights.material / 100;
        }
    }

    if params.features.evaluate_bishop_pairs {
        if has_bishop_pair(board, Color::White) {
            score += 50 * params.weights.bishop_pair / 100;
        }

        if has_bishop_pair(board, Color::Black) {
            score -= 50 * params.weights.bishop_pair / 100;
        }
    }

    // Evaluate pawn structure
    if params.features.evaluate_pawn_structure {
        let white_pawn_score = evaluate_pawn_structure(board);
        let black_pawn_score = evaluate_pawn_structure(&inverted);
        score += (white_pawn_score - black_pawn_score) * params.weights.pawn_structure / 100;
    }

    // King safety evaluation
    if params.features.evaluate_king_safety {
        let white_king_safety = evaluate_king_safety(board);
        let black_king_safety = evaluate_king_safety(&inverted);
        score += (white_king_safety - black_king_safety) * params.weights.king_safety / 100;
    }

    // Mobility evaluation
    if params.features.evaluate_mobility {
        let white_mobility = evaluate_mobility(board);
        let black_mobility = evaluate_mobility(&inverted);
        score += (white_mobility - black_mobility) * params.weights.mobility / 100;
    }

    if params.features.evaluate_piece_coordination {
        let white_coordination = evaluate_piece_coordination(board);
        let black_coordination = evaluate_piece_coordination(&inverted);
        score +=
            (white_coordination - black_coordination) * params.weights.piece_coordination / 100;
    }

    if params.features.evaluate_pawn_control {
        let white_pawn_control = evaluate_pawn_control(board);
        let black_pawn_control = evaluate_pawn_control(&inverted);
        score += (white_pawn_control - black_pawn_control) * params.weights.pawn_control / 100;
    }

    if params.features.evaluate_piece_protection {
        let white_protection = evaluate_piece_protection(board, &inverted);
        let black_protection = evaluate_piece_protection(&inverted, board);
        score += (white_protection - black_protection) * params.weights.piece_protection / 100;
    }

    if params.features.evaluate_trapped_pieces {
        let white_trapped = evaluate_trapped_pieces(board);
        let black_trapped = evaluate_trapped_pieces(&inverted);
        score += (white_trapped - black_trapped) * params.weights.trapped_pieces / 100;
    }

    if params.features.evaluate_strategic_squares {
        let white_strategic = evaluate_strategic_squares(board);
        let black_strategic = evaluate_strategic_squares(&inverted);
        score += (white_strategic - black_strategic) * params.weights.strategic_squares / 100;
    }

    if params.features.evaluate_piece_pressure {
        let white_pressure = evaluate_piece_pressure(board, &inverted);
        let black_pressure = evaluate_piece_pressure(&inverted, board);
        score += (white_pressure - black_pressure) * params.weights.piece_pressure / 100;
    }

    if params.features.evaluate_pawn_structure_quality {
        let white_quality = evaluate_pawn_structure_quality(board, &inverted);
        let black_quality = evaluate_pawn_structure_quality(&inverted, board);
        score += (white_quality - black_quality) * params.weights.pawn_structure_quality / 100;
    }

    if params.features.evaluate_pawn_defense_quality {
        let white_defense = evaluate_pawn_defense_quality(board);
        let black_defense = evaluate_pawn_defense_quality(&inverted);
        score += (white_defense - black_defense) * params.weights.pawn_defense_quality / 100;
    }

    score
}

fn has_bishop_pair(position: &Position, color: Color) -> bool {
    let bishop_map = if color == Color::White {
        position.get_piece_maps().white_bishops
    } else {
        position.get_piece_maps().black_bishops
    };

    bishop_map.count() >= 2
        && bishop_map.intersects(Bitboard::light_squares())
        && bishop_map.intersects(Bitboard::dark_squares())
}

fn evaluate_pawn_structure(position: &Position) -> i32 {
    let maps = position.get_piece_maps();
    let mut score = 0;

    // Doubled pawns
    for file in 0..8 {
        let file_mask = Bitboard::for_file(file);
        let pawns_in_file = (maps.white_pawns & file_mask).count();
        if pawns_in_file > 1 {
            score -= 20 * (pawns_in_file as i32 - 1);
        }
    }

    // Isolated pawns
    for file in 0..8 {
        let pawns_on_file = maps.white_pawns & Bitboard::for_file(file);
        if pawns_on_file.count() > 0 {
            let adjacent_pawns = maps.white_pawns & Bitboard::adjacent_files(file);
            if adjacent_pawns.count() == 0 {
                score -= 30;
            }
        }
    }

    // Passed pawns
    for pawn_pos in maps.white_pawns.into_iter() {
        let file = pawn_pos.get_col();
        let rank = pawn_pos.get_row();

        // Check squares ahead in this and adjacent files
        let passed_mask = Bitboard::for_file(file) | Bitboard::adjacent_files(file);
        let passed_mask = passed_mask & Bitboard::ahead_of_rank_white(rank);

        if !maps.black_pawns.intersects(passed_mask) {
            score += 50 + (7 - rank as i32) * 10;
        }
    }

    score
}

fn evaluate_pawn_structure_quality(position: &Position, inverted: &Position) -> i32 {
    let mut score = 0;
    let maps = position.get_piece_maps();

    // Evaluate pawn chains
    for pawn_pos in maps.white_pawns.into_iter() {
        let file = pawn_pos.get_col();
        let rank = pawn_pos.get_row();

        // Check for pawn chain
        let protected_by_pawn = maps.white_pawns & *pawn::attack_map_black(pawn_pos);

        if protected_by_pawn.count() > 0 {
            score += 15; // Base chain bonus

            // Additional bonus for advanced chains
            score += (7 - rank) as i32 * 5;

            // Extra bonus for central chains
            if file >= 2 && file <= 5 {
                score += 10;
            }
        }

        // Evaluate pawn tension (pawns facing each other)
        let enemy_pawn_file = Bitboard::for_file(file) & maps.black_pawns;
        if enemy_pawn_file.count() > 0 {
            let enemy_pawn_rank = enemy_pawn_file.into_iter().next().unwrap().get_row();
            if (enemy_pawn_rank as i32 - rank as i32).abs() == 1 {
                // Reward tension maintenance in good positions
                if position.count_attackers(pawn_pos) >= inverted.count_attackers(pawn_pos.invert())
                {
                    score += 20; // Keep tension when stronger
                }
            }
        }
    }

    score
}

fn evaluate_king_safety(position: &Position) -> i32 {
    let maps = position.get_piece_maps();
    let mut score = 0;

    if let Some(king_pos) = maps.white_king.into_iter().next() {
        let rank = king_pos.get_row();
        let file = king_pos.get_col();

        if !is_endgame(position) {
            // Penalize king movement away from back rank in middlegame
            if rank < 6 {
                score -= 20 * (6 - rank) as i32;
            }

            // Create pawn shield mask one rank in front of king
            let shield_rank = rank - 1;
            let shield_mask = Bitboard::for_file(file) | Bitboard::adjacent_files(file);
            let shield_mask = shield_mask & Bitboard::for_rank(shield_rank);

            // Count pawns in shield position
            let shield_pawns = (maps.white_pawns & shield_mask).count();
            score += shield_pawns as i32 * 20;

            // Penalize open files near king
            let king_and_adjacent = Bitboard::for_file(file) | Bitboard::adjacent_files(file);
            let pawns_near_king = maps.white_pawns & king_and_adjacent;
            score -= 15 * (3 - pawns_near_king.count() as i32);
        } else {
            // In endgame, king should be active and near pawns
            let pawn_proximity = maps
                .white_pawns
                .into_iter()
                .map(|p| manhattan_distance(king_pos, p))
                .min()
                .unwrap_or(7);
            score += (7 - pawn_proximity as i32) * 10;

            // Bonus for centralized king in endgame
            let center_distance = manhattan_distance(king_pos, Pos::xy(3, 3));
            score += (7 - center_distance as i32) * 5;
        }
    }

    score
}

fn evaluate_pawn_control(position: &Position) -> i32 {
    let mut score = 0;
    let maps = position.get_piece_maps();

    // Evaluate control of key central squares
    let central_squares = Bitboard::center();
    let white_pawn_attacks = pawn::generate_pawn_attacks(maps.white_pawns);

    score += (white_pawn_attacks & central_squares).count() as i32 * 15;

    // Evaluate pawn breaks and tension
    for pawn_pos in maps.white_pawns.into_iter() {
        let file = pawn_pos.get_col();
        let rank = pawn_pos.get_row();

        // Reward potential pawn breaks
        let ahead_mask = Bitboard::ahead_of_rank_white(rank) & Bitboard::for_file(file);
        if (ahead_mask & maps.black_pawns).count() == 1 {
            score += 20; // Potential break
        }
    }

    score
}

fn evaluate_piece_protection(position: &Position, inverted: &Position) -> i32 {
    let mut score: i32 = 0;

    for piece in &position.white_pieces {
        if let Some(piece) = piece {
            let pos = piece.position;
            let attackers = inverted.count_attackers(pos.invert()) as i32;
            let defenders = position.count_attackers(pos) as i32;

            // Base the importance of protection on piece value
            let piece_importance: i32 = match piece.piece_type {
                PieceType::Pawn => 1,
                PieceType::Knight | PieceType::Bishop => 3,
                PieceType::Rook => 4,
                PieceType::Queen => 5,
                PieceType::King => 6,
            };

            // Higher bonus for pieces that are well protected vs attacked
            // Scale by piece importance
            if attackers < defenders {
                score += (attackers - defenders) * piece_importance * 5;
            } else if defenders < attackers {
                // Penalty for poorly protected pieces
                score -= (defenders - attackers) * piece_importance * 5;
            }

            // Additional evaluation for pieces under direct threat
            if attackers > 0 && piece_importance > 1 {
                score -= 10 * piece_importance;
            }
        }
    }

    score
}

fn evaluate_trapped_pieces(position: &Position) -> i32 {
    let mut score = 0;

    for piece in &position.white_pieces {
        if let Some(piece) = piece {
            // Skip pawns and king
            if matches!(piece.piece_type, PieceType::Pawn | PieceType::King) {
                continue;
            }

            let moves = piece.get_legal_moves(position, true);
            if moves.count() <= 2 {
                // Penalty based on piece value
                score -= piece_value(piece.piece_type) / 4;
            }
        }
    }
    score
}

fn evaluate_strategic_squares(position: &Position) -> i32 {
    let mut score = 0;

    // Define strategic squares (like e4, d4, e5, d5, f4, f5)
    let strategic_squares = Bitboard::center()
        | Bitboard::from_squares(&[
            Pos::xy(5, 3), // f4
            Pos::xy(5, 4), // f5
            Pos::xy(2, 3), // c4
            Pos::xy(2, 4), // c5
        ]);

    // Count control of strategic squares by different piece types
    for piece in &position.white_pieces {
        if let Some(piece) = piece {
            let control_map = piece.get_legal_moves(position, true);
            let strategic_control = (control_map & strategic_squares).count();

            // Higher bonus for permanent control (not just attacks)
            let control_bonus = match piece.piece_type {
                PieceType::Pawn => 35,   // Pawns provide permanent control
                PieceType::Knight => 25, // Knights are good outpost pieces
                PieceType::Bishop => 20,
                PieceType::Rook => 15,
                PieceType::Queen => 10, // Lower bonus as queen is often temporary
                PieceType::King => 5,
            };

            score += strategic_control as i32 * control_bonus;
        }
    }

    score
}

fn evaluate_piece_pressure(position: &Position, inverted: &Position) -> i32 {
    let mut score = 0;

    for piece in &inverted.white_pieces {
        // Evaluate pressure on black pieces
        if let Some(piece) = piece {
            let pos_from_white = piece.position.invert();
            let attackers = position.count_attackers(pos_from_white);
            let defenders = inverted.count_attackers(pos_from_white.invert());

            // Reward pressure even without capture possibility
            if attackers > 0 {
                let pressure_score = match piece.piece_type {
                    PieceType::Queen => 15, // Keeping queen restricted is valuable
                    PieceType::Rook => 12,
                    PieceType::Bishop | PieceType::Knight => 8,
                    PieceType::Pawn => 3,
                    PieceType::King => 5,
                };

                // More pressure if piece is poorly defended
                let defense_multiplier = if attackers > defenders { 2 } else { 1 };
                score += pressure_score * attackers as i32 * defense_multiplier;
            }

            // Bonus for restricting piece mobility
            let mobility = piece.get_legal_moves(inverted, true).count();
            if mobility < 4 {
                score += (4 - mobility as i32) * 10;
            }
        }
    }

    score
}

fn evaluate_pawn_defense_quality(position: &Position) -> i32 {
    let mut score = 0;
    let maps = position.get_piece_maps();

    for pawn_pos in maps.white_pawns.into_iter() {
        let defenders = position.count_attackers(pawn_pos);
        let pawn_defenders = count_pawn_defenders(position, pawn_pos);

        let mut queen_defending = false;
        for piece in position.white_pieces.iter() {
            if let Some(piece) = piece {
                let queen_moves = piece.get_legal_moves(position, true);
                if queen_moves.get(pawn_pos) {
                    queen_defending = true;
                    break;
                }
            }
        }

        let mut pawn_score = 0;

        // Base defense evaluation
        if defenders > 0 {
            if pawn_defenders == 0 {
                // Penalty for no pawn defenders
                pawn_score -= 15;

                // Extra penalty for queen being the only defender
                if queen_defending && defenders == 1 {
                    pawn_score -= 25;
                }
            } else {
                // Bonus for pawn defenders
                pawn_score += 10;
            }
        }

        // Adjust score based on pawn's position
        let importance_multiplier = get_pawn_importance(pawn_pos);
        score += pawn_score * importance_multiplier / 100;
    }

    score
}

fn count_pawn_defenders(position: &Position, target: Pos) -> i32 {
    let maps = position.get_piece_maps();
    (*pawn::attack_map_black(target) & maps.white_pawns).count() as i32
}

fn get_pawn_importance(pos: Pos) -> i32 {
    let file = pos.get_col();
    let rank = pos.get_row();

    // Higher multiplier for:
    // - Central files (d,e)
    // - Semi-central files (c,f)
    // - Advanced ranks
    // - Passed pawns
    // - Protected pawns
    let file_multiplier = match file {
        3 | 4 => 150, // d and e files
        2 | 5 => 120, // c and f files
        _ => 100,
    };

    // More important as pawns advance
    let rank_multiplier = 100 + (7 - rank as i32) * 10;

    (file_multiplier * rank_multiplier) / 100
}

/// Returns the manhattan distance between two positions
#[inline(always)]
fn manhattan_distance(a: Pos, b: Pos) -> u8 {
    let file_diff = a.get_col().abs_diff(b.get_col());
    let rank_diff = a.get_row().abs_diff(b.get_row());
    file_diff + rank_diff
}

/// Checks if the position is likely in the endgame based on material
fn is_endgame(position: &Position) -> bool {
    let maps = position.get_piece_maps();

    // Count major pieces (queens and rooks)
    let white_major = (maps.white_queens | maps.white_rooks).count();
    let black_major = (maps.black_queens | maps.black_rooks).count();

    // Count minor pieces (bishops and knights)
    let white_minor = (maps.white_bishops | maps.white_knights).count();
    let black_minor = (maps.black_bishops | maps.black_knights).count();

    // We're in endgame if:
    // 1. No queens or
    // 2. Only one queen per side and no other major pieces and <= 1 minor piece each
    maps.white_queens.count() + maps.black_queens.count() == 0
        || (maps.white_queens.count() <= 1
            && maps.black_queens.count() <= 1
            && white_major == maps.white_queens.count()
            && black_major == maps.black_queens.count()
            && white_minor <= 1
            && black_minor <= 1)
}

fn evaluate_mobility(position: &Position) -> i32 {
    let mut white_score = 0;

    let legal_moves = position.count_pseudolegal_moves();

    for (piece_type, move_count) in legal_moves {
        let mobility_bonus = mobility_bonus(piece_type) * move_count as i32;
        white_score += mobility_bonus;
    }

    white_score
}

fn evaluate_piece_coordination(position: &Position) -> i32 {
    let mut score = 0;

    // Track squares attacked by multiple pieces
    let mut attack_map = [[0i32; 8]; 8];

    // Count attacks on each square
    for piece in &position.white_pieces {
        if let Some(piece) = piece {
            let legal_moves = piece.get_legal_moves(position, true);
            for mv in legal_moves {
                let col = mv.get_col() as usize;
                let row = mv.get_row() as usize;
                attack_map[col][row] += 1;

                // Bonus for pieces protecting each other
                if let Some(defender) = position.get_piece_at(mv) {
                    if defender.color == piece.color {
                        score += 10;
                    }
                }
            }
        }
    }

    // Score squares attacked by multiple pieces
    for row in 0..8 {
        for col in 0..8 {
            if attack_map[col][row] >= 2 {
                // Higher bonus for central squares
                let center_dist = (3.5 - col as f32).abs() + (3.5 - row as f32).abs();
                let position_bonus = ((4.0 - center_dist) * 5.0) as i32;

                score += attack_map[col][row] * position_bonus;
            }
        }
    }

    score
}

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
        let position = Position::new(
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

        // Neither color should have a bishop pair
        assert!(!has_bishop_pair(&position, Color::White));
        assert!(!has_bishop_pair(&position, Color::Black));
    }

    #[test]
    fn test_has_bishop_pair_different_colored_squares() {
        // Create a position with bishops on different colored squares
        let position = Position::new(
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

        // White should have a bishop pair
        assert!(has_bishop_pair(&position, Color::White));
        // Black should not have any bishops
        assert!(!has_bishop_pair(&position, Color::Black));
    }

    #[test]
    fn test_has_bishop_pair_same_colored_squares() {
        // Create a position with bishops on same colored squares
        let position = Position::new(
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

        // White should not have a bishop pair (same colored squares)
        assert!(!has_bishop_pair(&position, Color::White));
        // Black should not have any bishops
        assert!(!has_bishop_pair(&position, Color::Black));
    }

    #[test]
    fn test_has_bishop_pair_mixed_pieces() {
        // Create a position with other pieces to ensure we only count bishops
        let position = Position::new(
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
        let position = Position::new(
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

        let white_score = evaluate_king_safety(&position);
        // No pawn shield means no bonus, but endgame adds a little
        assert_eq!(white_score, 10);
    }

    #[test]
    fn test_evaluate_king_safety_partial_shield() {
        // Position with partial pawn shield (2 pawns)
        let position = Position::new(
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

        let white_score = evaluate_king_safety(&position);
        // Two pawns in shield = 40 points
        assert_eq!(white_score, 40);
    }

    #[test]
    fn test_evaluate_king_safety_full_shield() {
        // Position with full pawn shield (3 pawns)
        let position = Position::new(
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

        let white_score = evaluate_king_safety(&position);
        // Three pawns in shield = 60 points
        assert_eq!(white_score, 70);
    }

    #[test]
    fn test_evaluate_king_safety_edge_case() {
        // Test king safety evaluation when king is on the edge of the board
        let position = Position::new(
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

        let white_score = evaluate_king_safety(&position);
        // Two pawns possible at edge = 40 points
        assert_eq!(white_score, 40);
    }

    #[test]
    fn test_evaluate_king_safety_no_king() {
        // Test position with no white king (edge case)
        let position = Position::new(
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
        let position = Position::new(
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

        let white_score = evaluate_mobility(&position);
        // Knight in center has 8 possible moves * 4 points = 32
        assert_eq!(white_score, 32);
    }

    #[test]
    fn test_evaluate_mobility_single_bishop() {
        // Position with single bishop in center
        let position = Position::new(
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

        let white_score = evaluate_mobility(&position);
        // Bishop in center has 13 possible moves * 3 points = 39
        assert_eq!(white_score, 39);
    }

    #[test]
    fn test_evaluate_mobility_single_rook() {
        // Position with single rook in center
        let position = Position::new(
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

        let white_score = evaluate_mobility(&position);
        // Rook in center has 14 possible moves * 2 points = 28
        assert_eq!(white_score, 28);
    }

    #[test]
    fn test_evaluate_mobility_single_queen() {
        // Position with single queen in center
        let position = Position::new(
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
        let position = Position::new(
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

        let white_score = evaluate_mobility(&position);
        // Knight: 8 moves * 4 points = 32
        // Bishop: 13 moves * 3 points = 39
        // Total = 71
        assert_eq!(white_score, 71);
    }

    #[test]
    fn test_evaluate_mobility_king_pawns() {
        // Test that kings and pawns don't contribute to mobility score
        let position = Position::new(
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

        let score = evaluate_position(&position, GameType::Classic, &Default::default());

        assert!(score < 20);
    }

    #[test]
    fn good_opening_pawn() {
        // Opening with e4, a strong move
        let position =
            Position::parse_from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1")
                .unwrap();

        let score = evaluate_position(&position, GameType::Classic, &Default::default());

        dbg!(score);
        assert!(score > 20);

        // Opening with d4, another strong move
        let position =
            Position::parse_from_fen("rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 1")
                .unwrap();

        let score = evaluate_position(&position, GameType::Classic, &Default::default());

        assert!(score > 20);
    }
}
