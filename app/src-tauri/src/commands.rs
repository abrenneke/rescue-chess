use rescue_chess::PieceMove;
use tauri::{command, State};

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

    Ok(())
}
