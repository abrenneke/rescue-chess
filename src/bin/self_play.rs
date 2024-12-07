use clap::Parser;
use rescue_chess::{
    features::Features, piece_move::GameType, search::game_state::GameState, Color, Position,
};
use std::{thread, time::Duration};

#[derive(Parser)]
struct Cli {
    #[arg(short = 'd', long, default_value = "3")]
    pub depth: u32,

    #[arg(short = 't', long = "time")]
    pub think_time_ms: Option<u64>,

    #[arg(short = 'c', long)]
    pub classic: bool,

    #[arg(short = 'p', long, default_value = "500")]
    pub pause_ms: u64,

    #[arg(short = 'u', long)]
    pub unicode: bool,

    #[arg(long)]
    pub starting_fen: Option<String>,

    #[arg(short = 'v', long)]
    pub verbose: bool,
}

fn main() {
    let args = Cli::parse();

    if args.verbose {
        tracing_subscriber::fmt::fmt()
            .with_max_level(tracing::Level::TRACE)
            .init();
    }

    let game_type = if args.classic {
        GameType::Classic
    } else {
        GameType::Rescue
    };

    let position = match args.starting_fen {
        Some(fen) => fen.parse::<Position>().expect("Invalid FEN string"),
        None => Position::start_position(),
    };

    let think_time_ms = args.think_time_ms.unwrap_or(5_000);

    let mut game_state = GameState {
        features: Features {
            ..Default::default()
        },
        debug_logs_verbose: true,
        search_depth: args.depth,
        time_limit_ms: think_time_ms,
        ..GameState::from_position(position)
    };
    game_state.game_type = game_type;

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

        let (best_move, stats) = game_state.search_and_apply().unwrap();

        println!("Best move: {}", best_move);

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
                    .to_board_string_with_rank_file_holding()
            } else {
                game_state
                    .current_position
                    .to_board_string_with_rank_file_holding()
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
