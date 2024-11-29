pub mod go;
pub mod isready;
pub mod position;
pub mod setoption;
pub mod uci;

pub use go::GoCommand;
use isready::IsReadyCommand;
pub use position::PositionCommand;
pub use setoption::SetOptionCommand;

use super::UciEngine;

/// Represents UCI commands that can be sent from GUI to engine
#[derive(Debug)]
pub enum UciCommand {
    Uci(uci::UciCommand),
    IsReady(IsReadyCommand),
    UciNewGame,
    Position(PositionCommand),
    Go(GoCommand),
    Stop,
    Quit,
    SetOption(SetOptionCommand),
    Unknown(String),
}

pub trait CommandHandler {
    fn execute(&self, engine: &mut UciEngine) -> std::io::Result<bool>;
}

// Update FromStr implementation for UciCommand:
impl std::str::FromStr for UciCommand {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(UciCommand::Unknown(s.to_string()));
        }

        match parts[0] {
            "uci" => Ok(UciCommand::Uci(uci::UciCommand)),
            "isready" => Ok(UciCommand::IsReady(IsReadyCommand)),
            "ucinewgame" => Ok(UciCommand::UciNewGame),
            "stop" => Ok(UciCommand::Stop),
            "quit" => Ok(UciCommand::Quit),
            "position" => Ok(UciCommand::Position(PositionCommand::from_str(s)?)),
            "go" => Ok(UciCommand::Go(GoCommand::from_str(s)?)),
            "setoption" => Ok(UciCommand::SetOption(SetOptionCommand::from_str(s)?)),
            _ => Ok(UciCommand::Unknown(s.to_string())),
        }
    }
}
