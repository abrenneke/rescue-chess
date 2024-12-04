use crate::bitboard::Bitboard;
use crate::piece::occupancy::generate_occupancy_patterns;
use crate::pos::Pos;
use std::sync::LazyLock;

use super::occupancy::{generate_bishop_move_table, generate_bishop_occupancy_mask};

#[derive(Debug, Clone, PartialEq, Eq)]
struct MagicEntry {
    magic: u64,
    shift: u32,
    mask: Bitboard,
    moves: Vec<Bitboard>,
}

// The magic numbers you found, stored as (magic_number, shift) pairs
static BISHOP_MAGICS: [(u64, u32); 64] = [
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
        let (magic, shift) = BISHOP_MAGICS[sq as usize];
        let mask = generate_bishop_occupancy_mask(pos);
        let patterns = generate_occupancy_patterns(mask);
        let move_table = generate_bishop_move_table(pos);

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

/// Gets bishop moves using magic lookup
pub fn get_bishop_moves_magic(pos: Pos, occupied: Bitboard) -> Bitboard {
    let entry = &MAGIC_TABLE[pos.0 as usize];
    let relevant = occupied & entry.mask;
    let index = ((relevant.0.wrapping_mul(entry.magic)) >> entry.shift) as usize;
    entry.moves[index]
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use rand::Rng;

    use crate::piece::bishop::occupancy::generate_bishop_moves;

    use super::*;

    #[test]
    fn test_magic_lookup_empty_board() {
        let pos = Pos::from_algebraic("d4").unwrap();
        let moves = get_bishop_moves_magic(pos, Bitboard::new());

        // On empty board, bishop should attack all diagonals
        let expected: Bitboard = "
            . . . . . . . x
            x . . . . . x .
            . x . . . x . .
            . . x . x . . .
            . . . . . . . .
            . . x . x . . .
            . x . . . x . .
            x . . . . . x .
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
        occupied.set(Pos::from_algebraic("f6").unwrap());
        occupied.set(Pos::from_algebraic("b2").unwrap());

        let moves = get_bishop_moves_magic(pos, occupied);

        // Should include blocking squares but not beyond
        let expected: Bitboard = "
            . . . . . . . .
            x . . . . . . .
            . x . . . x . .
            . . x . x . . .
            . . . . . . . .
            . . x . x . . .
            . x . . . x . .
            . . . . . . x .
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
        // Test corner bishop
        let pos = Pos::from_algebraic("h1").unwrap();
        let moves = get_bishop_moves_magic(pos, Bitboard::new());

        assert!(moves.get(Pos::from_algebraic("g2").unwrap()));
        assert!(moves.get(Pos::from_algebraic("f3").unwrap()));
        assert!(!moves.get(Pos::from_algebraic("h2").unwrap()));

        // Test fully surrounded bishop
        let pos = Pos::from_algebraic("e4").unwrap();
        let mut occupied = Bitboard::new();
        for p in ["d3", "d5", "f3", "f5"] {
            occupied.set(Pos::from_algebraic(p).unwrap());
        }

        let moves = get_bishop_moves_magic(pos, occupied);
        assert_eq!(
            moves.count(),
            4,
            "Surrounded bishop should see exactly 4 squares"
        );
    }

    #[test]
    fn test_magic_lookup_matches_slow() {
        // Test that magic lookup matches the slower generate_bishop_moves
        // for various positions and occupancy patterns
        let test_positions = ["d4", "h1", "a8", "e4"];
        let test_blockers = [
            vec![],
            vec!["c3"],
            vec!["f6", "b2"],
            vec!["d3", "d5", "f3", "f5"],
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

                let magic_moves = get_bishop_moves_magic(pos, occupied);
                let slow_moves = generate_bishop_moves(pos, occupied);

                assert_eq!(
                    magic_moves, slow_moves,
                    "\nPosition: {}\nBlockers: {:?}\nMagic:\n{}\nSlow:\n{}",
                    pos_str, blockers, magic_moves, slow_moves
                );
            }
        }
    }

    /// Runs a benchmark comparing magic lookup vs traditional move generation
    pub fn benchmark_bishop_moves() {
        let iterations = 100_000_000;

        // Test positions - mix of center, edge, and blocked positions
        let test_cases = [
            ("Empty center", "d4", vec![]),
            ("Center blocked", "d4", vec!["c3", "e5", "f6"]),
            ("Corner empty", "h1", vec![]),
            ("Edge blocked", "h4", vec!["g3", "g5"]),
            ("Heavily blocked", "e4", vec!["d3", "d5", "f3", "f5"]),
        ];

        println!(
            "\nBenchmarking bishop move generation ({} iterations each):",
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
                magic_result = Some(get_bishop_moves_magic(pos, occupied));
            }
            let magic_time = start.elapsed();

            // Benchmark traditional method
            let start = Instant::now();
            let mut trad_result = None;
            for _ in 0..iterations {
                trad_result = Some(generate_bishop_moves(pos, occupied));
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
        benchmark_bishop_moves();
    }

    #[test]
    fn test_move_consistency() {
        // Test a large number of random positions to ensure
        // magic lookup always matches traditional method
        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            // Random bishop position
            let pos = Pos(rng.gen_range(0..64));

            // Random blocking pieces (about 10 of them)
            let mut occupied = Bitboard::new();
            for _ in 0..10 {
                occupied.set(Pos(rng.gen_range(0..64)));
            }

            let magic_moves = get_bishop_moves_magic(pos, occupied);
            let trad_moves = generate_bishop_moves(pos, occupied);

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
            let mask = generate_bishop_occupancy_mask(pos);
            let patterns = generate_occupancy_patterns(mask);

            for pattern in patterns {
                let magic_moves = get_bishop_moves_magic(pos, pattern);
                let trad_moves = generate_bishop_moves(pos, pattern);

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
