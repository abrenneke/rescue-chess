use std::sync::LazyLock;

use crate::{Bitboard, Pos};

static RESCUE_DROP_MAPS: LazyLock<[Bitboard; 64]> = LazyLock::new(|| {
    let mut maps = [Bitboard::new(); 64];

    for i in 0..64 {
        let mut board = Bitboard::new();
        let start_pos = Pos(i as u8);

        let mut pos = start_pos;
        if pos.can_move_down() {
            pos = pos.moved_unchecked(0, 1);
            board.set(pos);
        }

        pos = start_pos;
        if pos.can_move_up() {
            pos = pos.moved_unchecked(0, -1);
            board.set(pos);
        }

        pos = start_pos;
        if pos.can_move_left() {
            pos = pos.moved_unchecked(-1, 0);
            board.set(pos);
        }

        pos = start_pos;
        if pos.can_move_right() {
            pos = pos.moved_unchecked(1, 0);
            board.set(pos);
        }

        maps[i] = board;
    }

    maps
});

pub fn rescue_drop_map(pos: Pos) -> &'static Bitboard {
    &RESCUE_DROP_MAPS[pos.0 as usize]
}
