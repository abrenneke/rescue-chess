use crate::Pos;

pub trait SquareBonus {
    fn square_bonus(pos: Pos) -> i32;
}

pub fn square_bonus<T: SquareBonus>(pos: Pos) -> i32 {
    T::square_bonus(pos)
}
