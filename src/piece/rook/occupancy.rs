use crate::bitboard::Bitboard;
use crate::piece::occupancy::generate_occupancy_patterns;
use crate::pos::Pos;

/// Generates a bitboard representing all squares that could potentially block
/// a rook's movement from the given position. This excludes the edges of the
/// board since those squares can never affect which squares are reachable.
pub fn generate_rook_occupancy_mask(pos: Pos) -> Bitboard {
    let mut mask = Bitboard::new();
    let (x, y) = (pos.get_col() as i32, pos.get_row() as i32);

    // North
    if y < 6 {
        // Don't generate if too close to edge
        for cy in (y + 1)..7 {
            mask.set(Pos::xy(x as u8, cy as u8));
        }
    }

    // South
    if y > 1 {
        // Don't generate if too close to edge
        for cy in 1..(y) {
            mask.set(Pos::xy(x as u8, cy as u8));
        }
    }

    // East
    if x < 6 {
        // Don't generate if too close to edge
        for cx in (x + 1)..7 {
            mask.set(Pos::xy(cx as u8, y as u8));
        }
    }

    // West
    if x > 1 {
        // Don't generate if too close to edge
        for cx in 1..(x) {
            mask.set(Pos::xy(cx as u8, y as u8));
        }
    }

    mask
}

/// Generates the set of squares a rook can move to, given an occupancy pattern
pub fn generate_rook_moves(from: Pos, occupied: Bitboard) -> Bitboard {
    let mut moves = Bitboard::new();

    // Check all four orthogonal directions
    for &(dx, dy) in &[(0, -1), (0, 1), (-1, 0), (1, 0)] {
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

/// Generates a complete move table for a rook at a given position
pub fn generate_rook_move_table(pos: Pos) -> Vec<Bitboard> {
    let mask = generate_rook_occupancy_mask(pos);
    let patterns = generate_occupancy_patterns(mask);

    // Generate moves for each occupancy pattern
    patterns
        .into_iter()
        .map(|occupied| generate_rook_moves(pos, occupied))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rook_occupancy_masks() {
        // Test corner square (h8)
        let h8_mask = generate_rook_occupancy_mask(Pos::from_algebraic("h8").unwrap());
        let expected_h8: Bitboard = "
            . x x x x x x .
            . . . . . . . x
            . . . . . . . x
            . . . . . . . x
            . . . . . . . x
            . . . . . . . x
            . . . . . . . x
            . . . . . . . .
        "
        .parse()
        .unwrap();

        assert_eq!(
            h8_mask, expected_h8,
            "\nExpected h8 mask:\n{}\nGot:\n{}",
            expected_h8, h8_mask
        );

        // Test edge square (h4)
        let h4_mask = generate_rook_occupancy_mask(Pos::from_algebraic("h4").unwrap());
        let expected_h4: Bitboard = "
        . . . . . . . .
        . . . . . . . x
        . . . . . . . x
        . . . . . . . x
        . x x x x x x .
        . . . . . . . x
        . . . . . . . x
        . . . . . . . .
    "
        .parse()
        .unwrap();

        assert_eq!(
            h4_mask, expected_h4,
            "\nExpected h4 mask:\n{}\nGot:\n{}",
            expected_h4, h4_mask
        );

        // Test center square (d4)
        let d5_mask = generate_rook_occupancy_mask(Pos::from_algebraic("d5").unwrap());
        let expected_d5: Bitboard = "
        . . . . . . . .
        . . . x . . . .
        . . . x . . . .
        . x x . x x x .
        . . . x . . . .
        . . . x . . . .
        . . . x . . . .
        . . . . . . . .
    "
        .parse()
        .unwrap();

        assert_eq!(
            d5_mask, expected_d5,
            "\nExpected d5 mask:\n{}\nGot:\n{}",
            expected_d5, d5_mask
        );
    }
}
