use serde::Serialize;
use std::thread;

use rescue_chess::{
    piece_move::GameType,
    search::{
        alpha_beta::{self, SearchParams},
        search_results::{SearchResults, SearchState},
    },
    Color, PieceMove,
};
use tauri::{command, Manager, State};

use crate::global_state::GlobalState;

const GAME_TYPE: GameType = GameType::Rescue;

#[command]
pub fn get_valid_positions_for(
    x: u8,
    y: u8,
    state: State<GlobalState>,
) -> Result<Vec<PieceMove>, String> {
    let gs = state.lock().unwrap();

    let all_moves = gs
        .position
        .get_all_legal_moves(GAME_TYPE)
        .map_err(|e| e.to_string())?;

    let moves_for_piece = all_moves
        .iter()
        .filter(|m| m.from == (x, y).into())
        .cloned()
        .collect();

    Ok(moves_for_piece)
}

#[command]
pub fn reset(state: State<GlobalState>) {
    let mut gs = state.lock().unwrap();

    gs.reset();
}

#[command]
pub fn get_position_fen(state: State<GlobalState>) -> String {
    let gs = state.lock().unwrap();

    gs.position.to_fen()
}

#[command]
pub fn move_piece(mv: PieceMove, state: State<GlobalState>) -> Result<(), String> {
    let mut gs = state.lock().unwrap();

    match gs.position.get_piece_at(mv.from) {
        Some(piece) => match piece.color {
            Color::White => {
                let all_moves = gs
                    .position
                    .get_all_legal_moves(GAME_TYPE)
                    .map_err(|e| e.to_string())?;

                let matching_move = all_moves
                    .into_iter()
                    .find(|m| *m == mv)
                    .ok_or_else(|| "Invalid move".to_string())?;

                gs.position
                    .apply_move(matching_move)
                    .map_err(|e| e.to_string())?;
            }
            Color::Black => {
                // Invert the position, apply the move, and invert back
                let mut inverted_position = gs.position.inverted();
                let mv = mv.inverted();

                let all_moves = inverted_position
                    .get_all_legal_moves(GAME_TYPE)
                    .map_err(|e| e.to_string())?;

                let matching_move = all_moves
                    .into_iter()
                    .find(|m| *m == mv)
                    .ok_or_else(|| "Invalid move".to_string())?;

                inverted_position
                    .apply_move(matching_move)
                    .map_err(|e| e.to_string())?;

                gs.position = inverted_position.inverted();
            }
        },
        None => return Err("No piece at that position".to_string()),
    }

    Ok(())
}

#[derive(Clone, Serialize)]
struct BlackMoveResponse {
    results: SearchResults,
    move_from_whites_perspective: PieceMove,
}

#[derive(Clone, Serialize)]
struct WhiteMoveResponse {
    results: SearchResults,
    move_from_whites_perspective: PieceMove,
}

#[command]
pub fn get_black_move(state: State<GlobalState>, app: tauri::AppHandle) -> Result<(), String> {
    let gs = state.lock().unwrap();
    let transposition_table = gs.transposition_table.clone();
    let depth = gs.depth;

    let from_black = gs.position.inverted();

    println!("Getting black move");
    println!(
        "Position\n{}",
        from_black.to_board_string_with_rank_file_holding()
    );

    thread::spawn(move || -> () {
        let mut transposition_table = transposition_table.lock().unwrap();
        let mut state = SearchState::new(&mut transposition_table);

        let params = SearchParams {
            depth,
            game_type: GAME_TYPE,
            ..Default::default()
        };

        let results = alpha_beta::search(&from_black, &mut state, params, 0);

        match results {
            Ok(results) => {
                let move_from_whites_perspective = results.best_move.unwrap().inverted();

                app.emit(
                    "black_move",
                    BlackMoveResponse {
                        results,
                        move_from_whites_perspective,
                    },
                )
                .unwrap();
            }
            Err(_) => {
                eprintln!("Error getting black move");
            }
        }
    });

    Ok(())
}

#[command]
pub fn get_white_move(state: State<GlobalState>, app: tauri::AppHandle) -> Result<(), String> {
    let gs = state.lock().unwrap();
    let from_white = gs.position.clone();

    println!("Getting white move");
    println!(
        "Position\n{}",
        gs.position.to_board_string_with_rank_file_holding()
    );

    let transposition_table = gs.transposition_table.clone();
    let depth = gs.depth;

    thread::spawn(move || -> () {
        let mut transposition_table = transposition_table.lock().unwrap();
        let mut state = SearchState::new(&mut transposition_table);

        let params = SearchParams {
            depth,
            game_type: GAME_TYPE,
            ..Default::default()
        };

        let results = alpha_beta::search(&from_white, &mut state, params, 0);

        match results {
            Ok(results) => {
                let move_from_whites_perspective = results.best_move.unwrap();

                app.emit(
                    "white_move",
                    WhiteMoveResponse {
                        results,
                        move_from_whites_perspective,
                    },
                )
                .unwrap();
            }
            Err(_) => {
                eprintln!("Error getting white move");
            }
        }
    });

    Ok(())
}
