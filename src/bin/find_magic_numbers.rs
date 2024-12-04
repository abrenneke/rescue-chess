use rand::Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rescue_chess::bitboard::Bitboard;
use rescue_chess::piece::bishop::occupancy::{
    generate_bishop_move_table, generate_bishop_occupancy_mask,
};
use rescue_chess::piece::occupancy::generate_occupancy_patterns;
use rescue_chess::piece::queen::occupancy::{
    generate_queen_move_table, generate_queen_occupancy_mask,
};
use rescue_chess::piece::rook::occupancy::{
    generate_rook_move_table, generate_rook_occupancy_mask,
};
use rescue_chess::pos::Pos;
use rescue_chess::PieceType;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

// A candidate magic number and its required shift
#[derive(Debug, Clone, Copy)]
pub struct Magic {
    number: u64,
    shift: u32,
}

impl Magic {
    fn new(number: u64, shift: u32) -> Self {
        Magic { number, shift }
    }
}

// Adjust random_sparse_u64 to generate more bits for queens
fn random_sparse_u64(rng: &mut impl Rng) -> u64 {
    let mut result = 0u64;
    // Use more bits for potentially better distribution
    let num_bits = 12;
    for _ in 0..num_bits {
        result |= 1u64 << rng.gen_range(0..64);
    }
    result
}

/// Tests if a magic number candidate works for a given square's patterns
fn test_magic(
    _square: Pos,
    magic: Magic,
    occupancy_patterns: &[Bitboard],
    move_table: &[Bitboard],
) -> bool {
    let size = 1 << (64 - magic.shift);
    let mut used = vec![None; size];

    if magic.shift >= 64 {
        return false;
    }

    // For each possible occupancy pattern
    for (i, &occupancy) in occupancy_patterns.iter().enumerate() {
        // Calculate the magic index
        let index = ((occupancy.0.wrapping_mul(magic.number)) >> magic.shift) as usize;

        if index >= used.len() {
            return false;
        }

        match used[index] {
            None => {
                // First time we've seen this index, store the moves
                used[index] = Some(move_table[i]);
            }
            Some(prev_moves) => {
                // We've seen this index before, verify moves match
                if prev_moves != move_table[i] {
                    return false; // Collision with different moves
                }
            }
        }
    }

    true
}

fn main() {
    let piece_type = PieceType::Queen;

    // Process squares in parallel
    let magics: Vec<_> = (0..64)
        .into_iter()
        .map(|sq| {
            let pos = Pos(sq);
            let magic = find_magic_for_square_parallel(pos, piece_type);
            (pos, magic)
        })
        .collect();

    println!("\nMagic numbers for {}:", piece_type);
    for (pos, magic) in magics {
        println!(
            "{}: ({:#x}, {}),",
            pos.to_algebraic(),
            magic.number,
            magic.shift
        );
    }
}

/// Finds a magic number using parallel search
pub fn find_magic_for_square_parallel(square: Pos, piece_type: PieceType) -> Magic {
    let start_time = Instant::now();

    // Generate all the patterns and moves we need to encode
    let (mask, move_table) = match piece_type {
        PieceType::Bishop => {
            let mask = generate_bishop_occupancy_mask(square);
            let move_table = generate_bishop_move_table(square);
            (mask, move_table)
        }
        PieceType::Rook => {
            let mask = generate_rook_occupancy_mask(square);
            let move_table = generate_rook_move_table(square);
            (mask, move_table)
        }
        PieceType::Queen => {
            let mask = generate_queen_occupancy_mask(square);
            let move_table = generate_queen_move_table(square);
            (mask, move_table)
        }
        _ => {
            panic!("Invalid piece type");
        }
    };

    let occupancy_patterns = generate_occupancy_patterns(mask);
    let required_bits = (occupancy_patterns.len() as f64).log2().ceil() as u32;

    // For rooks and queens, adjust shift based on square position
    let shift = match piece_type {
        PieceType::Rook => {
            let x = square.get_col();
            let y = square.get_row();
            if x == 0 || x == 7 || y == 0 || y == 7 {
                52
            } else if x == 1 || x == 6 || y == 1 || y == 6 {
                51
            } else {
                50
            }
        }
        PieceType::Bishop => 64 - required_bits,
        PieceType::Queen => {
            // Add a small buffer to required_bits to ensure enough space
            // The buffer helps avoid collisions while keeping tables reasonably sized
            let buffer = 3; // Adjust this based on empirical testing
            64 - (required_bits + buffer)
        }
        _ => 64 - required_bits,
    };

    println!(
        "Searching for magic number for {} at square {} (shift: {})",
        piece_type,
        square.to_algebraic(),
        shift
    );
    println!(
        "Need {} bits (shift {}) for {} patterns for {}",
        required_bits,
        shift,
        occupancy_patterns.len(),
        square.to_algebraic()
    );

    // Share patterns and move table between threads
    let patterns = Arc::new(occupancy_patterns);
    let moves = Arc::new(move_table);
    let attempts = Arc::new(AtomicU64::new(0));
    let found_magic = Arc::new(AtomicU64::new(0));
    let found = Arc::new(std::sync::atomic::AtomicBool::new(false));

    // Create multiple search batches that can run in parallel
    let batch_size = 1_000;
    let num_threads = rayon::current_num_threads();
    let magic = (0..num_threads)
        .into_par_iter()
        .find_map_any(|_| {
            let patterns = Arc::clone(&patterns);
            let moves = Arc::clone(&moves);
            let attempts = Arc::clone(&attempts);
            let found = Arc::clone(&found);
            let found_magic = Arc::clone(&found_magic);

            let mut rng = rand::thread_rng();
            let mut local_attempts = 0;

            while !found.load(Ordering::Relaxed) {
                // Generate and test a batch of candidates
                for _ in 0..batch_size {
                    local_attempts += 1;
                    let candidate = random_sparse_u64(&mut rng);
                    let magic = Magic::new(candidate, shift);

                    if test_magic(square, magic, &patterns, &moves) {
                        found.store(true, Ordering::Relaxed);
                        found_magic.store(candidate, Ordering::Relaxed);
                        attempts.fetch_add(local_attempts, Ordering::Relaxed);
                        return Some(magic);
                    }
                }

                // Update global attempt counter periodically
                attempts.fetch_add(batch_size, Ordering::Relaxed);

                // Print progress every million attempts
                let total_attempts = attempts.load(Ordering::Relaxed);
                if total_attempts % 1_000 == 0 {
                    println!(
                        "Tried {}k candidates for {}...",
                        total_attempts / 1_000,
                        square.to_algebraic()
                    );
                }
            }
            None
        })
        .unwrap();

    println!(
        "Found magic for {} with shift {} after {} attempts ({:?})",
        square.to_algebraic(),
        magic.shift,
        attempts.load(Ordering::Relaxed),
        start_time.elapsed()
    );

    magic
}
