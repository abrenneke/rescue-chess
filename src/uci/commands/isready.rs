use super::CommandHandler;
use crate::uci::UciEngine;
use std::io::Write;

#[derive(Debug)]
pub struct IsReadyCommand;

impl CommandHandler for IsReadyCommand {
    fn execute(&self, engine: &mut UciEngine) -> std::io::Result<bool> {
        writeln!(engine.stdout, "readyok")?;
        engine.stdout.flush()?;
        Ok(true)
    }
}
