use crate::{piece::occupancy::generate_occupancy_patterns, Bitboard, Pos};

/// Generates a bitboard representing all squares that could potentially block
/// a queen's movement from the given position. This combines both the bishop and rook
/// occupancy patterns, incorporating their respective edge-handling rules.
pub fn generate_queen_occupancy_mask(pos: Pos) -> Bitboard {
    let mut mask = Bitboard::new();
    let (x, y) = (pos.get_col() as i32, pos.get_row() as i32);

    // Bishop-style diagonal movements
    for &(dx, dy) in &[(-1, -1), (-1, 1), (1, -1), (1, 1)] {
        let mut cx = x + dx;
        let mut cy = y + dy;

        while cx > 0 && cx < 7 && cy > 0 && cy < 7 {
            mask.set(Pos::xy(cx as u8, cy as u8));
            cx += dx;
            cy += dy;
        }
    }

    // North
    if y < 6 {
        for cy in (y + 1)..7 {
            mask.set(Pos::xy(x as u8, cy as u8));
        }
    }

    // South
    if y > 1 {
        for cy in 1..(y) {
            mask.set(Pos::xy(x as u8, cy as u8));
        }
    }

    // East
    if x < 6 {
        for cx in (x + 1)..7 {
            mask.set(Pos::xy(cx as u8, y as u8));
        }
    }

    // West
    if x > 1 {
        for cx in 1..(x) {
            mask.set(Pos::xy(cx as u8, y as u8));
        }
    }

    mask
}

/// Generates the set of squares a queen can move to, given an occupancy pattern.
/// This combines both diagonal (bishop-style) and orthogonal (rook-style) movements.
pub fn generate_queen_moves(from: Pos, occupied: Bitboard) -> Bitboard {
    let mut moves = Bitboard::new();
    let x = from.get_col() as i32;
    let y = from.get_row() as i32;

    // Check all eight directions (diagonals and orthogonals)

    for &(dx, dy) in &[
        (-1, -1),
        (-1, 0),
        (-1, 1), // SW, W, NW
        (0, -1),
        (0, 1), // S, N
        (1, -1),
        (1, 0),
        (1, 1), // SE, E, NE
    ] {
        let mut cx = x + dx;
        let mut cy = y + dy;

        while cx >= 0 && cx < 8 && cy >= 0 && cy < 8 {
            let current = Pos::xy(cx as u8, cy as u8);
            moves.set(current);

            // If we hit an occupied square, stop in this direction
            if occupied.get(current) {
                break;
            }

            cx += dx;
            cy += dy;
        }
    }

    moves
}

/// Generates a complete move table for a queen at a given position
pub fn generate_queen_move_table(pos: Pos) -> Vec<Bitboard> {
    let mask = generate_queen_occupancy_mask(pos);
    let patterns = generate_occupancy_patterns(mask);

    // Generate moves for each occupancy pattern
    patterns
        .into_iter()
        .map(|occupied| generate_queen_moves(pos, occupied))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queen_occupancy_masks() {
        // Test corner square (h8)
        let h8_mask = generate_queen_occupancy_mask(Pos::from_algebraic("h8").unwrap());
        let expected_h8: Bitboard = "
            . x x x x x x .
            . . . . . . x x
            . . . . . x . x
            . . . . x . . x
            . . . x . . . x
            . . x . . . . x
            . x . . . . . x
            . . . . . . . .
        "
        .parse()
        .unwrap();

        assert_eq!(
            h8_mask, expected_h8,
            "\nExpected h8 mask:\n{}\nGot:\n{}",
            expected_h8, h8_mask
        );

        // Test center square (d4)
        let d4_mask = generate_queen_occupancy_mask(Pos::from_algebraic("d4").unwrap());
        let expected_d4: Bitboard = "
            . . . . . . . .
            . . . x . . x .
            . x . x . x . .
            . . x x x . . .
            . x x . x x x .
            . . x x x . . .
            . x . x . x . .
            . . . . . . . .
        "
        .parse()
        .unwrap();

        assert_eq!(
            d4_mask, expected_d4,
            "\nExpected d4 mask:\n{}\nGot:\n{}",
            expected_d4, d4_mask
        );
    }

    #[test]
    fn test_queen_moves_empty_board() {
        // Test queen moves from center square (d4) on empty board
        let d4_moves = generate_queen_moves(Pos::from_algebraic("d4").unwrap(), Bitboard::new());
        let expected_d4: Bitboard = "
            . . . x . . . x
            x . . x . . x .
            . x . x . x . .
            . . x x x . . .
            x x x . x x x x
            . . x x x . . .
            . x . x . x . .
            x . . x . . x .
        "
        .parse()
        .unwrap();

        assert_eq!(
            d4_moves, expected_d4,
            "\nExpected d4 moves (empty board):\n{}\nGot:\n{}",
            expected_d4, d4_moves
        );

        // Test queen moves from corner (a1) on empty board
        let a1_moves = generate_queen_moves(Pos::from_algebraic("a1").unwrap(), Bitboard::new());
        let expected_a1: Bitboard = "
            x . . . . . . x
            x . . . . . x .
            x . . . . x . .
            x . . . x . . .
            x . . x . . . .
            x . x . . . . .
            x x . . . . . .
            . x x x x x x x
        "
        .parse()
        .unwrap();

        assert_eq!(
            a1_moves, expected_a1,
            "\nExpected a1 moves (empty board):\n{}\nGot:\n{}",
            expected_a1, a1_moves
        );
    }

    #[test]
    fn test_queen_moves_blocked() {
        // Test queen moves from e4 with blocking pieces
        let mut occupied = Bitboard::new();
        // Place blocking pieces at e6 (2 squares up), c4 (2 squares left),
        // and f5 (diagonal up-right)
        occupied.set(Pos::from_algebraic("e6").unwrap());
        occupied.set(Pos::from_algebraic("c4").unwrap());
        occupied.set(Pos::from_algebraic("f5").unwrap());

        let e4_moves = generate_queen_moves(Pos::from_algebraic("e4").unwrap(), occupied);
        let expected_e4: Bitboard = "
            x . . . . . . .
            . x . . . . . .
            . . x . x . . .
            . . . x x x . .
            . . x x . x x x
            . . . x x x . .
            . . x . x . x .
            . x . . x . . x
        "
        .parse()
        .unwrap();

        assert_eq!(
            e4_moves, expected_e4,
            "\nExpected e4 moves (blocked):\n{}\nGot:\n{}",
            expected_e4, e4_moves
        );
    }

    #[test]
    fn test_queen_move_table() {
        // Test move table generation for a corner square (h1)
        let h1_table = generate_queen_move_table(Pos::from_algebraic("h1").unwrap());

        // Verify the size of the move table
        // For h1, we should have 2^n patterns where n is the number of squares
        // in the occupancy mask for h1
        let h1_mask = generate_queen_occupancy_mask(Pos::from_algebraic("h1").unwrap());
        let expected_size = 1 << h1_mask.count();
        assert_eq!(
            h1_table.len(),
            expected_size,
            "Expected move table size {} for h1, got {}",
            expected_size,
            h1_table.len()
        );

        // Test specific cases from the move table
        // Case 1: Empty board (first entry in table)
        let empty_board_moves = h1_table[0];
        let expected_empty: Bitboard = "
            x . . . . . . x
            . x . . . . . x
            . . x . . . . x
            . . . x . . . x
            . . . . x . . x
            . . . . . x . x
            . . . . . . x x
            x x x x x x x .
        "
        .parse()
        .unwrap();

        assert_eq!(
            empty_board_moves, expected_empty,
            "\nExpected h1 moves (empty board):\n{}\nGot:\n{}",
            expected_empty, empty_board_moves
        );
    }

    #[test]
    fn test_queen_edge_cases() {
        // Test queen at edge positions
        let positions = [
            "a1", "h1", // corners
            "a8", "h8", // corners
            "d1", "d8", // edges
            "a4", "h4", // edges
        ];

        for pos_str in positions.iter() {
            let pos = Pos::from_algebraic(pos_str).unwrap();
            let moves = generate_queen_moves(pos, Bitboard::new());

            // Basic sanity checks for edge positions
            assert!(
                moves.get(pos),
                "Origin square {} should be included in moves",
                pos_str
            );
            assert!(
                moves.count() >= 14,
                "Queen at {} should have at least 14 moves on empty board",
                pos_str
            );

            // Verify moves don't go off board
            for row in 0..8 {
                for col in 0..8 {
                    if moves.get(Pos::xy(col, row)) {
                        assert!(
                            row < 8 && col < 8,
                            "Move at position ({}, {}) is off the board",
                            col,
                            row
                        );
                    }
                }
            }
        }
    }
}
