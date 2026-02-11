pub mod game_engine;
pub mod game_state;
pub mod llm_service;
pub mod memory_manager;
pub mod models;
pub mod npc;
pub mod npc_engine;
pub mod numerical_system;
pub mod plot_engine;
pub mod prompt_builder;
pub mod response_validator;
pub mod save_load;
pub mod script;
pub mod script_manager;
pub mod tauri_commands;

use game_engine::GameEngine;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化游戏引擎
    let game_engine = Mutex::new(GameEngine::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(game_engine)
        .invoke_handler(tauri::generate_handler![
            tauri_commands::initialize_game,
            tauri_commands::execute_player_action,
            tauri_commands::get_game_state,
            tauri_commands::save_game,
            tauri_commands::load_game,
            tauri_commands::load_script,
            tauri_commands::generate_random_script,
            tauri_commands::get_player_options,
            tauri_commands::initialize_plot,
            tauri_commands::get_plot_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
