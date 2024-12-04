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
    pub fn count(&self) -> usize {
        self.0.count_ones() as usize
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
}

impl IntoIterator for Bitboard {
    type Item = Pos;
    type IntoIter = BitboardIter;

    fn into_iter(self) -> Self::IntoIter {
        BitboardIter {
            board: self,
            current: 0,
        }
    }
}

pub struct BitboardIter {
    board: Bitboard,
    current: u8,
}

impl Iterator for BitboardIter {
    type Item = Pos;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current < 64 {
            let pos = Pos(self.current);
            self.current += 1;
            if self.board.get(pos) {
                return Some(pos);
            }
        }
        None
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
}
