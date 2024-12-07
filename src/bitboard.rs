use crate::pos::Pos;

/// An integer that represents every position on the chess board, with a
/// 1 representing a piece and a 0 representing an empty square.
#[derive(PartialEq, Copy, Clone, Eq, Hash)]
pub struct Bitboard(pub u64);

impl Bitboard {
    /// Gets whether the specified position is occupied by a piece.
    #[inline(always)]
    pub fn get(&self, position: Pos) -> bool {
        self.0 & (1 << position.0) != 0
    }

    /// Sets the specified position to be occupied by a piece.
    #[inline(always)]
    pub fn set(&mut self, position: Pos) {
        self.0 |= 1 << position.0;
    }

    /// Sets the specified position to be occupied by a piece and returns. For use in method chaining.
    #[inline(always)]
    pub fn with(mut self, position: Pos) -> Self {
        self.0 |= 1 << position.0;
        self
    }

    #[inline(always)]
    pub fn clear(&mut self, position: Pos) {
        self.0 &= !(1 << position.0);
    }

    /// Creates a new empty bitboard. All positions are set to 0.
    #[inline(always)]
    pub fn new() -> Self {
        Bitboard(0)
    }

    #[inline(always)]
    pub fn count(&self) -> u8 {
        self.0.count_ones() as u8
    }

    #[inline(always)]
    pub fn invert(self) -> Self {
        // To rotate 180 degrees:
        // 1. Flip vertically by reversing each byte
        // 2. Flip horizontally by reversing the byte order

        let mut result = 0u64;
        let mut value = self.0;

        // Process each byte (8 rows of the board)
        for _ in 0..8 {
            // Get current byte (row)
            let byte = (value & 0xFF) as u8;
            // Reverse the bits in the byte using reverse_bits()
            let reversed = byte.reverse_bits();
            // Add to result and shift
            result = (result << 8) | (reversed as u64);
            // Move to next byte
            value >>= 8;
        }

        Bitboard(result)
    }

    #[inline(always)]
    pub fn for_rank(rank: u8) -> Self {
        debug_assert!(rank < 8);
        Bitboard(0xFF << (rank * 8))
    }

    /// Returns a bitboard with all bits set in the given file (0-7)
    #[inline(always)]
    pub fn for_file(file: u8) -> Self {
        debug_assert!(file < 8);
        Bitboard(0x0101010101010101u64 << file)
    }

    /// Returns a bitboard representing squares ahead of the given rank for white
    #[inline(always)]
    pub fn ahead_of_rank_black(rank: u8) -> Self {
        debug_assert!(rank < 8);
        Bitboard(!((1u64 << ((rank + 1) * 8)) - 1))
    }

    /// Returns a bitboard representing squares ahead of the given rank for black
    #[inline(always)]
    pub fn ahead_of_rank_white(rank: u8) -> Self {
        debug_assert!(rank < 8);
        Bitboard((1u64 << (rank * 8)) - 1)
    }

    /// Returns a bitboard representing the central four squares (d4, e4, d5, e5)
    #[inline(always)]
    pub fn center() -> Self {
        Bitboard(0x0000001818000000u64)
    }

    /// Returns a bitboard representing the extended center (16 squares)
    #[inline(always)]
    pub fn extended_center() -> Self {
        Bitboard(0x00003C3C3C3C0000u64)
    }

    /// Returns a bitboard of all light squares
    #[inline(always)]
    pub fn light_squares() -> Self {
        Bitboard(0x55AA55AA55AA55AAu64)
    }

    /// Returns a bitboard of all dark squares
    #[inline(always)]
    pub fn dark_squares() -> Self {
        !Self::light_squares()
    }

    /// Returns a bitboard representing adjacent files
    #[inline(always)]
    pub fn adjacent_files(file: u8) -> Self {
        debug_assert!(file < 8);
        let mut mask = 0u64;
        if file > 0 {
            mask |= Self::for_file(file - 1).0;
        }
        if file < 7 {
            mask |= Self::for_file(file + 1).0;
        }
        Bitboard(mask)
    }

    /// Returns whether this bitboard intersects with another
    #[inline(always)]
    pub fn intersects(&self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    /// Returns whether this bitboard contains all bits from another
    #[inline(always)]
    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline(always)]
    pub fn from_squares(squares: &[Pos]) -> Self {
        let mut board = Bitboard::new();
        for &pos in squares {
            board.set(pos);
        }
        board
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SumBitboards(pub [i32; 64]);

impl SumBitboards {
    pub fn new() -> Self {
        SumBitboards([0; 64])
    }

    pub fn add(&mut self, bitboard: Bitboard) {
        for pos in bitboard.into_iter() {
            self.0[pos.0 as usize] += 1;
        }
    }

    pub fn get(&self, pos: Pos) -> i32 {
        self.0[pos.0 as usize]
    }

    pub fn subtract(&mut self, bitboard: Bitboard) {
        for pos in bitboard.into_iter() {
            self.0[pos.0 as usize] -= 1;
        }
    }
}

pub struct BitboardIter {
    remaining: u64,
}

impl Iterator for BitboardIter {
    type Item = Pos;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            None
        } else {
            // Get position of least significant set bit
            let pos = self.remaining.trailing_zeros() as u8;
            // Clear the least significant set bit
            self.remaining &= self.remaining - 1;
            Some(Pos(pos))
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.remaining.count_ones() as usize;
        (count, Some(count))
    }
}

impl IntoIterator for Bitboard {
    type Item = Pos;
    type IntoIter = BitboardIter;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        BitboardIter { remaining: self.0 }
    }
}

/// Displays the bitboard as a string of 1s and 0s, with each row separated by a newline.
impl std::fmt::Display for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut board = String::new();
        for i in 0..64 {
            if i % 8 == 0 {
                board.push_str("\n");
            }
            board.push_str(if self.get(i.into()) { "x " } else { ". " });
        }
        write!(f, "{}", board)
    }
}

/// Displays the bitboard as a string of 1s and 0s, with each row separated by a newline.
impl std::fmt::Debug for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

/// Creates a default empty bitboard.
impl std::default::Default for Bitboard {
    fn default() -> Self {
        Bitboard(0)
    }
}

/// Implementation for bitwise OR.
impl std::ops::BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 | rhs.0)
    }
}

/// Implementation for bitwise AND.
impl std::ops::BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 & rhs.0)
    }
}

/// Implementation for bitwise XOR.
impl std::ops::BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 ^ rhs.0)
    }
}

/// Implementation for bitwise NOT.
impl std::ops::Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Bitboard(!self.0)
    }
}

/// Converts from a u64 to a bitboard.
impl std::convert::From<u64> for Bitboard {
    fn from(value: u64) -> Self {
        Bitboard(value)
    }
}

/// Converts from a bitboard to a u64.
impl std::convert::From<Bitboard> for u64 {
    fn from(value: Bitboard) -> Self {
        value.0
    }
}

/// Parses a bitboard from a string of 1s and 0s, with each row separated by a newline.
/// The string may contain spaces, which will be ignored.
///
/// # Example
///
/// ```
/// use rescue_chess::{Bitboard, Pos};
///
/// let board = Bitboard::from_str(r#"
///     10000000
///     01000000
///     00100000
///     00010000
///     00001000
///     00000100
///     00000010
///     00000001
/// "#).unwrap();
impl std::str::FromStr for Bitboard {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut board = Bitboard::new();
        let mut position = Pos(0);
        for character in s.chars() {
            match character {
                '1' | 'x' => {
                    if position >= Pos(64) {
                        return Err(anyhow::anyhow!("Bitboard notation is too long"));
                    }

                    board.set(position);
                    position += 1;
                }
                '0' | '.' => {
                    if position >= Pos(64) {
                        return Err(anyhow::anyhow!("Bitboard notation is too long"));
                    }

                    position += 1;
                }
                '\n' => {}
                ' ' => {}
                _ => {
                    return Err(anyhow::anyhow!("Invalid character in bitboard notation"));
                }
            }
        }
        Ok(board)
    }
}
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_bitboard_invert() {
        // Test Case 1: Single piece in top-left corner
        let board = Bitboard::from_str(
            r#"
            10000000
            00000000
            00000000
            00000000
            00000000
            00000000
            00000000
            00000000
        "#,
        )
        .unwrap();

        let expected = Bitboard::from_str(
            r#"
            00000000
            00000000
            00000000
            00000000
            00000000
            00000000
            00000000
            00000001
        "#,
        )
        .unwrap();

        assert_eq!(
            board.invert(),
            expected,
            "Failed to invert single piece in corner"
        );

        // Test Case 2: Diagonal pattern
        let board = Bitboard::from_str(
            r#"
            10000000
            01000000
            00100000
            00010000
            00001000
            00000100
            00000010
            00000001
        "#,
        )
        .unwrap();

        let expected = Bitboard::from_str(
            r#"
            10000000
            01000000
            00100000
            00010000
            00001000
            00000100
            00000010
            00000001
        "#,
        )
        .unwrap();

        assert_eq!(
            board.invert(),
            expected,
            "Failed to invert diagonal pattern"
        );

        // Test Case 3: Complex pattern
        let board = Bitboard::from_str(
            r#"
            11000011
            10000001
            00000000
            00111100
            00111100
            00000000
            10000001
            11000011
        "#,
        )
        .unwrap();

        assert_eq!(board.invert(), board, "Failed to invert symmetric pattern");

        // Test Case 4: Empty board
        let empty = Bitboard::new();
        assert_eq!(empty.invert(), empty, "Failed to invert empty board");

        // Test Case 5: Full board
        let full = !Bitboard::new();
        assert_eq!(full.invert(), full, "Failed to invert full board");
    }

    #[test]
    fn test_double_invert() {
        // Property test: inverting twice should return to original state
        let board = Bitboard::from_str(
            r#"
            10101010
            01010101
            11001100
            00110011
            10101010
            01010101
            11001100
            00110011
        "#,
        )
        .unwrap();

        assert_eq!(
            board.invert().invert(),
            board,
            "Double invert should return original board"
        );
    }

    #[test]
    fn test_invert_individual_bits() {
        // Test each bit position individually
        for pos in 0..64 {
            let mut board = Bitboard::new();
            board.set(Pos(pos));

            let inverted = board.invert();
            let expected_pos = 63 - pos;

            assert!(
                inverted.get(Pos(expected_pos)),
                "Failed to correctly invert bit at position {}",
                pos
            );
            assert_eq!(
                inverted.count(),
                1,
                "Inverted board should still have exactly one bit set"
            );
        }
    }

    #[test]
    fn files() {
        println!("{}", Bitboard::from(0x0101010101010101 << 1));
    }

    #[test]
    fn for_rank() {
        println!("{}", Bitboard::for_rank(3));
    }

    #[test]
    fn for_file() {
        println!("{}", Bitboard::for_file(3));
    }

    #[test]
    fn ahead_of_rank_white() {
        println!("{}", Bitboard::ahead_of_rank_white(3));
    }

    #[test]
    fn ahead_of_rank_black() {
        println!("{}", Bitboard::ahead_of_rank_black(3));
    }

    #[test]
    fn center() {
        println!("{}", Bitboard::center());
    }

    #[test]
    fn extended_center() {
        println!("{}", Bitboard::extended_center());
    }

    #[test]
    fn light_squares() {
        println!("{}", Bitboard::light_squares());
    }

    #[test]
    fn dark_squares() {
        println!("{}", Bitboard::dark_squares());
    }

    #[test]
    fn adjacent_files() {
        println!("{}", Bitboard::adjacent_files(3));
    }
}
