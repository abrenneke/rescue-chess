use crate::bitboard::Bitboard;
use crate::piece::occupancy::generate_occupancy_patterns;
use crate::pos::Pos;
use std::sync::LazyLock;

use super::occupancy::{generate_queen_move_table, generate_queen_occupancy_mask};

#[derive(Debug, Clone, PartialEq, Eq)]
struct MagicEntry {
    magic: u64,
    shift: u32,
    mask: Bitboard,
    moves: Vec<Bitboard>,
}

// The magic numbers you found, stored as (magic_number, shift) pairs
static QUEEN_MAGICS: [(u64, u32); 64] = [
    (0x8090408840100, 58),
    (0x20010101110009, 59),
    (0x41020091001200, 59),
    (0x4084040080000104, 59),
    (0x2004504000008006, 59),
    (0x200904420204000, 59),
    (0x200940c400000, 59),
    (0x840401012840, 58),
    (0x401002028100, 59),
    (0x1004280180a0, 59),
    (0x86081020010, 59),
    (0x202a40400808000, 59),
    (0x4000c2420000002, 59),
    (0x22000a0114200000, 59),
    (0x1010004402205000, 59),
    (0x14104012100, 59),
    (0x8008004002840404, 59),
    (0x44001810008a00, 59),
    (0x2080118020082, 57),
    (0x800029200c002, 57),
    (0x4002080a04000, 57),
    (0x2000408808021020, 57),
    (0x242008c100800, 59),
    (0x111000080480600, 59),
    (0x10100004a00202, 59),
    (0x81110088020800, 59),
    (0x201010040820201, 57),
    (0x41280004004010, 55),
    (0x22080801008a000, 55),
    (0x1810022012080, 57),
    (0x1888021041000, 59),
    (0x8008808000260800, 59),
    (0x408080490082000, 59),
    (0xa4020200200410, 59),
    (0x2020090080200a0, 57),
    (0x404008a0220200, 55),
    (0x209010400020202, 55),
    (0x450100280004040, 57),
    (0x4208080048008200, 59),
    (0x1021020020108, 59),
    (0x24041240400800, 59),
    (0x400c410000460, 59),
    (0x220104030000800, 57),
    (0x12004010400201, 57),
    (0x404091000a00, 57),
    (0x4500040c01200, 57),
    (0x20020601400a00, 59),
    (0x1220081002202, 59),
    (0x415010100804, 59),
    (0x401c10808020000, 59),
    (0x3010088040402, 59),
    (0x200008842020080, 59),
    (0x882002048102, 59),
    (0x220202410008000, 59),
    (0x40222282020000, 59),
    (0x120080100488400, 59),
    (0x450818020200, 58),
    (0x20010101101600, 59),
    (0xa1c4040400, 59),
    (0x102180400420200, 59),
    (0x40060042409, 59),
    (0x810100093, 59),
    (0x88400408008300, 59),
    (0x4080801012200, 58),
];

static MAGIC_TABLE: LazyLock<[MagicEntry; 64]> = LazyLock::new(|| {
    let mut entries = Vec::with_capacity(64);

    for sq in 0..64 {
        let pos = Pos(sq);
        let (magic, shift) = QUEEN_MAGICS[sq as usize];
        let mask = generate_queen_occupancy_mask(pos);
        let patterns = generate_occupancy_patterns(mask);
        let move_table = generate_queen_move_table(pos);

        // Initialize lookup table
        let table_size = 1 << (64 - shift);
        let mut moves = vec![Bitboard::new(); table_size];

        // Fill lookup table
        for (i, &pattern) in patterns.iter().enumerate() {
            let index = ((pattern.0.wrapping_mul(magic)) >> shift) as usize;
            moves[index] = move_table[i];
        }

        entries.push(MagicEntry {
            magic,
            shift,
            mask,
            moves,
        });
    }

    entries.try_into().unwrap()
});

/// Gets queen moves using magic lookup
pub fn get_queen_moves_magic(pos: Pos, occupied: Bitboard) -> Bitboard {
    let entry = &MAGIC_TABLE[pos.0 as usize];
    let relevant = occupied & entry.mask;
    let index = ((relevant.0.wrapping_mul(entry.magic)) >> entry.shift) as usize;
    entry.moves[index]
}
