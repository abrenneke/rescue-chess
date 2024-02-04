// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod global_state;

use global_state::GlobalState;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(GlobalState::default())
        .invoke_handler(tauri::generate_handler![
            commands::get_valid_positions_for,
            commands::reset,
            commands::get_position_fen,
            commands::move_piece,
            commands::get_black_move
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
