use clap::Parser;
use rescue_chess::{
    piece_move::GameType,
    search::{
        alpha_beta::{self, SearchParams},
        search_results::SearchState,
        transposition_table::TranspositionTable,
    },
    Position,
};
use std::{thread, time::Duration};

#[derive(Parser)]
struct Cli {
    #[arg(short = 'd', long, default_value = "3")]
    pub depth: u32,

    #[arg(short = 't', long = "time")]
    pub think_time_ms: Option<u32>,

    #[arg(short = 'c', long)]
    pub classic: bool,

    #[arg(short = 'p', long, default_value = "500")]
    pub pause_ms: u64,

    #[arg(short = 'u', long)]
    pub unicode: bool,

    #[arg(long)]
    pub starting_fen: Option<String>,
}

fn main() {
    let args = Cli::parse();

    let game_type = if args.classic {
        GameType::Classic
    } else {
        GameType::Rescue
    };

    let mut position = match args.starting_fen {
        Some(fen) => fen.parse::<Position>().expect("Invalid FEN string"),
        None => Position::start_position(),
    };

    // Create separate transposition tables for each player
    let mut transposition_table = TranspositionTable::new();

    let mut move_number = 1;
    let mut is_blacks_turn = false;

    println!("\nStarting position:");
    println!("{}", position.to_board_string_with_rank_file(args.unicode));

    while !position.is_checkmate(game_type).unwrap() {
        let mut state = SearchState::new(&mut transposition_table);

        let params = SearchParams {
            depth: args.depth,
            game_type,
            ..Default::default()
        };

        println!(
            "\nMove {}: {} to play",
            move_number,
            if is_blacks_turn { "Black" } else { "White" }
        );

        let result = alpha_beta::search(&position, &mut state, params);

        if let Some(best_move) = result.best_move {
            println!("Best move: {}", best_move);
            println!(
                "Score: {}",
                if is_blacks_turn {
                    -result.score
                } else {
                    result.score
                }
            );
            println!("Nodes searched: {}", state.nodes_searched);
            println!("Time taken: {}", state.start_time.elapsed().as_millis());
            println!("Cache hits: {}", state.cached_positions);
            println!("Pruned: {}", state.pruned);

            position.apply_move(best_move.clone()).unwrap();

            println!("\nPosition after {}:", best_move);
            println!(
                "{}",
                if is_blacks_turn {
                    position
                        .inverted()
                        .to_board_string_with_rank_file(args.unicode)
                } else {
                    position.to_board_string_with_rank_file(args.unicode)
                }
            );
            println!(
                "{}",
                if is_blacks_turn {
                    position.inverted().to_fen()
                } else {
                    position.to_fen()
                }
            );

            // Invert the position for the next player
            position.invert();
            is_blacks_turn = !is_blacks_turn;

            if !is_blacks_turn {
                move_number += 1;
            }
        } else {
            println!("No legal moves available!");
            break;
        }

        // Pause between moves for readability
        thread::sleep(Duration::from_millis(args.pause_ms));
    }

    println!("\nGame Over!");
    // Note: You might need to adjust the winner detection based on your engine's implementation
    println!("Final position:");
    println!(
        "{}",
        if is_blacks_turn {
            position
                .inverted()
                .to_board_string_with_rank_file(args.unicode)
        } else {
            position.to_board_string_with_rank_file(args.unicode)
        }
    );
}
