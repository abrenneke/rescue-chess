use std::{
    io::{self, BufRead},
    str::FromStr,
};

use rescue_chess::uci::{commands::UciCommand, UciEngine};
use tracing::{error, trace};

fn get_next_log_file(base_name: &str) -> String {
    let mut counter = 1;
    loop {
        let file_name = format!("{}.{:03}", base_name, counter);
        if !std::path::Path::new(&file_name).exists() {
            return file_name;
        }
        counter += 1;
    }
}

fn main() -> io::Result<()> {
    // Initialize logging to uci_log.txt
    let log_file = get_next_log_file("uci_log.txt");
    tracing_subscriber::fmt()
        .with_writer(std::fs::File::create(log_file)?)
        .with_max_level(tracing::Level::TRACE)
        .with_ansi(false)
        .init();

    trace!("Starting UCI engine");

    let result = std::panic::catch_unwind(|| main_loop());

    if let Err(e) = result {
        let panic_information = if let Some(s) = e.downcast_ref::<String>() {
            s.clone()
        } else if let Some(s) = e.downcast_ref::<&str>() {
            s.to_string()
        } else {
            "Unknown panic".to_string()
        };

        error!("Panic: {}", panic_information);
    }

    Ok(())
}

fn main_loop() -> io::Result<()> {
    let stdin = io::stdin();
    let mut engine = UciEngine::new();
    let mut buffer = String::new();

    loop {
        buffer.clear();
        stdin.lock().read_line(&mut buffer)?;

        let cmd_str = buffer.trim();

        trace!("Received command: {}", cmd_str);

        match UciCommand::from_str(cmd_str) {
            Ok(cmd) => {
                if !engine.handle_command(cmd)? {
                    break;
                }
            }
            Err(e) => {
                error!("Error parsing command: {}", e);
                eprintln!("Error parsing command: {}", e)
            }
        }
    }

    trace!("Exiting UCI engine");

    Ok(())
}
