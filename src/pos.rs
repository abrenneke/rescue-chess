use std::{
    ops::{Deref, DerefMut},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use crate::bitboard::Bitboard;

#[derive(PartialEq, Copy, Clone, Default, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Pos(pub u8);

impl Pos {
    #[inline(always)]
    pub fn xy(x: u8, y: u8) -> Pos {
        Pos(x + y * 8)
    }

    #[inline(always)]
    pub fn get_xy(&self) -> (u8, u8) {
        (self.0 % 8, self.0 / 8)
    }

    pub fn from_algebraic(notation: &str) -> Result<Pos, anyhow::Error> {
        if notation.len() != 2 {
            return Err(anyhow::anyhow!(
                "Algebraic notation must be 2 characters long"
            ));
        }

        let file = notation.chars().nth(0).unwrap();
        let rank = notation.chars().nth(1).unwrap();

        let file = match file {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            _ => return Err(anyhow::anyhow!("Invalid file")),
        };

        // Our internal representation has rank 0 as the top of the board
        let rank = match rank {
            '1' => 7,
            '2' => 6,
            '3' => 5,
            '4' => 4,
            '5' => 3,
            '6' => 2,
            '7' => 1,
            '8' => 0,
            _ => return Err(anyhow::anyhow!("Invalid rank")),
        };

        Ok(Pos::xy(file, rank))
    }

    pub fn to_algebraic(&self) -> String {
        let (x, y) = self.get_xy();
        let file = match x {
            0 => "a",
            1 => "b",
            2 => "c",
            3 => "d",
            4 => "e",
            5 => "f",
            6 => "g",
            7 => "h",
            _ => unreachable!(),
        };

        // Our internal representation has rank 0 as the top of the board
        let rank = (7 - y + 1).to_string();
        format!("{}{}", file, rank)
    }

    #[inline]
    pub fn moved(&self, x: i8, y: i8) -> Option<Pos> {
        let cur_x = (self.0 as i8) % 8;
        let cur_y = (self.0 as i8) / 8;

        if cur_x + x < 0 || cur_x + x > 7 || cur_y + y < 0 || cur_y + y > 7 {
            None
        } else {
            Some(self.moved_unchecked(x, y))
        }
    }

    #[inline(always)]
    pub fn moved_unchecked(&self, x: i8, y: i8) -> Pos {
        Pos(((self.0 as i8) + x + y * 8) as u8)
    }

    #[inline(always)]
    pub fn moved_up_unchecked(&self) -> Pos {
        Pos(self.0 - 8)
    }

    #[inline(always)]
    pub fn moved_down_unchecked(&self) -> Pos {
        Pos(self.0 + 8)
    }

    #[inline(always)]
    pub fn moved_left_unchecked(&self) -> Pos {
        Pos(self.0 - 1)
    }

    #[inline(always)]
    pub fn moved_right_unchecked(&self) -> Pos {
        Pos(self.0 + 1)
    }

    pub const fn top_left() -> Pos {
        Pos(0)
    }

    pub const fn top_right() -> Pos {
        Pos(7)
    }

    pub const fn bottom_left() -> Pos {
        Pos(56)
    }

    pub const fn bottom_right() -> Pos {
        Pos(63)
    }

    #[inline(always)]
    pub fn is_col(&self, col: u8) -> bool {
        self.0 % 8 == col
    }

    #[inline(always)]
    pub fn is_row(&self, row: u8) -> bool {
        self.0 / 8 == row
    }

    #[inline(always)]
    pub fn get_col(&self) -> u8 {
        self.0 % 8
    }

    #[inline(always)]
    pub fn get_row(&self) -> u8 {
        self.0 / 8
    }

    #[inline(always)]
    pub fn can_move_up(&self) -> bool {
        self.0 > 7
    }

    #[inline(always)]
    pub fn can_move_down(&self) -> bool {
        self.0 < 56
    }

    #[inline(always)]
    pub fn can_move_left(&self) -> bool {
        self.0 % 8 != 0
    }

    #[inline(always)]
    pub fn can_move_right(&self) -> bool {
        self.0 % 8 != 7
    }

    #[inline(always)]
    pub fn invert(&self) -> Pos {
        let x = self.0 % 8;
        let y = self.0 / 8;

        Pos::xy(7 - x, 7 - y)
    }

    #[inline(always)]
    pub fn as_tuple(&self) -> (u8, u8) {
        (self.0 % 8, self.0 / 8)
    }

    pub fn get_cardinal_adjacent(&self) -> [Option<Pos>; 4] {
        let mut adjacent: [Option<Pos>; 4] = [None; 4];

        let (x, y) = self.as_tuple();

        if x > 0 {
            adjacent[0] = Some(Pos::xy(x - 1, y));
        }

        if x < 7 {
            adjacent[1] = Some(Pos::xy(x + 1, y));
        }

        if y > 0 {
            adjacent[2] = Some(Pos::xy(x, y - 1));
        }

        if y < 7 {
            adjacent[3] = Some(Pos::xy(x, y + 1));
        }

        adjacent
    }
}

pub const A8: Pos = Pos(0);
pub const A1: Pos = Pos(56);

pub const B8: Pos = Pos(1);
pub const B1: Pos = Pos(57);

pub const C8: Pos = Pos(2);
pub const C1: Pos = Pos(58);

pub const D8: Pos = Pos(3);
pub const D1: Pos = Pos(59);

pub const E8: Pos = Pos(4);
pub const E1: Pos = Pos(60);

pub const F8: Pos = Pos(5);
pub const F1: Pos = Pos(61);

pub const G8: Pos = Pos(6);
pub const G1: Pos = Pos(62);

pub const H8: Pos = Pos(7);
pub const H1: Pos = Pos(63);

impl Deref for Pos {
    type Target = u8;

    #[inline(always)]
    fn deref(&self) -> &u8 {
        &self.0
    }
}

impl DerefMut for Pos {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut u8 {
        &mut self.0
    }
}

impl From<u8> for Pos {
    #[inline(always)]
    fn from(position: u8) -> Pos {
        Pos(position)
    }
}

impl From<(u8, u8)> for Pos {
    #[inline(always)]
    fn from((x, y): (u8, u8)) -> Pos {
        Pos::xy(x, y)
    }
}

impl From<Pos> for u8 {
    #[inline(always)]
    fn from(pos: Pos) -> u8 {
        pos.0
    }
}

impl std::ops::Add<u8> for Pos {
    type Output = Pos;

    #[inline(always)]
    fn add(self, rhs: u8) -> Pos {
        Pos(self.0 + rhs)
    }
}

impl std::ops::AddAssign<u8> for Pos {
    #[inline(always)]
    fn add_assign(&mut self, rhs: u8) {
        self.0 += rhs;
    }
}

impl std::ops::Sub<u8> for Pos {
    type Output = Pos;

    #[inline(always)]
    fn sub(self, rhs: u8) -> Pos {
        Pos(self.0 - rhs)
    }
}

impl std::ops::SubAssign<u8> for Pos {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: u8) {
        self.0 -= rhs;
    }
}

impl std::ops::Add<Pos> for Pos {
    type Output = Pos;

    #[inline(always)]
    fn add(self, rhs: Pos) -> Pos {
        Pos(self.0 + rhs.0)
    }
}

impl std::ops::AddAssign<Pos> for Pos {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Pos) {
        self.0 += rhs.0;
    }
}

impl std::ops::Sub<Pos> for Pos {
    type Output = Pos;

    #[inline(always)]
    fn sub(self, rhs: Pos) -> Pos {
        Pos(self.0 - rhs.0)
    }
}

impl std::ops::SubAssign<Pos> for Pos {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Pos) {
        self.0 -= rhs.0;
    }
}

impl std::ops::Rem<u8> for Pos {
    type Output = Pos;

    #[inline(always)]
    fn rem(self, rhs: u8) -> Pos {
        Pos(self.0 % rhs)
    }
}

impl std::ops::RemAssign<u8> for Pos {
    #[inline(always)]
    fn rem_assign(&mut self, rhs: u8) {
        self.0 %= rhs;
    }
}

impl std::fmt::Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}, {})", self.0 % 8, self.0 / 8)
    }
}

impl std::fmt::Debug for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}, {})", self.0 % 8, self.0 / 8)
    }
}

impl From<Pos> for Bitboard {
    fn from(pos: Pos) -> Bitboard {
        Bitboard::new().with(pos)
    }
}

impl From<&'static str> for Pos {
    fn from(s: &'static str) -> Pos {
        Pos::from_algebraic(s).unwrap()
    }
}

impl FromStr for Pos {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Pos::from_algebraic(s)
    }
}
