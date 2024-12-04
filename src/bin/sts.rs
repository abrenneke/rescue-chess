use clap::Parser;
use crossbeam::channel;
use num_cpus;
use rayon::{self, prelude::*};
use rescue_chess::{
    piece_move::GameType,
    position::extended_fen::{EpdOperand, ExtendedPosition},
    search::{
        alpha_beta::{self, SearchParams},
        search_results::SearchState,
        transposition_table::TranspositionTable,
    },
    Color,
};
use std::fs::File;
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

#[derive(Parser)]
struct Cli {
    #[arg(short = 'd', long)]
    pub depth: Option<u32>,

    #[arg(short = 'v', long)]
    pub verbose: bool,

    #[arg(short = 's', long)]
    pub stats: bool,

    #[arg(short = 'j', long, default_value_t = num_cpus::get())]
    pub jobs: usize,

    /// Path to the EPD file containing the Strategic Test Suite
    pub epd_file: PathBuf,
}

#[derive(Debug)]
struct TestResult {
    position_number: usize,
    success: bool,
    error: Option<String>,

    #[allow(dead_code)]
    time_taken_ms: u128,
    position_output: String,
    stats_output: Option<String>,
}

fn run_position_test(
    position: ExtendedPosition,
    depth: u32,
    stats: bool,
    position_number: usize,
) -> TestResult {
    let start_time = Instant::now();
    let mut output = String::new();
    let mut stats_output = None;

    let mut transposition_table = TranspositionTable::new();
    let mut state = SearchState::new(&mut transposition_table);

    let params = SearchParams {
        depth,
        game_type: GameType::Classic,
        debug_print_verbose: false,
        ..Default::default()
    };

    let result = match alpha_beta::search(&position.position, &mut state, params, 0) {
        Ok(r) => r,
        Err(e) => {
            return TestResult {
                position_number,
                success: false,
                error: Some(format!("Search error: {}", e)),
                time_taken_ms: start_time.elapsed().as_millis(),
                position_output: String::new(),
                stats_output: None,
            }
        }
    };

    let best_move = match result.best_move {
        Some(m) => {
            if position.position.true_active_color == Color::White {
                m.to_string()
            } else {
                m.inverted().to_string()
            }
        }
        None => {
            return TestResult {
                position_number,
                success: false,
                error: Some("No best move found".to_string()),
                time_taken_ms: start_time.elapsed().as_millis(),
                position_output: String::new(),
                stats_output: None,
            }
        }
    };

    let expected_best_move = match position.get_operation("bm").and_then(|ops| ops.first()) {
        Some(EpdOperand::SanMove(expected)) => expected,
        _ => {
            return TestResult {
                position_number,
                success: false,
                error: Some("Invalid or missing best move in EPD".to_string()),
                time_taken_ms: start_time.elapsed().as_millis(),
                position_output: String::new(),
                stats_output: None,
            }
        }
    };

    let success = best_move == *expected_best_move;

    if !success {
        if let Some(id) = position.get_operation("id").and_then(|ops| ops.first()) {
            output.push_str(&format!("\nPosition ID: {:?}\n", id));
        }
        output.push_str(&format!("Position {}\n", position_number));
        output.push_str(&format!("Expected best move: {}\n", expected_best_move));
        output.push_str(&format!("Found best move: {}\n", best_move));
        output.push_str(&format!(
            "Principal variation: {:?}\n",
            result.principal_variation
        ));
        output.push_str(&format!("{}\n", position.position.to_fen()))
    }

    if stats {
        let mut stats = String::new();
        stats.push_str(&format!("Position {} stats:\n", position_number));
        stats.push_str(&format!("Nodes searched: {}\n", state.data.nodes_searched));
        stats.push_str(&format!(
            "Cached positions: {}\n",
            state.data.cached_positions
        ));
        stats.push_str(&format!(
            "Time taken: {}ms\n",
            start_time.elapsed().as_millis()
        ));
        stats.push_str(&format!("Pruned: {}\n", state.data.pruned));
        stats.push_str(&format!("Score: {}\n", result.score));
        stats_output = Some(stats);
    }

    TestResult {
        position_number,
        success,
        error: None,
        time_taken_ms: start_time.elapsed().as_millis(),
        position_output: output,
        stats_output,
    }
}

fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_level(false)
        .without_time()
        .with_file(false)
        .with_line_number(false)
        .with_thread_names(false)
        .with_thread_ids(false)
        .with_ansi(false)
        .with_target(false)
        .init();

    let args = Cli::parse();
    let depth = args.depth.unwrap_or(3);

    println!("Running with {} worker threads", args.jobs);

    let file = File::open(&args.epd_file)?;
    let reader = io::BufReader::new(file);
    let positions: Vec<_> = reader
        .lines()
        .enumerate()
        .filter_map(|(i, line)| match line {
            Ok(l) if !l.trim().is_empty() => Some((i + 1, l)),
            _ => None,
        })
        .collect();

    let total_positions = positions.len();
    println!("Loaded {} positions", total_positions);

    let (sender, receiver) = channel::unbounded();
    let completed = Arc::new(AtomicUsize::new(0));
    let start_time = Instant::now();

    let completed_clone = completed.clone();
    thread::spawn(move || {
        let mut last_completed = 0;

        while completed_clone.load(Ordering::Relaxed) < total_positions {
            thread::sleep(std::time::Duration::from_secs(1));
            let completed = completed_clone.load(Ordering::Relaxed);

            if completed == last_completed {
                continue;
            }

            last_completed = completed;

            println!(
                "Progress: {}/{} ({:.1}%)",
                completed,
                total_positions,
                (completed as f64 / total_positions as f64) * 100.0
            );
        }
    });

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(args.jobs)
        .build()?;

    pool.scope(|s| {
        for (position_number, line) in positions {
            let sender = sender.clone();
            let completed = completed.clone();
            let stats = args.stats;

            s.spawn(move |_| {
                let result = match ExtendedPosition::parse_from_epd(&line) {
                    Ok(position) => run_position_test(position, depth, stats, position_number),
                    Err(e) => TestResult {
                        position_number,
                        success: false,
                        error: Some(format!("Parse error: {}", e)),
                        time_taken_ms: 0,
                        position_output: String::new(),
                        stats_output: None,
                    },
                };

                sender.send(result).unwrap();
                completed.fetch_add(1, Ordering::Relaxed);
            });
        }
    });

    drop(sender);

    let mut results: Vec<TestResult> = receiver.iter().collect();
    results.sort_by_key(|r| r.position_number);

    let successful_positions = results.iter().filter(|r| r.success).count();
    let total_time = start_time.elapsed();

    println!("\nFinal Results:");
    println!("Total positions tested: {}", total_positions);
    println!("Successful positions: {}", successful_positions);
    println!(
        "Success rate: {:.1}%",
        (successful_positions as f64 / total_positions as f64) * 100.0
    );
    println!("Total time: {:.2}s", total_time.as_secs_f64());

    println!("\nFailed Positions:");
    for result in results.iter().filter(|r| !r.success) {
        if let Some(error) = &result.error {
            println!("\nPosition {}: Error - {}", result.position_number, error);
        } else {
            print!("{}", result.position_output);
        }

        if args.stats {
            if let Some(stats) = &result.stats_output {
                print!("{}", stats);
            }
        }
    }

    if args.verbose {
        println!("\nAll Positions:");
        for result in &results {
            if !result.position_output.is_empty() {
                print!("{}", result.position_output);
            }
            if args.stats {
                if let Some(stats) = &result.stats_output {
                    print!("{}", stats);
                }
            }
        }
    }

    Ok(())
}
