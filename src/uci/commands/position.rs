use tracing::trace;

use crate::{search::game_state::GameState, uci::UciEngine, Color, PieceMove, Position};

use super::CommandHandler;

/// Represents a position command with either a FEN or startpos and optional moves
#[derive(Debug)]
pub struct PositionCommand {
    pub fen: Option<String>,
    pub moves: Vec<String>,
}

impl CommandHandler for PositionCommand {
    fn execute(&self, engine: &mut UciEngine) -> std::io::Result<bool> {
        // Set up initial position
        engine.game_state = match &self.fen {
            Some(fen) => {
                if let Ok(pos) = Position::parse_from_fen(fen) {
                    GameState::from_position(pos)
                } else {
                    eprintln!("Invalid FEN: {}", fen);
                    return Ok(true);
                }
            }
            None => GameState::new(),
        };

        // Apply moves
        for move_str in &self.moves {
            let mv = if engine.game_state.current_turn == Color::White {
                PieceMove::from_uci(
                    &engine.game_state.current_position,
                    move_str,
                    engine.game_state.game_type,
                )
            } else {
                PieceMove::from_uci_inverted(
                    &engine.game_state.current_position,
                    move_str,
                    engine.game_state.game_type,
                )
            };

            match mv {
                Ok(mv) => {
                    let result = engine.game_state.apply_move(mv);

                    trace!("Applied move: {}", move_str);

                    if let Err(e) = result {
                        trace!("Error applying move {}: {}", move_str, e);
                        panic!("Error applying move {}: {}", move_str, e);
                    }
                }
                Err(e) => {
                    trace!("Error parsing move {}: {}", move_str, e);
                    panic!("Error parsing move {}: {}", move_str, e);
                }
            }
        }

        Ok(true)
    }
}

impl std::str::FromStr for PositionCommand {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() < 2 {
            return Err("Invalid position command".to_string());
        }

        let mut moves = Vec::new();
        let mut fen = None;

        match parts[1] {
            "startpos" => {
                // Handle moves after startpos
                if let Some(moves_idx) = parts.iter().position(|&x| x == "moves") {
                    moves = parts[moves_idx + 1..]
                        .iter()
                        .map(|&s| s.to_string())
                        .collect();
                }
            }
            "fen" => {
                // Find where the FEN string ends (either at "moves" or end of string)
                let moves_idx = parts.iter().position(|&x| x == "moves");
                let fen_end = moves_idx.unwrap_or(parts.len());
                fen = Some(parts[2..fen_end].join(" "));

                // Collect moves if present
                if let Some(idx) = moves_idx {
                    moves = parts[idx + 1..].iter().map(|&s| s.to_string()).collect();
                }
            }
            _ => return Err("Invalid position command format".to_string()),
        }

        Ok(PositionCommand { fen, moves })
    }
}
