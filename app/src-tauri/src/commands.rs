use std::thread;

use rescue_chess::{search::negamax_hashing, Color, PieceMove};
use tauri::{command, Manager, State};

use crate::global_state::GlobalState;

#[command]
pub fn get_valid_positions_for(
    x: u8,
    y: u8,
    state: State<GlobalState>,
) -> Result<Vec<PieceMove>, String> {
    let gs = state.lock().unwrap();

    let all_moves = gs
        .position
        .get_all_legal_moves()
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
pub fn move_piece(
    from_x: u8,
    from_y: u8,
    to_x: u8,
    to_y: u8,
    state: State<GlobalState>,
) -> Result<(), String> {
    let mut gs = state.lock().unwrap();

    let from = (from_x, from_y).into();
    let to = (to_x, to_y).into();

    match gs.position.get_piece_at(from) {
        Some(piece) => match piece.color {
            Color::White => {
                let all_moves = gs
                    .position
                    .get_all_legal_moves()
                    .map_err(|e| e.to_string())?;

                let matching_move = all_moves
                    .into_iter()
                    .find(|m| m.from == from && m.to == to)
                    .ok_or_else(|| "Invalid move".to_string())?;

                gs.position
                    .apply_move(matching_move)
                    .map_err(|e| e.to_string())?;
            }
            Color::Black => {
                // Invert the position, apply the move, and invert back
                let mut inverted_position = gs.position.inverted();
                let from = from.invert();
                let to = to.invert();

                let all_moves = inverted_position
                    .get_all_legal_moves()
                    .map_err(|e| e.to_string())?;

                let matching_move = all_moves
                    .into_iter()
                    .find(|m| m.from == from && m.to == to)
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

#[command]
pub fn get_black_move(state: State<GlobalState>, app: tauri::AppHandle) -> Result<(), String> {
    let gs = state.lock().unwrap();
    let from_black = gs.position.inverted();

    thread::spawn(move || -> () {
        let (mv, _) = negamax_hashing::search(&from_black, 4);
        let mv = match mv {
            Some(mv) => mv,
            None => panic!("No move found"),
        };

        let move_from_whites_perspective = mv.inverted();

        app.emit("black_move", move_from_whites_perspective)
            .unwrap();
    });

    Ok(())
}
