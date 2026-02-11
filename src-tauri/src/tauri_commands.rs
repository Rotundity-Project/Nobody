use crate::game_engine::GameEngine;
use crate::game_state::GameState;
use crate::numerical_system::{Action, Context, StatChange};
use crate::plot_engine::{PlayerAction, PlayerOption, PlotEngine, PlotState};
use crate::script::Script;
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
    engine.initialize_game(script).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn execute_player_action(
    action: PlayerAction,
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<String, String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;

    let mut game_state = engine.get_current_state().map_err(|e| e.to_string())?;
    let mut plot_state = engine.get_plot_state().map_err(|e| e.to_string())?;

    let plot_engine = PlotEngine::new();
    let context = Context {
        location: game_state.player.location.clone(),
        time_of_day: "day".to_string(),
        weather: None,
    };

    let mut action_result = plot_engine
        .process_player_action(
            &action,
            &game_state.player.stats,
            &plot_state.current_scene.available_options,
            &context,
        )
        .map_err(|e| e.to_string())?;

    if let Some(selected_option_id) = action.selected_option_id {
        if let Some(selected_option) = plot_state.current_scene.available_options.get(selected_option_id)
        {
            match &selected_option.action {
                Action::Cultivate => {
                    let old_power = game_state.player.stats.combat_power;
                    let gain = ((old_power as f32 * 0.03).round() as u64).max(1);
                    let new_power = old_power.saturating_add(gain);
                    game_state.player.stats.combat_power = new_power;
                    action_result.stat_changes.push(StatChange {
                        stat_name: "combat_power".to_string(),
                        old_value: old_power.to_string(),
                        new_value: new_power.to_string(),
                    });
                    action_result.description = format!(
                        "{} Combat power increased by {}.",
                        action_result.description, gain
                    );
                }
                Action::Breakthrough => {
                    if action_result.success
                        && game_state.player.stats.cultivation_realm.sub_level < 3
                    {
                        let old_sub = game_state.player.stats.cultivation_realm.sub_level;
                        game_state.player.stats.cultivation_realm.sub_level += 1;
                        game_state.player.stats.cultivation_realm.power_multiplier *= 1.2;
                        game_state.player.stats.update_combat_power();
                        action_result.stat_changes.push(StatChange {
                            stat_name: "realm_sub_level".to_string(),
                            old_value: old_sub.to_string(),
                            new_value: game_state
                                .player
                                .stats
                                .cultivation_realm
                                .sub_level
                                .to_string(),
                        });
                    }
                }
                Action::Rest | Action::Custom { .. } | Action::Combat { .. } => {}
            }
        }
    }

    game_state.game_time.advance_days(1);

    let mut plot_update = plot_engine.advance_plot(&plot_state, &action_result);

    let npc_reactions = engine
        .process_npc_reactions_for_events(&plot_update.triggered_events)
        .map_err(|e| e.to_string())?;
    if !npc_reactions.is_empty() {
        let reaction_line = format!(
            "NPC reactions: {}",
            npc_reactions
                .iter()
                .map(|d| format!("{} -> {}", d.npc_id, d.action))
                .collect::<Vec<String>>()
                .join(", ")
        );
        plot_update.plot_text = format!("{}\n{}", plot_update.plot_text, reaction_line);
    }

    plot_state.last_action_result = Some(action_result);
    plot_state.add_to_history(plot_update.plot_text.clone());
    plot_state.current_scene.description = plot_update.plot_text.clone();
    plot_state.current_scene.available_options = plot_engine
        .generate_player_options(&plot_state.current_scene, &game_state.player.stats);

    engine
        .update_current_state(game_state)
        .map_err(|e| e.to_string())?;
    engine
        .update_plot_state(plot_state)
        .map_err(|e| e.to_string())?;

    Ok(plot_update.plot_text)
}

#[tauri::command]
pub async fn get_game_state(engine: State<'_, Mutex<GameEngine>>) -> Result<GameState, String> {
    let engine = engine.lock().map_err(|e| e.to_string())?;
    engine.get_current_state().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_game(slot_id: u32, engine: State<'_, Mutex<GameEngine>>) -> Result<(), String> {
    let engine = engine.lock().map_err(|e| e.to_string())?;
    engine.save_game(slot_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_game(
    slot_id: u32,
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<GameState, String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    engine.load_game(slot_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_script(
    script_path: String,
    _engine: State<'_, Mutex<GameEngine>>,
) -> Result<Script, String> {
    use crate::script_manager::ScriptManager;

    let manager = ScriptManager::new();
    manager
        .load_custom_script(&script_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_random_script() -> Result<Script, String> {
    use crate::script_manager::ScriptManager;

    let manager = ScriptManager::new();
    manager
        .generate_random_script()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_player_options(
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<Vec<PlayerOption>, String> {
    let engine = engine.lock().map_err(|e| e.to_string())?;
    let plot_state = engine.get_plot_state().map_err(|e| e.to_string())?;
    Ok(plot_state.current_scene.available_options)
}

#[tauri::command]
pub async fn initialize_plot(
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<PlotState, String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    engine.initialize_plot().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_plot_state(
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<PlotState, String> {
    let engine = engine.lock().map_err(|e| e.to_string())?;
    engine.get_plot_state().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CultivationRealm, Element, Grade, SpiritualRoot};
    use crate::script::{InitialState, Location, ScriptType, WorldSetting};

    fn create_test_script() -> Script {
        let mut world_setting = WorldSetting::new();
        world_setting.cultivation_realms = vec![
            CultivationRealm::new("Qi Condensation".to_string(), 1, 0, 1.0),
            CultivationRealm::new("Foundation Establishment".to_string(), 2, 0, 2.0),
        ];
        world_setting.locations = vec![Location {
            id: "sect".to_string(),
            name: "Azure Cloud Sect".to_string(),
            description: "A peaceful cultivation sect".to_string(),
            spiritual_energy: 1.0,
        }];

        let initial_state = InitialState {
            player_name: "Test Player".to_string(),
            player_spiritual_root: SpiritualRoot {
                element: Element::Fire,
                grade: Grade::Heavenly,
                affinity: 0.9,
            },
            starting_location: "sect".to_string(),
            starting_age: 16,
        };

        Script::new(
            "test".to_string(),
            "Test Script".to_string(),
            ScriptType::Custom,
            world_setting,
            initial_state,
        )
    }

    #[test]
    fn test_command_logic_initialize_game() {
        let mut engine = GameEngine::new();
        let script = create_test_script();

        let result = engine.initialize_game(script);

        assert!(result.is_ok());
        let game_state = result.unwrap();
        assert_eq!(game_state.player.name, "Test Player");
        assert_eq!(game_state.player.location, "sect");
    }

    #[test]
    fn test_command_logic_get_game_state_before_initialization() {
        let engine = GameEngine::new();
        let result = engine.get_current_state();

        assert!(result.is_err());
    }

    #[test]
    fn test_error_response_conversion() {
        let error = anyhow::anyhow!("Test error");
        let response: ErrorResponse = error.into();

        assert_eq!(response.error, "Test error");
    }
}
