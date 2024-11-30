use clap::Parser;
use rescue_chess::{piece_move::GameType, search::game_state::GameState, Color, Position};
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

    let position = match args.starting_fen {
        Some(fen) => fen.parse::<Position>().expect("Invalid FEN string"),
        None => Position::start_position(),
    };

    let mut game_state = GameState::from_position(position);
    game_state.debug_logs_1 = true;

    println!("\nStarting position:");
    println!(
        "{}",
        game_state
            .current_position
            .to_board_string_with_rank_file(args.unicode)
    );

    while !game_state.current_position.is_checkmate(game_type).unwrap() {
        let mut is_blacks_turn = game_state.current_turn == Color::Black;

        println!(
            "\nMove {}: {} to play",
            game_state.move_number,
            if is_blacks_turn { "Black" } else { "White" }
        );

        let (result, stats) = game_state.search_and_apply().unwrap();

        if let Some(best_move) = result.best_move {
            println!("Best move: {}", best_move);
            println!("Score: {}", result.score);

            is_blacks_turn = !is_blacks_turn;

            println!("Nodes searched: {}", stats.nodes_searched);
            println!("Time taken: {}", stats.time_taken_ms);
            println!("Cache hits: {}", stats.cached_positions);
            println!("Pruned: {}", stats.pruned);

            println!("\nPosition after {}:", best_move);
            println!(
                "{}",
                if is_blacks_turn {
                    game_state
                        .current_position
                        .inverted()
                        .to_board_string_with_rank_file(args.unicode)
                } else {
                    game_state
                        .current_position
                        .to_board_string_with_rank_file(args.unicode)
                }
            );
            println!(
                "{}",
                if is_blacks_turn {
                    game_state.current_position.inverted().to_fen()
                } else {
                    game_state.current_position.to_fen()
                }
            );
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
        if game_state.current_turn == Color::Black {
            game_state
                .current_position
                .inverted()
                .to_board_string_with_rank_file(args.unicode)
        } else {
            game_state
                .current_position
                .to_board_string_with_rank_file(args.unicode)
        }
    );
    println!("{}", game_state.current_position.to_fen())
}
