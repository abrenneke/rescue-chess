use tracing::trace;

use crate::uci::UciEngine;

use super::CommandHandler;

/// Represents a setoption command
#[derive(Debug)]
pub struct SetOptionCommand {
    pub name: String,
    pub value: Option<String>,
}

impl CommandHandler for SetOptionCommand {
    fn execute(&self, engine: &mut UciEngine) -> std::io::Result<bool> {
        let mut game_state = engine.game_state.lock().unwrap();

        trace!("Setting option: {} = {:?}", self.name, self.value);

        match self.name.as_str() {
            "EnableLMR" => {
                game_state.features.enable_lmr = self.value.as_deref() == Some("true");
            }
            "EnableTranspositionTable" => {
                game_state.features.enable_transposition_table =
                    self.value.as_deref() == Some("true");
            }
            "EnableWindowSearch" => {
                game_state.features.enable_window_search = self.value.as_deref() == Some("true");
            }
            "EnableKillerMoves" => {
                game_state.features.enable_killer_moves = self.value.as_deref() == Some("true");
            }
            // Add other options as needed
            _ => eprintln!("Unknown option: {}", self.name),
        }
        Ok(true)
    }
}

impl std::str::FromStr for SetOptionCommand {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() < 3 || parts[1] != "name" {
            return Err("Invalid setoption command".to_string());
        }

        let value_idx = parts.iter().position(|&x| x == "value");
        let name = match value_idx {
            Some(idx) => parts[2..idx].join(" "),
            None => parts[2..].join(" "),
        };

        let value = value_idx.map(|idx| parts[idx + 1..].join(" "));

        Ok(SetOptionCommand { name, value })
    }
}
