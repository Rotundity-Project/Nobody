pub mod game_engine;
pub mod game_state;
pub mod event_log;
pub mod app_error;
pub mod combat_style_rules;
pub mod context_builder;
pub mod entity_store;
pub mod entity_types;
pub mod entity_validator;
pub mod llm_runtime_config;
pub mod llm_service;
pub mod llm_bootstrap;
pub mod memory_manager;
pub mod memory_layers;
pub mod models;
pub mod npc;
pub mod npc_engine;
pub mod novel_generator;
pub mod novel_parser;
pub mod numeric_guard;
pub mod numerical_system;
pub mod plot_engine;
pub mod plot_consistency;
pub mod prompt_builder;
pub mod response_validator;
pub mod save_load;
pub mod script;
pub mod script_manager;
pub mod state_patch_validator;
pub mod tauri_commands;
pub mod travel_rules;
pub mod world_registry;

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
            tauri_commands::travel_to_location,
            tauri_commands::get_reachable_locations,
            tauri_commands::get_map_overview,
            tauri_commands::get_game_state,
            tauri_commands::save_game,
            tauri_commands::load_game,
            tauri_commands::list_save_slots,
            tauri_commands::migrate_all_saves,
            tauri_commands::load_script,
            tauri_commands::generate_random_script,
            tauri_commands::parse_novel_characters,
            tauri_commands::load_existing_novel,
            tauri_commands::get_player_options,
            tauri_commands::initialize_plot,
            tauri_commands::get_plot_state,
            tauri_commands::update_plot_settings,
            tauri_commands::get_consistency_policy,
            tauri_commands::update_consistency_policy,
            tauri_commands::reset_consistency_policy,
            tauri_commands::generate_novel,
            tauri_commands::export_novel,
            tauri_commands::set_llm_config,
            tauri_commands::clear_llm_config,
            tauri_commands::get_llm_config_status,
            tauri_commands::test_llm_connection,
            tauri_commands::generate_entity_candidates,
            tauri_commands::commit_entities,
            tauri_commands::query_entities,
            tauri_commands::build_context_bundle_command,
            tauri_commands::get_world_registry,
            tauri_commands::apply_world_registry_patch,
            tauri_commands::summarize_generation_diagnostics,
            tauri_commands::summarize_generation_failures,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests;
