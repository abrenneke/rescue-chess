use std::io::Write;

use tracing::trace;

use crate::{uci::UciEngine, Color};

use super::CommandHandler;

/// Represents a go command with all possible search parameters
#[derive(Debug)]
pub struct GoCommand {
    pub search_moves: Option<Vec<String>>,
    pub ponder: bool,
    pub wtime: Option<u64>,
    pub btime: Option<u64>,
    pub winc: Option<u64>,
    pub binc: Option<u64>,
    pub movestogo: Option<u32>,
    pub depth: Option<u32>,
    pub nodes: Option<u64>,
    pub mate: Option<u32>,
    pub movetime: Option<u64>,
    pub infinite: bool,
}

impl CommandHandler for GoCommand {
    fn execute(&self, engine: &mut UciEngine) -> std::io::Result<bool> {
        // Update search depth if specified
        {
            let mut game_state = engine.game_state.lock().unwrap();
            if let Some(depth) = self.depth {
                game_state.search_depth = depth;
            }

            trace!("Searching to depth {}", game_state.search_depth);
            trace!("Current position: {}", game_state.current_position.to_fen());
        }

        let game_state = engine.game_state.clone();
        let stdout = engine.stdout.clone();

        std::thread::spawn(move || {
            let mut game_state = game_state.lock().unwrap();
            let mut stdout = stdout.lock().unwrap();

            // Perform search
            match game_state.search_and_apply() {
                Ok((mut best_move, _)) => {
                    if game_state.current_turn == Color::White {
                        best_move = best_move.inverted();
                    }

                    trace!("Best move: {}", best_move);
                    writeln!(stdout, "bestmove {}", best_move.to_uci()).unwrap();
                }
                Err(e) => {
                    trace!("Error searching: {}", e);
                    writeln!(stdout, "bestmove 0000").unwrap();
                }
            }
            stdout.flush().unwrap();
        });

        Ok(true)
    }
}

impl std::str::FromStr for GoCommand {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        let mut cmd = GoCommand {
            search_moves: None,
            ponder: false,
            wtime: None,
            btime: None,
            winc: None,
            binc: None,
            movestogo: None,
            depth: None,
            nodes: None,
            mate: None,
            movetime: None,
            infinite: false,
        };

        let mut i = 1;
        while i < parts.len() {
            match parts[i] {
                "infinite" => {
                    cmd.infinite = true;
                    i += 1;
                }
                "ponder" => {
                    cmd.ponder = true;
                    i += 1;
                }
                "wtime" => {
                    if i + 1 < parts.len() {
                        cmd.wtime = parts[i + 1].parse().ok();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "btime" => {
                    if i + 1 < parts.len() {
                        cmd.btime = parts[i + 1].parse().ok();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "winc" => {
                    if i + 1 < parts.len() {
                        cmd.winc = parts[i + 1].parse().ok();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "binc" => {
                    if i + 1 < parts.len() {
                        cmd.binc = parts[i + 1].parse().ok();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "movestogo" => {
                    if i + 1 < parts.len() {
                        cmd.movestogo = parts[i + 1].parse().ok();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "depth" => {
                    if i + 1 < parts.len() {
                        cmd.depth = parts[i + 1].parse().ok();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "nodes" => {
                    if i + 1 < parts.len() {
                        cmd.nodes = parts[i + 1].parse().ok();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "mate" => {
                    if i + 1 < parts.len() {
                        cmd.mate = parts[i + 1].parse().ok();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "movetime" => {
                    if i + 1 < parts.len() {
                        cmd.movetime = parts[i + 1].parse().ok();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                _ => i += 1,
            }
        }

        Ok(cmd)
    }
}
