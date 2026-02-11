use crate::game_engine::GameEngine;
use crate::game_state::GameState;
use crate::plot_engine::{PlayerAction, PlayerOption, PlotState};
use crate::script::Script;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl From<anyhow::Error> for ErrorResponse {
    fn from(err: anyhow::Error) -> Self {
        ErrorResponse {
            error: err.to_string(),
        }
    }
}

#[tauri::command]
pub async fn initialize_game(
    script: Script,
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<GameState, String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    
    engine
        .initialize_game(script)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn execute_player_action(
    action: PlayerAction,
    _engine: State<'_, Mutex<GameEngine>>,
) -> Result<String, String> {
    Ok(format!("Action received: {:?}", action.action_type))
}

#[tauri::command]
pub async fn get_game_state(
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<GameState, String> {
    let engine = engine.lock().map_err(|e| e.to_string())?;
    
    engine
        .get_current_state()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_game(
    slot_id: u32,
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<(), String> {
    let engine = engine.lock().map_err(|e| e.to_string())?;
    
    engine
        .save_game(slot_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_game(
    slot_id: u32,
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<GameState, String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    
    engine
        .load_game(slot_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_script(
    _file_path: String,
    _engine: State<'_, Mutex<GameEngine>>,
) -> Result<Script, String> {
    Err("Script loading will be implemented in future version".to_string())
}

#[tauri::command]
pub async fn get_player_options(
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<Vec<PlayerOption>, String> {
    let engine = engine.lock().map_err(|e| e.to_string())?;
    
    let plot_state = engine
        .get_plot_state()
        .map_err(|e| e.to_string())?;
    
    Ok(plot_state.current_scene.available_options)
}

#[tauri::command]
pub async fn initialize_plot(
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<PlotState, String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    
    engine
        .initialize_plot()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_plot_state(
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<PlotState, String> {
    let engine = engine.lock().map_err(|e| e.to_string())?;
    
    engine
        .get_plot_state()
        .map_err(|e| e.to_string())
}
