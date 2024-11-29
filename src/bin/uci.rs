use std::{
    io::{self, BufRead},
    str::FromStr,
};

use rescue_chess::uci::{commands::UciCommand, UciEngine};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut engine = UciEngine::new();
    let mut buffer = String::new();

    loop {
        buffer.clear();
        stdin.lock().read_line(&mut buffer)?;

        match UciCommand::from_str(buffer.trim()) {
            Ok(cmd) => {
                if !engine.handle_command(cmd)? {
                    break;
                }
            }
            Err(e) => eprintln!("Error parsing command: {}", e),
        }
    }

    Ok(())
}
