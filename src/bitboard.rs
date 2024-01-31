#[derive(PartialEq, Copy, Clone)]
pub struct Bitboard(pub u64);

impl Bitboard {
    #[inline(always)]
    pub fn get(&self, position: u8) -> bool {
        self.0 & (1 << position) != 0
    }

    #[inline(always)]
    pub fn set(&mut self, position: u8) {
        self.0 |= 1 << position;
    }

    #[inline(always)]
    pub fn with(mut self, position: u8) -> Self {
        self.0 &= !(1 << position);
        self
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
            board.push_str(if self.get(i) { "1" } else { "0" });
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
