use crate::bitboard::Bitboard;
use crate::piece::occupancy::generate_occupancy_patterns;
use crate::pos::Pos;

/// Generates a bitboard representing all squares that could potentially block
/// a bishop's movement from the given position. This excludes the edges of the
/// board since those squares can never affect which squares are reachable.
pub fn generate_bishop_occupancy_mask(pos: Pos) -> Bitboard {
    let mut mask = Bitboard::new();
    let (x, y) = (pos.get_col() as i32, pos.get_row() as i32);

    // For each diagonal direction
    for &(dx, dy) in &[(-1, -1), (-1, 1), (1, -1), (1, 1)] {
        let mut cx = x + dx;
        let mut cy = y + dy;

        // Move along diagonal until we're one square from the edge
        while cx > 0 && cx < 7 && cy > 0 && cy < 7 {
            mask.set(Pos::xy(cx as u8, cy as u8));
            cx += dx;
            cy += dy;
        }
    }

    mask
}

/// Generates the set of squares a bishop can move to, given an occupancy pattern
/// This is similar to your current bishop move generation, but simplified to just
/// handle blocking pieces (not distinguishing between friend/foe)
pub fn generate_bishop_moves(from: Pos, occupied: Bitboard) -> Bitboard {
    let mut moves = Bitboard::new();

    // Check all four diagonal directions
    for &(dx, dy) in &[(-1, -1), (-1, 1), (1, -1), (1, 1)] {
        let mut x = from.get_col() as i32 + dx;
        let mut y = from.get_row() as i32 + dy;

        while x >= 0 && x < 8 && y >= 0 && y < 8 {
            let current = Pos::xy(x as u8, y as u8);
            moves.set(current);

            // If we hit an occupied square, stop in this direction
            if occupied.get(current) {
                break;
            }

            x += dx;
            y += dy;
        }
    }

    moves
}

/// Generates a complete move table for a bishop at a given position
pub fn generate_bishop_move_table(pos: Pos) -> Vec<Bitboard> {
    let mask = generate_bishop_occupancy_mask(pos);
    let patterns = generate_occupancy_patterns(mask);

    // Generate moves for each occupancy pattern
    patterns
        .into_iter()
        .map(|occupied| generate_bishop_moves(pos, occupied))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_center_square() {
        // Test mask for a bishop on d4 (central square)
        let pos = Pos::from_algebraic("d4").unwrap();
        let mask = generate_bishop_occupancy_mask(pos);

        // The mask should include squares along the diagonals, excluding edges
        let expected: Bitboard = "
            . . . . . . . .
            . . . . . . x .
            . x . . . x . .
            . . x . x . . .
            . . . . . . . .
            . . x . x . . .
            . x . . . x . .
            . . . . . . . .
        "
        .parse()
        .unwrap();

        assert_eq!(mask, expected, "\nExpected:\n{}\nGot:\n{}", expected, mask);
    }

    #[test]
    fn test_corner_proximity() {
        // Test mask for a bishop on b2 (near corner)
        let pos = Pos::from_algebraic("b2").unwrap();
        let mask = generate_bishop_occupancy_mask(pos);

        // Should only include squares that could block paths
        let expected: Bitboard = "
            . . . . . . . .
            . . . . . . x .
            . . . . . x . .
            . . . . x . . .
            . . . x . . . .
            . . x . . . . .
            . . . . . . . .
            . . . . . . . .
        "
        .parse()
        .unwrap();

        assert_eq!(mask, expected, "\nExpected:\n{}\nGot:\n{}", expected, mask);
    }

    #[test]
    fn test_edge_square() {
        // Test mask for a bishop on h4 (edge square)
        let pos = Pos::from_algebraic("h4").unwrap();
        let mask = generate_bishop_occupancy_mask(pos);

        // Should only include squares that could block paths
        let expected: Bitboard = "
            . . . . . . . .
            . . . . x . . .
            . . . . . x . .
            . . . . . . x .
            . . . . . . . .
            . . . . . . x .
            . . . . . x . .
            . . . . . . . .
        "
        .parse()
        .unwrap();

        assert_eq!(mask, expected, "\nExpected:\n{}\nGot:\n{}", expected, mask);
    }

    #[test]
    fn test_empty_board_moves() {
        let pos = Pos::from_algebraic("d4").unwrap();
        let moves = generate_bishop_moves(pos, Bitboard::new());

        // On an empty board, bishop should attack all diagonal squares
        let expected: Bitboard = "
            . . . . . . . x
            x . . . . . x .
            . x . . . x . .
            . . x . x . . .
            . . . . . . . .
            . . x . x . . .
            . x . . . x . .
            x . . . . . x .
        "
        .parse()
        .unwrap();

        assert_eq!(
            moves, expected,
            "\nExpected:\n{}\nGot:\n{}",
            expected, moves
        );
    }

    #[test]
    fn test_single_blocker() {
        let pos = Pos::from_algebraic("d4").unwrap();
        let mut occupied = Bitboard::new();
        occupied.set(Pos::from_algebraic("f6").unwrap());

        let moves = generate_bishop_moves(pos, occupied);

        // Should include the blocking square but not beyond it
        let expected: Bitboard = "
            . . . . . . . .
            x . . . . . . .
            . x . . . x . .
            . . x . x . . .
            . . . . . . . .
            . . x . x . . .
            . x . . . x . .
            x . . . . . x .
        "
        .parse()
        .unwrap();

        assert_eq!(
            moves, expected,
            "\nExpected:\n{}\nGot:\n{}",
            expected, moves
        );
    }

    #[test]
    fn test_move_table_size() {
        let pos = Pos::from_algebraic("d4").unwrap();
        let move_table = generate_bishop_move_table(pos);

        // Center square has 9 blocking squares, so 2^9 = 512 patterns
        assert_eq!(move_table.len(), 512);

        // Each pattern should produce a unique set of moves
        let unique_moves: std::collections::HashSet<_> = move_table.into_iter().collect();
        assert!(
            unique_moves.len() > 20,
            "Expected more than 20 unique move patterns, got {}",
            unique_moves.len()
        );
    }

    #[test]
    fn test_edge_cases() {
        // Test corner position
        let pos = Pos::from_algebraic("h1").unwrap();
        let moves = generate_bishop_moves(pos, Bitboard::new());

        // Corner bishop should only attack one diagonal
        assert!(moves.get(Pos::from_algebraic("g2").unwrap()));
        assert!(moves.get(Pos::from_algebraic("f3").unwrap()));
        assert!(moves.get(Pos::from_algebraic("e4").unwrap()));

        // Test blocked on all sides
        let pos = Pos::from_algebraic("e4").unwrap();
        let mut occupied = Bitboard::new();
        for pos_str in ["d3", "d5", "f3", "f5"] {
            occupied.set(Pos::from_algebraic(pos_str).unwrap());
        }

        let moves = generate_bishop_moves(pos, occupied);
        assert_eq!(
            moves.count(),
            4,
            "Bishop blocked on all sides should only see 4 squares"
        );
    }
}
