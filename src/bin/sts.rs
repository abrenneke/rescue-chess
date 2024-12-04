use clap::Parser;
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

#[derive(Parser)]
struct Cli {
    #[arg(short = 'd', long)]
    pub depth: Option<u32>,

    #[arg(short = 'v', long)]
    pub verbose: bool,

    #[arg(short = 's', long)]
    pub stats: bool,

    /// Path to the EPD file containing the Strategic Test Suite
    pub epd_file: PathBuf,
}

fn run_position_test(
    position: ExtendedPosition,
    depth: u32,
    verbose: bool,
    stats: bool,
) -> Result<bool, anyhow::Error> {
    let mut transposition_table = TranspositionTable::new();
    let mut state = SearchState::new(&mut transposition_table);

    let params = SearchParams {
        depth,
        game_type: GameType::Classic,
        debug_print_verbose: verbose,
        ..Default::default()
    };

    let result = alpha_beta::search(&position.position, &mut state, params, 0).unwrap();
    let best_move = result
        .best_move
        .ok_or_else(|| anyhow::anyhow!("No best move found"))?;

    let best_move = if position.position.true_active_color == Color::White {
        best_move.to_string()
    } else {
        best_move.inverted().to_string()
    };

    let expected_best_move = position
        .get_operation("bm")
        .and_then(|ops| ops.first())
        .ok_or_else(|| anyhow::anyhow!("No best move operation in EPD"))?;

    match expected_best_move {
        EpdOperand::SanMove(expected) => {
            let success = best_move == *expected;

            if !success || verbose {
                if let Some(id) = position.get_operation("id").and_then(|ops| ops.first()) {
                    println!("\nPosition ID: {:?}", id);
                }
                println!("Expected best move: {}", expected);
                println!("Found best move: {}", best_move);
                println!("Principal variation: {:?}", result.principal_variation);
                println!(
                    "{}",
                    position.position.to_board_string_with_rank_file_holding()
                );
            }

            if stats {
                println!("Nodes searched: {}", state.data.nodes_searched);
                println!("Cached positions: {}", state.data.cached_positions);
                println!(
                    "Time taken: {}ms",
                    state.data.start_time.elapsed().as_millis()
                );
                println!("Pruned: {}", state.data.pruned);
                println!("Score: {}", result.score);
            }

            Ok(success)
        }
        _ => Err(anyhow::anyhow!("Invalid best move operand type")),
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

    let file = File::open(&args.epd_file)?;
    let reader = io::BufReader::new(file);

    let mut total_positions = 0;
    let mut successful_positions = 0;

    // Process each line in the EPD file
    for (line_number, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        total_positions += 1;
        println!("\nTesting position {} ...", line_number + 1);

        match ExtendedPosition::parse_from_epd(&line) {
            Ok(position) => match run_position_test(position, depth, args.verbose, args.stats) {
                Ok(true) => {
                    successful_positions += 1;
                    if args.verbose {
                        println!("✓ Position {} passed", line_number + 1);
                    }
                }
                Ok(false) => {
                    println!("✗ Position {} failed", line_number + 1);
                }
                Err(e) => {
                    eprintln!("Error testing position {}: {}", line_number + 1, e);
                }
            },
            Err(e) => {
                eprintln!("Error parsing position {}: {}", line_number + 1, e);
            }
        }
    }

    // Print final results
    println!("\nResults:");
    println!("Total positions tested: {}", total_positions);
    println!("Successful positions: {}", successful_positions);
    println!(
        "Success rate: {:.1}%",
        (successful_positions as f64 / total_positions as f64) * 100.0
    );

    Ok(())
}
