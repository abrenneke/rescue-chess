use clap::Parser;
use rescue_chess::{
    piece_move::GameType,
    search::{
        alpha_beta::{self, SearchParams},
        iterative_deepening::IterativeDeepeningData,
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

    #[arg(long)]
    pub all_scores: bool,

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

            let mut iterative_deepening_data = IterativeDeepeningData::new();

            iterative_deepening_data.update_position(position.clone());

            let params = SearchParams {
                depth,
                game_type,
                ..Default::default()
            };

            iterative_deepening_data.search(params.clone());

            let best_move = iterative_deepening_data.best_move;
            let best_score = iterative_deepening_data.best_score;

            if position.true_active_color == Color::White {
                println!("{}", best_move.unwrap());
            } else {
                println!("{}", best_move.unwrap().inverted());
            }

            if args.all_scores {
                println!("Getting all scores...");

                let mut transposition_table = TranspositionTable::new();
                let mut state = SearchState::new(&mut transposition_table);

                let scored_moves =
                    alpha_beta::score_all_moves(&position, &mut state, params.clone(), 0).unwrap();

                for scored_move in scored_moves {
                    let mut principal_variation = vec![scored_move.mv];
                    let mut is_black = position.true_active_color != Color::Black;

                    for mv in scored_move.principal_variation.unwrap_or_default() {
                        if is_black {
                            principal_variation.push(mv.inverted());
                        } else {
                            principal_variation.push(mv);
                        }
                        is_black = !is_black;
                    }

                    println!(
                        "{}: {}    {{{:?}}}",
                        scored_move.mv, scored_move.score, principal_variation
                    );
                }
            }

            if args.stats {
                println!("------------------------------------");
                println!(
                    "Nodes searched: {}",
                    iterative_deepening_data.stats.nodes_searched
                );
                println!(
                    "Cached positions: {}",
                    iterative_deepening_data.stats.cached_positions
                );
                println!(
                    "Time taken: {}ms",
                    iterative_deepening_data.stats.time_taken_ms
                );
                println!("Pruned: {}", iterative_deepening_data.stats.pruned);
                println!("Score: {}", best_score.unwrap());

                let mut principal_variation = vec![];
                let mut is_black = position.true_active_color == Color::Black;
                for mv in iterative_deepening_data
                    .previous_pv
                    .as_ref()
                    .unwrap()
                    .iter()
                {
                    if is_black {
                        principal_variation.push(mv.inverted());
                    } else {
                        principal_variation.push(mv.clone());
                    }
                    is_black = !is_black;
                }

                println!("Principal variation: {:?}", principal_variation);

                let mut current_position = position.clone();
                for mv in iterative_deepening_data
                    .previous_pv
                    .as_ref()
                    .unwrap()
                    .iter()
                {
                    current_position.apply_move(mv.clone()).unwrap();
                    current_position.invert();
                }

                println!(
                    "Final board state (scored {}) after principal variation: {}",
                    best_score.unwrap(),
                    current_position.to_fen()
                );
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
