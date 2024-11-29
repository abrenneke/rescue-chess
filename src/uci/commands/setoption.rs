use crate::uci::UciEngine;

use super::CommandHandler;

/// Represents a setoption command
#[derive(Debug)]
pub struct SetOptionCommand {
    pub name: String,
    pub value: Option<String>,
}

impl CommandHandler for SetOptionCommand {
    fn execute(&self, _engine: &mut UciEngine) -> std::io::Result<bool> {
        match self.name.as_str() {
            "Hash" => {
                if let Some(value) = &self.value {
                    if let Ok(size) = value.parse::<usize>() {
                        // TODO: Implement hash table resizing
                        println!("Hash size set to {} MB", size);
                    }
                }
            }
            "UCI_Chess960" => {
                if let Some(value) = &self.value {
                    if let Ok(enabled) = value.parse::<bool>() {
                        // TODO: Implement Chess960 mode
                        println!(
                            "Chess960 mode {}",
                            if enabled { "enabled" } else { "disabled" }
                        );
                    }
                }
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
