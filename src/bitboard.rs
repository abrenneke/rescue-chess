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

    /// Iterates over all the positions that are occupied by a piece.
    pub fn iter(&self) -> impl Iterator<Item = Pos> + '_ {
        (0..64).filter_map(move |i| if self.get(Pos(i)) { Some(Pos(i)) } else { None })
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
}

/// Displays the bitboard as a string of 1s and 0s, with each row separated by a newline.
impl std::fmt::Display for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut board = String::new();
        for i in 0..64 {
            if i % 8 == 0 {
                board.push_str("\n");
            }
            board.push_str(if self.get(i.into()) { "1" } else { "0" });
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
                '1' => {
                    if position >= Pos(64) {
                        return Err(anyhow::anyhow!("Bitboard notation is too long"));
                    }

                    board.set(position);
                    position += 1;
                }
                '0' => {
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
