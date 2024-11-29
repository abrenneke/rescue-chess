use super::CommandHandler;
use crate::uci::UciEngine;
use std::io::Write;

#[derive(Debug)]
pub struct UciCommand;

impl CommandHandler for UciCommand {
    fn execute(&self, engine: &mut UciEngine) -> std::io::Result<bool> {
        writeln!(engine.stdout, "id name RescueChess")?;
        writeln!(engine.stdout, "id author Your Name")?;

        // Send options
        writeln!(
            engine.stdout,
            "option name Hash type spin default 16 min 1 max 1024"
        )?;
        writeln!(
            engine.stdout,
            "option name UCI_Chess960 type check default false"
        )?;

        writeln!(engine.stdout, "uciok")?;
        engine.stdout.flush()?;
        Ok(true)
    }
}
