use clap::Parser;
use rescue_chess::{
    search::{alpha_beta, search_results::SearchState, transposition_table::TranspositionTable},
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

    pub fen: String,
}

fn main() {
    let args = Cli::parse();

    let position = args.fen.parse::<Position>();
    let depth = args.depth.unwrap_or(3);

    match position {
        Ok(position) => {
            let mut transposition_table = TranspositionTable::new();
            let mut state = SearchState::new(&mut transposition_table);
            let result = alpha_beta::search(&position, depth, &mut state);
            println!("{}", result.best_move);

            if args.stats {
                println!("Nodes searched: {}", state.nodes_searched);
                println!("Cached positions: {}", state.cached_positions);
                println!("Time taken: {}ms", state.start_time.elapsed().as_millis());
                println!("Pruned: {}", state.pruned);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
