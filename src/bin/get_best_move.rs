use clap::Parser;
use rescue_chess::{
    piece_move::GameType,
    search::{
        alpha_beta::{self, Features, SearchParams},
        search_results::SearchState,
        transposition_table::TranspositionTable,
    },
    Color, Position,
};

#[derive(Parser)]
struct Cli {
    #[arg(short = 'd', long)]
    pub depth: Option<u32>,

    #[arg(short = 't', long = "time")]
    pub max_time: Option<u32>,

    #[arg(short = 's', long)]
    pub stats: bool,

    #[arg(short = 'c', long)]
    pub classic: bool,

    #[arg(short = 'p', long)]
    pub print_board: bool,

    #[arg(long)]
    pub print_valid_moves: bool,

    #[arg(short = 'v', long)]
    pub verbose: bool,

    pub fen: String,
}

fn main() {
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

    let position = args.fen.parse::<Position>();

    let depth = args.depth.unwrap_or(3);

    let game_type = if args.classic {
        GameType::Classic
    } else {
        GameType::Rescue
    };

    match position {
        Ok(position) => {
            if args.print_board {
                println!("{}", position.to_board_string_with_rank_file(true));
            }

            if args.print_valid_moves {
                println!(
                    "{}",
                    position
                        .get_all_legal_moves(game_type)
                        .unwrap()
                        .iter()
                        .map(|m| m.to_string())
                        .collect::<Vec<String>>()
                        .join(" ")
                );
            }

            let mut transposition_table = TranspositionTable::new();
            let mut state = SearchState::new(&mut transposition_table);

            let params = SearchParams {
                depth,
                game_type,
                // debug_print: true,
                // debug_print_all_moves: true,
                debug_print_verbose: args.verbose,
                // enable_transposition_table: false,
                features: Features {
                    ..Default::default()
                },
                // enable_window_search: false,
                ..Default::default()
            };

            let result = alpha_beta::search(&position, &mut state, params, 0).unwrap();

            if position.true_active_color == Color::White {
                println!("{}", result.best_move.unwrap());
            } else {
                println!("{}", result.best_move.unwrap().inverted());
            }

            if args.stats {
                println!("Nodes searched: {}", state.data.nodes_searched);
                println!("Cached positions: {}", state.data.cached_positions);
                println!(
                    "Time taken: {}ms",
                    state.data.start_time.elapsed().as_millis()
                );
                println!("Pruned: {}", state.data.pruned);
                println!("Score: {}", result.score);
                println!("Principal variation: {:?}", result.principal_variation);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
