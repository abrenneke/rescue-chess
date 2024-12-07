use crate::bitboard::Bitboard;
use crate::piece::occupancy::generate_occupancy_patterns;
use crate::pos::Pos;
use std::sync::LazyLock;

use super::occupancy::{generate_rook_move_table, generate_rook_occupancy_mask};

#[derive(Debug, Clone, PartialEq, Eq)]
struct MagicEntry {
    magic: u64,
    shift: u32,
    mask: Bitboard,
    moves: Vec<Bitboard>,
}

// Pre-computed magic numbers for rooks. Each entry is (magic_number, shift)
static ROOK_MAGICS: [(u64, u32); 64] = [
    (0x8000400a102081, 52),
    (0x80210014408080, 52),
    (0x2040082000431000, 52),
    (0xc040024004010800, 52),
    (0x8100102100020804, 52),
    (0x1004040084402400, 52),
    (0x280008000520300, 52),
    (0x880008000244100, 52),
    (0xa00800010208440, 52),
    (0x20101000208005, 51),
    (0xa0a0080400a0, 51),
    (0x840012808801, 51),
    (0x402c0008040200, 51),
    (0x4020082080090008, 51),
    (0x804800450002100, 51),
    (0x1080401088002040, 52),
    (0x800080800410c008, 52),
    (0x480804800a2000, 51),
    (0x20220004000c6000, 50),
    (0x4800050090000100, 50),
    (0x400048808041010, 50),
    (0x8001408080011400, 50),
    (0x10c4020404010, 51),
    (0x4110014800800040, 52),
    (0x80510008402000, 52),
    (0x4c207000800, 51),
    (0x2001000406022100, 50),
    (0x808200200008105, 50),
    (0x810004821020, 50),
    (0x80201102a0004, 50),
    (0x8002000060008010, 51),
    (0x22200088010440, 52),
    (0x800200010400940, 52),
    (0x210400080240004, 51),
    (0x1000022002218001, 50),
    (0x10000a10040284, 50),
    (0x400440800400091, 50),
    (0x2000830000210100, 50),
    (0x40920400260, 51),
    (0x1000800405300100, 52),
    (0x40208010800821, 52),
    (0x8042208114000, 51),
    (0x20200844201200, 50),
    (0x1421000010007, 50),
    (0x200800c004012400, 50),
    (0x2100400890042, 50),
    (0x8000020008c08005, 51),
    (0x2884520001, 52),
    (0x400210013800100, 52),
    (0x400402000040806, 51),
    (0x400024008180022, 51),
    (0x3400a200108, 51),
    (0x1000a88004000280, 51),
    (0x4000401008a00c0, 51),
    (0x20008010004c102, 51),
    (0x4004800100006050, 52),
    (0x2102040820102, 52),
    (0x40008009204011, 52),
    (0x800200008104045, 52),
    (0x1100100008a005, 52),
    (0x8022000104200a, 52),
    (0x120004110282, 52),
    (0x148802100044, 52),
    (0x1000030420083, 52),
];

static MAGIC_TABLE: LazyLock<[MagicEntry; 64]> = LazyLock::new(|| {
    let mut entries = Vec::with_capacity(64);

    for sq in 0..64 {
        let pos = Pos(sq);
        let (magic, shift) = ROOK_MAGICS[sq as usize];
        let mask = generate_rook_occupancy_mask(pos);
        let patterns = generate_occupancy_patterns(mask);
        let move_table = generate_rook_move_table(pos);

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

/// Gets rook moves using magic lookup
#[inline]
pub fn get_rook_moves_magic(pos: Pos, occupied: Bitboard) -> Bitboard {
    let entry = &MAGIC_TABLE[pos.0 as usize];
    let relevant = occupied & entry.mask;
    let index = ((relevant.0.wrapping_mul(entry.magic)) >> entry.shift) as usize;
    entry.moves[index]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::piece::rook::occupancy::generate_rook_moves;
    use rand::Rng;
    use std::time::Instant;

    #[test]
    fn test_magic_lookup_empty_board() {
        let pos = Pos::from_algebraic("d4").unwrap();
        let moves = get_rook_moves_magic(pos, Bitboard::new());

        // On empty board, rook should attack all ranks and files
        let expected: Bitboard = "
            . . . x . . . .
            . . . x . . . .
            . . . x . . . .
            . . . x . . . .
            x x x . x x x x
            . . . x . . . .
            . . . x . . . .
            . . . x . . . .
        "
        .parse()
        .unwrap();

        assert_eq!(
            moves, expected,
            "\nExpected:\n{}\nGot:\n{}",
            expected, moves
        );
    }

    #[test]
    fn test_magic_lookup_blocked() {
        let pos = Pos::from_algebraic("d4").unwrap();

        // Place blocking pieces
        let mut occupied = Bitboard::new();
        occupied.set(Pos::from_algebraic("d6").unwrap()); // Block up
        occupied.set(Pos::from_algebraic("b4").unwrap()); // Block left

        let moves = get_rook_moves_magic(pos, occupied);

        // Should include blocking squares but not beyond
        let expected: Bitboard = "
            . . . . . . . .
            . . . . . . . .
            . . . x . . . .
            . . . x . . . .
            . x x . x x x x
            . . . x . . . .
            . . . x . . . .
            . . . x . . . .
        "
        .parse()
        .unwrap();

        assert_eq!(
            moves, expected,
            "\nExpected:\n{}\nGot:\n{}",
            expected, moves
        );
    }

    #[test]
    fn test_magic_lookup_edge_cases() {
        // Test corner rook
        let pos = Pos::from_algebraic("h1").unwrap();
        let moves = get_rook_moves_magic(pos, Bitboard::new());

        assert!(moves.get(Pos::from_algebraic("h2").unwrap()));
        assert!(moves.get(Pos::from_algebraic("g1").unwrap()));
        assert!(!moves.get(Pos::from_algebraic("g2").unwrap()));

        // Test fully surrounded rook
        let pos = Pos::from_algebraic("e4").unwrap();
        let mut occupied = Bitboard::new();
        for p in ["e3", "e5", "d4", "f4"] {
            occupied.set(Pos::from_algebraic(p).unwrap());
        }

        let moves = get_rook_moves_magic(pos, occupied);
        assert_eq!(
            moves.count(),
            4,
            "Surrounded rook should see exactly 4 squares"
        );
    }

    #[test]
    fn test_magic_lookup_matches_slow() {
        // Test that magic lookup matches the slower generate_rook_moves
        // for various positions and occupancy patterns
        let test_positions = ["d4", "h1", "a8", "e4"];
        let test_blockers = [
            vec![],
            vec!["d3"],
            vec!["d6", "b4"],
            vec!["e3", "e5", "d4", "f4"],
        ];

        for pos_str in test_positions {
            let pos = Pos::from_algebraic(pos_str).unwrap();

            for blockers in &test_blockers {
                let mut occupied = Bitboard::new();
                for &blocker in blockers {
                    if let Ok(p) = Pos::from_algebraic(blocker) {
                        occupied.set(p);
                    }
                }

                let magic_moves = get_rook_moves_magic(pos, occupied);
                let slow_moves = generate_rook_moves(pos, occupied);

                assert_eq!(
                    magic_moves, slow_moves,
                    "\nPosition: {}\nBlockers: {:?}\nMagic:\n{}\nSlow:\n{}",
                    pos_str, blockers, magic_moves, slow_moves
                );
            }
        }
    }

    /// Runs a benchmark comparing magic lookup vs traditional move generation
    pub fn benchmark_rook_moves() {
        let iterations = 100_000_000;

        // Test positions - mix of center, edge, and blocked positions
        let test_cases = [
            ("Empty center", "d4", vec![]),
            ("Center blocked", "d4", vec!["d3", "e4", "d6"]),
            ("Corner empty", "h1", vec![]),
            ("Edge blocked", "h4", vec!["h3", "h5"]),
            ("Heavily blocked", "e4", vec!["e3", "e5", "d4", "f4"]),
        ];

        println!(
            "\nBenchmarking rook move generation ({} iterations each):",
            iterations
        );
        println!("{:-<60}", "");

        for (name, pos_str, blockers) in test_cases {
            let pos = Pos::from_algebraic(pos_str).unwrap();

            // Set up blocking pieces
            let mut occupied = Bitboard::new();
            for &blocker in &blockers {
                if let Ok(p) = Pos::from_algebraic(blocker) {
                    occupied.set(p);
                }
            }

            // Benchmark magic lookup
            let start = Instant::now();
            let mut magic_result = None;
            for _ in 0..iterations {
                magic_result = Some(get_rook_moves_magic(pos, occupied));
            }
            let magic_time = start.elapsed();

            // Benchmark traditional method
            let start = Instant::now();
            let mut trad_result = None;
            for _ in 0..iterations {
                trad_result = Some(generate_rook_moves(pos, occupied));
            }
            let trad_time = start.elapsed();

            // Verify results match
            assert_eq!(
                magic_result, trad_result,
                "Move generation methods produced different results!"
            );

            println!("\nTest case: {}", name);
            println!("Position: {}, Blockers: {:?}", pos_str, blockers);
            println!("Magic lookup: {:?}", magic_time);
            println!("Traditional: {:?}", trad_time);
            println!(
                "Speedup: {:.2}x",
                trad_time.as_nanos() as f64 / magic_time.as_nanos() as f64
            );
        }
    }

    #[test]
    fn test_benchmark_moves() {
        benchmark_rook_moves();
    }

    #[test]
    fn test_move_consistency() {
        // Test a large number of random positions to ensure
        // magic lookup always matches traditional method
        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            // Random rook position
            let pos = Pos(rng.gen_range(0..64));

            // Random blocking pieces (about 10 of them)
            let mut occupied = Bitboard::new();
            for _ in 0..10 {
                occupied.set(Pos(rng.gen_range(0..64)));
            }

            let magic_moves = get_rook_moves_magic(pos, occupied);
            let trad_moves = generate_rook_moves(pos, occupied);

            assert_eq!(
                magic_moves,
                trad_moves,
                "\nPosition: {}\nOccupied:\n{}\nMagic moves:\n{}\nTraditional moves:\n{}",
                pos.to_algebraic(),
                occupied,
                magic_moves,
                trad_moves
            );
        }
    }

    #[test]
    fn test_all_squares_all_patterns() {
        // Test every square with every relevant occupancy pattern
        for sq in 0..64 {
            let pos = Pos(sq);
            let mask = generate_rook_occupancy_mask(pos);
            let patterns = generate_occupancy_patterns(mask);

            for pattern in patterns {
                let magic_moves = get_rook_moves_magic(pos, pattern);
                let trad_moves = generate_rook_moves(pos, pattern);

                assert_eq!(
                    magic_moves,
                    trad_moves,
                    "\nPosition: {}\nPattern:\n{}\nMagic moves:\n{}\nTraditional moves:\n{}",
                    pos.to_algebraic(),
                    pattern,
                    magic_moves,
                    trad_moves
                );
            }
        }
    }
}
