use crate::bitboard::Bitboard;

/// Generates all possible occupancy patterns for the blocking squares defined by a mask.
/// Each bit in the mask can be either occupied or empty, giving us 2^n patterns where
/// n is the number of bits set in the mask.
pub fn generate_occupancy_patterns(mask: Bitboard) -> Vec<Bitboard> {
    let bit_count = mask.count();
    let pattern_count = 1 << bit_count;
    let mut patterns = Vec::with_capacity(pattern_count);

    // For each possible pattern (0 to 2^n - 1)
    for index in 0..pattern_count {
        let mut occupancy = Bitboard::new();
        let mut mask_copy = mask;
        let mut bit_index = 0;

        // While we still have squares in our mask
        while mask_copy.0 != 0 {
            // Get least significant bit position
            let ls1b = mask_copy.0 & mask_copy.0.wrapping_neg();

            // If the corresponding bit is set in our index pattern
            if (index & (1 << bit_index)) != 0 {
                occupancy.0 |= ls1b;
            }

            // Clear the least significant bit
            mask_copy.0 &= mask_copy.0 - 1;
            bit_index += 1;
        }

        patterns.push(occupancy);
    }

    patterns
}

#[cfg(test)]
mod tests {
    use crate::{piece::bishop::occupancy::generate_bishop_occupancy_mask, Pos};

    use super::*;

    #[test]
    fn test_single_bit_mask() {
        // Create a mask with just one bit set
        let mut mask = Bitboard::new();
        mask.set(Pos::from_algebraic("d4").unwrap());

        let patterns = generate_occupancy_patterns(mask);

        // Should generate exactly two patterns: empty and occupied
        assert_eq!(patterns.len(), 2);
        assert_eq!(patterns[0], Bitboard::new()); // Empty pattern
        assert_eq!(patterns[1], mask); // Fully occupied pattern
    }

    #[test]
    fn test_two_bit_mask() {
        // Create a mask with two bits set
        let mut mask = Bitboard::new();
        mask.set(Pos::from_algebraic("d4").unwrap());
        mask.set(Pos::from_algebraic("e5").unwrap());

        let patterns = generate_occupancy_patterns(mask);

        // Should generate exactly four patterns
        assert_eq!(patterns.len(), 4);

        // Verify all patterns are unique
        let unique_patterns: std::collections::HashSet<_> = patterns.into_iter().collect();
        assert_eq!(unique_patterns.len(), 4);
    }

    #[test]
    fn test_bishop_d4_patterns() {
        let pos = Pos::from_algebraic("d4").unwrap();
        let mask = generate_bishop_occupancy_mask(pos);
        let patterns = generate_occupancy_patterns(mask);

        // For a bishop on d4, mask has 9 bits set, so we expect 2^9 = 512 patterns
        assert_eq!(patterns.len(), 512);

        // First pattern should be empty
        assert_eq!(patterns[0], Bitboard::new());

        // Last pattern should have all mask bits set
        assert_eq!(patterns[511], mask);

        // Verify all patterns are subsets of the mask
        for pattern in &patterns {
            assert_eq!(
                *pattern & mask,
                *pattern,
                "Pattern contained bits outside the mask:\nPattern:\n{}\nMask:\n{}",
                pattern,
                mask
            );
        }

        // Verify all patterns are unique
        let unique_patterns: std::collections::HashSet<_> = patterns.into_iter().collect();
        assert_eq!(unique_patterns.len(), 512);
    }

    #[test]
    fn test_edge_square_patterns() {
        let pos = Pos::from_algebraic("h4").unwrap();
        let mask = generate_bishop_occupancy_mask(pos);

        let patterns = generate_occupancy_patterns(mask);

        // Edge square bishop has 5 blocking squares, so 2^5 = 32 patterns
        assert_eq!(patterns.len(), 32);

        // Verify all patterns are unique and valid
        let unique_patterns: std::collections::HashSet<_> = patterns.clone().into_iter().collect();
        assert_eq!(unique_patterns.len(), 32);

        for pattern in &patterns {
            assert_eq!(*pattern & mask, *pattern);
        }
    }
}
