use crate::pos::Pos;

#[derive(PartialEq, Copy, Clone, Eq, Hash)]
pub struct Bitboard(pub u64);

impl Bitboard {
    #[inline(always)]
    pub fn get(&self, position: Pos) -> bool {
        self.0 & (1 << position.0) != 0
    }

    #[inline(always)]
    pub fn set(&mut self, position: Pos) {
        self.0 |= 1 << position.0;
    }

    #[inline(always)]
    pub fn with(mut self, position: Pos) -> Self {
        self.0 |= 1 << position.0;
        self
    }

    pub fn iter(&self) -> impl Iterator<Item = Pos> + '_ {
        (0..64).filter_map(move |i| if self.get(Pos(i)) { Some(Pos(i)) } else { None })
    }

    #[inline(always)]
    pub fn new() -> Self {
        Bitboard(0)
    }
}

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

impl std::fmt::Debug for Bitboard {
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

impl std::default::Default for Bitboard {
    fn default() -> Self {
        Bitboard(0)
    }
}

impl std::ops::BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 & rhs.0)
    }
}

impl std::ops::BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 ^ rhs.0)
    }
}

impl std::ops::Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Bitboard(!self.0)
    }
}

impl std::convert::From<u64> for Bitboard {
    fn from(value: u64) -> Self {
        Bitboard(value)
    }
}

impl std::convert::From<Bitboard> for u64 {
    fn from(value: Bitboard) -> Self {
        value.0
    }
}

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
