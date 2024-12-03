use super::CommandHandler;
use crate::uci::UciEngine;
use std::io::Write;

#[derive(Debug)]
pub struct UciCommand;

impl CommandHandler for UciCommand {
    fn execute(&self, engine: &mut UciEngine) -> std::io::Result<bool> {
        let mut stdout = engine.stdout.lock().unwrap();

        writeln!(stdout, "id name Rescue")?;
        writeln!(stdout, "id author Andy Brenneke")?;

        // Send options
        writeln!(
            stdout,
            "option name Hash type spin default 64 min 1 max 16384"
        )?;
        writeln!(
            stdout,
            "option name EnableTranspositionTable type check default true"
        )?;
        writeln!(stdout, "option name EnableLMR type check default true")?;
        writeln!(
            stdout,
            "option name EnableWindowSearch type check default true"
        )?;

        writeln!(stdout, "uciok")?;
        stdout.flush()?;
        Ok(true)
    }
}
