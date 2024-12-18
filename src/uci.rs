pub mod commands;

use commands::{CommandHandler, UciCommand};
use tracing::error;

use crate::search::game_state::GameState;
use std::{
    io::{self},
    sync::{Arc, Mutex},
};

pub struct UciEngine {
    pub game_state: Arc<Mutex<GameState>>,
    pub stdout: Arc<Mutex<Box<dyn io::Write + Send>>>,
}

impl UciEngine {
    pub fn new() -> Self {
        Self {
            game_state: Arc::new(Mutex::new(GameState::default())),
            stdout: Arc::new(Mutex::new(Box::new(io::stdout()))),
        }
    }

    pub fn handle_command(&mut self, command: UciCommand) -> io::Result<bool> {
        match command {
            UciCommand::Uci(cmd) => cmd.execute(self),
            UciCommand::IsReady(cmd) => cmd.execute(self),
            UciCommand::UciNewGame => {
                self.game_state = Arc::new(Mutex::new(GameState::new()));
                Ok(true)
            }
            UciCommand::Position(cmd) => cmd.execute(self),
            UciCommand::Go(cmd) => cmd.execute(self),
            UciCommand::Stop => Ok(true), // TODO: Implement search stopping
            UciCommand::Quit => Ok(false),
            UciCommand::SetOption(cmd) => cmd.execute(self),
            UciCommand::Unknown(cmd) => {
                if cmd.trim().is_empty() {
                    return Ok(true);
                }

                error!("Unknown command: {}", cmd);
                // eprintln!("Unknown command: {}", cmd);
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
            stdout: Arc::new(Mutex::new(Box::new(CaptureStdout::new(buffer.clone())))),
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
        assert_eq!(
            engine.game_state.lock().unwrap().current_position,
            Default::default()
        );

        // Test position with moves
        let cmd = "position startpos moves e2e4 e7e5"
            .parse::<UciCommand>()
            .unwrap();
        engine.handle_command(cmd).unwrap();

        // After e2e4 e7e5, black's king should be on e8
        assert!(engine
            .game_state
            .lock()
            .unwrap()
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
            .lock()
            .unwrap()
            .current_position
            .get_piece_at(Pos::from_algebraic("e4").unwrap())
            .is_some());
        assert!(engine
            .game_state
            .lock()
            .unwrap()
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
        assert_eq!(
            engine.game_state.lock().unwrap().current_position,
            Default::default()
        );
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
            .lock()
            .unwrap()
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
            .lock()
            .unwrap()
            .current_position
            .get_piece_at(Pos::from_algebraic("e5").unwrap().invert())
            .is_some());
    }

    #[test]
    fn castling() {
        let (mut engine, _capture) = create_test_engine();

        // Test starting position
        let cmd = "position startpos moves e2e4 b8c6 g1f3 g8f6 b1c3 d7d5 e4e5 f6e4 f1d3 c8f5 d1e2 e4c3 d2c3 f5d3 c2d3 e7e6 f3g5 f8e7 e2h5 g7g6 h5h3 e7g5 f2f4 g5h4 g2g3 h4e7 f4f5 c6e5 e1g1"
            .parse::<UciCommand>()
            .unwrap();

        engine.handle_command(cmd).unwrap();
    }

    #[test]
    fn castling_2() {
        let (mut engine, _capture) = create_test_engine();

        let cmd = "position startpos moves d2d4 d7d5 b1c3 g8f6 c1f4 c8f5 c3b5 b8a6 g1f3 f6h5 f4d2 f5c2 d1c2 d8d7 e2e3 d7e6 b5c7 a6c7 c2c7 a7a6 c7b7 e6c8 f1a6 c8b7 a6b7 a8a7 b7d5 a7c7 f3e5 e7e6 d5c6 c7c6 e5c6 f8d6 e3e4 f7f5 e4e5 h5f6 e5d6 e8g8 c6d8 f8d8"
            .parse::<UciCommand>()
            .unwrap();

        engine.handle_command(cmd).unwrap();
    }

    #[test]
    fn en_passant() {
        let (mut engine, _capture) = create_test_engine();

        let cmd = "position startpos moves d2d4 d7d5 b1c3 g8f6 g1f3 b8c6 c1f4 e7e6 f3e5 f6e4 e5c6 b7c6 c3e4 d5e4 f4e5 f8d6 e5g7 d8g5 g7h8 g5g8 h8e5 a8b8 a1b1 g8g5 f2f4 e4f3".parse::<UciCommand>().unwrap();

        engine.handle_command(cmd).unwrap();

        let game_state = engine.game_state.lock().unwrap();
        let pos = &game_state.current_position;

        println!("{}", pos.to_fen());
    }

    #[test]
    fn cannot_castle() {
        let (mut engine, _capture) = create_test_engine();

        let cmd = "position startpos moves e2e4 e7e5 g1f3 b8c6 b1c3 g8f6 f1b5 c6d4 f3e5 d8e7 c3d5 f6d5 c2c3 d4b5 e4d5 e7e5 d1e2 e5e2 e1e2 b5d6 h1e1 f8e7 e2f1 d6c4 b2b3 c4b6 d5d6 c7d6 c1a3 e8d8 e1e7 d8e7 a1e1 e7d8 a3d6 h7h5 d6e7 d8c7 e7g5 c7c6 a2a4 d7d5 e1e7 c8e6 g5f4 b6d7 b3b4".parse::<UciCommand>().unwrap();

        engine.handle_command(cmd).unwrap();

        let mut game_state = engine.game_state.lock().unwrap();

        game_state.search_depth = 10;

        let (best_move, _) = game_state.search_and_apply().unwrap();

        println!("Best move: {}", best_move);

        assert!(best_move.to_string() != "Kb1");
    }
}
