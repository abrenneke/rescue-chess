pub mod commands;

use commands::{CommandHandler, UciCommand};

use crate::search::game_state::GameState;
use std::io::{self};

pub struct UciEngine {
    pub game_state: GameState,
    pub stdout: Box<dyn io::Write>,
}

impl UciEngine {
    pub fn new() -> Self {
        Self {
            game_state: Default::default(),
            stdout: Box::new(io::stdout()),
        }
    }

    pub fn handle_command(&mut self, command: UciCommand) -> io::Result<bool> {
        match command {
            UciCommand::Uci(cmd) => cmd.execute(self),
            UciCommand::IsReady(cmd) => cmd.execute(self),
            UciCommand::UciNewGame => {
                self.game_state = GameState::new();
                Ok(true)
            }
            UciCommand::Position(cmd) => cmd.execute(self),
            UciCommand::Go(cmd) => cmd.execute(self),
            UciCommand::Stop => Ok(true), // TODO: Implement search stopping
            UciCommand::Quit => Ok(false),
            UciCommand::SetOption(cmd) => cmd.execute(self),
            UciCommand::Unknown(cmd) => {
                eprintln!("Unknown command: {}", cmd);
                Ok(true)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        uci::{commands::UciCommand, UciEngine},
        Pos,
    };
    use std::{
        io::{self, Write},
        sync::{Arc, Mutex},
    };

    // Helper struct to capture stdout
    struct CaptureStdout {
        buffer: Arc<Mutex<Vec<u8>>>,
    }

    impl CaptureStdout {
        fn new(buffer: Arc<Mutex<Vec<u8>>>) -> Self {
            Self { buffer }
        }

        fn as_string(&self) -> String {
            String::from_utf8_lossy(&self.buffer.lock().unwrap()).to_string()
        }
    }

    impl Write for CaptureStdout {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.buffer.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    // Helper function to create engine with captured stdout
    fn create_test_engine() -> (UciEngine, CaptureStdout) {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let engine = UciEngine {
            game_state: Default::default(),
            stdout: Box::new(CaptureStdout::new(buffer.clone())),
        };
        (engine, CaptureStdout::new(buffer))
    }

    #[test]
    fn test_uci_command() {
        let (mut engine, capture) = create_test_engine();

        // Send UCI command
        let cmd = "uci".parse::<UciCommand>().unwrap();
        engine.handle_command(cmd).unwrap();

        let output = capture.as_string();

        // Verify expected response
        assert!(output.contains("id name RescueChess"));
        assert!(output.contains("id author"));
        assert!(output.contains("option name Hash"));
        assert!(output.contains("option name UCI_Chess960"));
        assert!(output.contains("uciok"));
    }

    #[test]
    fn test_isready_command() {
        let (mut engine, capture) = create_test_engine();

        let cmd = "isready".parse::<UciCommand>().unwrap();
        engine.handle_command(cmd).unwrap();

        assert_eq!(capture.as_string().trim(), "readyok");
    }

    #[test]
    fn test_position_command() {
        let (mut engine, _capture) = create_test_engine();

        // Test starting position
        let cmd = "position startpos".parse::<UciCommand>().unwrap();
        engine.handle_command(cmd).unwrap();
        assert_eq!(engine.game_state.current_position, Default::default());

        // Test position with moves
        let cmd = "position startpos moves e2e4 e7e5"
            .parse::<UciCommand>()
            .unwrap();
        engine.handle_command(cmd).unwrap();

        // After e2e4 e7e5, black's king should be on e8
        assert!(engine
            .game_state
            .current_position
            .get_piece_at(Pos::from_algebraic("e8").unwrap())
            .is_some());
    }

    #[test]
    fn test_position_fen_command() {
        let (mut engine, _capture) = create_test_engine();

        // Test setting a specific position using FEN
        let cmd = "position fen rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2"
            .parse::<UciCommand>()
            .unwrap();
        engine.handle_command(cmd).unwrap();

        // Verify some aspects of the position
        assert!(engine
            .game_state
            .current_position
            .get_piece_at(Pos::from_algebraic("e4").unwrap())
            .is_some());
        assert!(engine
            .game_state
            .current_position
            .get_piece_at(Pos::from_algebraic("c5").unwrap())
            .is_some());
    }

    #[test]
    fn test_go_command() {
        let (mut engine, capture) = create_test_engine();

        // Set up a simple position
        let pos_cmd = "position startpos".parse::<UciCommand>().unwrap();
        engine.handle_command(pos_cmd).unwrap();

        // Send go command with depth 4
        let go_cmd = "go depth 4".parse::<UciCommand>().unwrap();
        engine.handle_command(go_cmd).unwrap();

        let output = capture.as_string();

        // Verify that a bestmove was output
        assert!(output.contains("bestmove"));

        // The move should be in algebraic notation (e.g., "e2e4")
        let last_line = output.lines().last().unwrap();
        assert!(last_line.starts_with("bestmove"));

        // Move should be 4 chars (like e2e4) or 5 chars (like e7e8q for promotion)
        let parts: Vec<&str> = last_line.split_whitespace().collect();
        dbg!(&parts);
        assert!(parts[1].len() >= 4);
    }

    #[test]
    fn test_unknown_command() {
        let (mut engine, _capture) = create_test_engine();

        let cmd = "invalid_command".parse::<UciCommand>().unwrap();
        let result = engine.handle_command(cmd).unwrap();

        // Engine should continue running even with unknown command
        assert!(result);
    }

    #[test]
    fn test_quit_command() {
        let (mut engine, _capture) = create_test_engine();

        let cmd = "quit".parse::<UciCommand>().unwrap();
        let result = engine.handle_command(cmd).unwrap();

        // Quit should return false to stop the engine
        assert!(!result);
    }

    #[test]
    fn test_setoption_command() {
        let (mut engine, _capture) = create_test_engine();

        // Test setting hash size
        let cmd = "setoption name Hash value 32"
            .parse::<UciCommand>()
            .unwrap();
        engine.handle_command(cmd).unwrap();

        // Test setting UCI_Chess960
        let cmd = "setoption name UCI_Chess960 value true"
            .parse::<UciCommand>()
            .unwrap();
        engine.handle_command(cmd).unwrap();
    }

    #[test]
    fn test_ucinewgame_command() {
        let (mut engine, _capture) = create_test_engine();

        // First set up some position
        let pos_cmd = "position startpos moves e2e4"
            .parse::<UciCommand>()
            .unwrap();
        engine.handle_command(pos_cmd).unwrap();

        // Send ucinewgame
        let cmd = "ucinewgame".parse::<UciCommand>().unwrap();
        engine.handle_command(cmd).unwrap();

        // Verify position was reset
        assert_eq!(engine.game_state.current_position, Default::default());
    }

    #[test]
    fn test_position_command_multiple_moves() {
        let (mut engine, _capture) = create_test_engine();

        // Test starting position
        let cmd = "position startpos moves e2e4 e7e5 d2d4"
            .parse::<UciCommand>()
            .unwrap();

        engine.handle_command(cmd).unwrap();

        // After e2e4 e7e5 d2d4, white should have a pawn on d4

        assert!(engine
            .game_state
            .current_position
            .get_piece_at(Pos::from_algebraic("d4").unwrap())
            .is_some());
    }

    #[test]
    fn test_position_command_multiple_moves_2() {
        let (mut engine, _capture) = create_test_engine();

        // Test starting position
        let cmd = "position startpos moves e2e4 g8f6 e4e5"
            .parse::<UciCommand>()
            .unwrap();

        engine.handle_command(cmd).unwrap();

        // After e2e4 g8f6 e4e5, white should have a pawn on e5

        assert!(engine
            .game_state
            .current_position
            .get_piece_at(Pos::from_algebraic("e5").unwrap().invert())
            .is_some());
    }
}
