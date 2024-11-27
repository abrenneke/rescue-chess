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

    #[arg(short = 'b', long)]
    pub black: bool,

    #[arg(short = 'p', long)]
    pub print_board: bool,

    pub fen: String,
}

fn main() {
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
            let position = if args.black {
                position.inverted()
            } else {
                position
            };

            if args.print_board {
                println!("{}", position.to_board_string_with_rank_file(true));
            }

            let mut transposition_table = TranspositionTable::new();
            let mut state = SearchState::new(&mut transposition_table);

            let params = SearchParams {
                depth,
                game_type,
                ..Default::default()
            };

            let result = alpha_beta::search(&position, &mut state, params);
            println!("{}", result.best_move.unwrap());

            if args.stats {
                println!("Nodes searched: {}", state.nodes_searched);
                println!("Cached positions: {}", state.cached_positions);
                println!("Time taken: {}ms", state.start_time.elapsed().as_millis());
                println!("Pruned: {}", state.pruned);
                println!("Score: {}", result.score);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
