use tracing::trace;

use super::CommandHandler;
use crate::uci::UciEngine;
use std::io::Write;

#[derive(Debug)]
pub struct IsReadyCommand;

impl CommandHandler for IsReadyCommand {
    fn execute(&self, engine: &mut UciEngine) -> std::io::Result<bool> {
        let mut stdout = engine.stdout.lock().unwrap();

        trace!("Sending readyok");
        writeln!(stdout, "readyok")?;
        stdout.flush()?;
        Ok(true)
    }
}
