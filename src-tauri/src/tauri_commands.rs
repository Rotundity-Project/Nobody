use crate::game_engine::GameEngine;
use crate::game_state::GameState;
use crate::event_log::EventImportance;
use crate::llm_runtime_config::{
    clear_runtime_llm_config, get_llm_config_status as runtime_llm_config_status,
    resolve_llm_config, set_runtime_llm_config, LLMConfigStatus,
};
use crate::llm_service::{LLMConfig, LLMRequest, LLMService};
use crate::novel_generator::{Novel, NovelGenerator};
use crate::numerical_system::{Action, Context, StatChange};
use crate::plot_engine::{PlayerAction, PlayerOption, PlotEngine, PlotSettings, PlotState};
use crate::save_load::SaveInfo;
use crate::script::Script;
use crate::app_error::AppError;
use serde::{Deserialize, Serialize};
use std::path::Path;
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

fn map_error(context: &str, err: impl Into<AppError>) -> String {
    err.into().with_context(context).to_string()
}

fn validate_slot_id(slot_id: u32) -> Result<(), AppError> {
    if (1..=99).contains(&slot_id) {
        Ok(())
    } else {
        Err(AppError::new(
            crate::app_error::AppErrorKind::InvalidInput,
            format!("存档槽位必须在 1-99 之间，当前为 {}", slot_id),
        ))
    }
}

fn validate_file_path(path: &str, allowed_exts: &[&str]) -> Result<(), AppError> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(AppError::new(
            crate::app_error::AppErrorKind::NotFound,
            format!("文件不存在: {}", path),
        ));
    }
    if !p.is_file() {
        return Err(AppError::new(
            crate::app_error::AppErrorKind::InvalidInput,
            format!("路径不是文件: {}", path),
        ));
    }
    if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
        if allowed_exts.iter().any(|allowed| ext.eq_ignore_ascii_case(allowed)) {
            return Ok(());
        }
    }
    Err(AppError::new(
        crate::app_error::AppErrorKind::InvalidInput,
        format!("文件格式不支持: {}", path),
    ))
}

fn validate_endpoint(endpoint: &str) -> Result<(), AppError> {
    let trimmed = endpoint.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        Ok(())
    } else {
        Err(AppError::new(
            crate::app_error::AppErrorKind::InvalidInput,
            "LLM endpoint 必须以 http:// 或 https:// 开头",
        ))
    }
}

fn validate_non_empty(value: &str, label: &str) -> Result<(), AppError> {
    if value.trim().is_empty() {
        Err(AppError::new(
            crate::app_error::AppErrorKind::InvalidInput,
            format!("{}不能为空", label),
        ))
    } else {
        Ok(())
    }
}

fn validate_output_path(path: &str, allowed_exts: &[&str]) -> Result<(), AppError> {
    let p = Path::new(path);
    if let Some(parent) = p.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            return Err(AppError::new(
                crate::app_error::AppErrorKind::NotFound,
                format!("输出目录不存在: {}", parent.display()),
            ));
        }
    }
    if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
        if allowed_exts.iter().any(|allowed| ext.eq_ignore_ascii_case(allowed)) {
            return Ok(());
        }
    }
    Err(AppError::new(
        crate::app_error::AppErrorKind::InvalidInput,
        format!("输出文件格式不支持: {}", path),
    ))
}

fn validate_llm_config_input(input: &LLMConfigInput) -> Result<(), AppError> {
    validate_endpoint(&input.endpoint)?;
    validate_non_empty(&input.model, "模型名称")?;
    if input.max_tokens == 0 || input.max_tokens > 8192 {
        return Err(AppError::new(
            crate::app_error::AppErrorKind::InvalidInput,
            "max_tokens 必须在 1-8192 之间",
        ));
    }
    if !(0.0..=2.0).contains(&input.temperature) {
        return Err(AppError::new(
            crate::app_error::AppErrorKind::InvalidInput,
            "temperature 必须在 0-2 之间",
        ));
    }
    if input.api_key.trim().is_empty() {
        let endpoint = input.endpoint.to_lowercase();
        let local = endpoint.contains("localhost") || endpoint.contains("127.0.0.1");
        if !local {
            return Err(AppError::new(
                crate::app_error::AppErrorKind::InvalidInput,
                "API Key 不能为空",
            ));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LLMConfigInput {
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[tauri::command]
pub async fn set_llm_config(input: LLMConfigInput) -> Result<String, String> {
    validate_llm_config_input(&input).map_err(|e| map_error("LLM 配置校验失败", e))?;
    let config = LLMConfig {
        endpoint: input.endpoint,
        api_key: input.api_key,
        model: input.model,
        max_tokens: input.max_tokens,
        temperature: input.temperature,
    };
    LLMService::new(config.clone()).map_err(|e| map_error("LLM 配置校验失败", e))?;
    set_runtime_llm_config(config);
    Ok("LLM 配置已更新".to_string())
}

#[tauri::command]
pub async fn clear_llm_config() -> Result<String, String> {
    clear_runtime_llm_config();
    Ok("已清除运行时 LLM 配置".to_string())
}

#[tauri::command]
pub async fn get_llm_config_status() -> Result<LLMConfigStatus, String> {
    Ok(runtime_llm_config_status())
}

#[tauri::command]
pub async fn test_llm_connection() -> Result<String, String> {
    let cfg = resolve_llm_config().ok_or_else(|| "未检测到 LLM 配置".to_string())?;
    let service = LLMService::new(cfg).map_err(|e| e.to_string())?;
    let response = service
        .generate(LLMRequest {
            prompt: "请回复：连接成功".to_string(),
            max_tokens: Some(32),
            temperature: Some(0.1),
        })
        .await
        .map_err(|e| e.to_string())?;
    Ok(response.text)
}

#[tauri::command]
pub async fn initialize_game(
    script: Script,
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<GameState, String> {
    let mut engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    engine.initialize_game(script).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn execute_player_action(
    action: PlayerAction,
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<String, String> {
    let (mut game_state, mut plot_state) = {
        let engine = match engine.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let game_state = engine.get_current_state().map_err(|e| e.to_string())?;
        let plot_state = engine.get_plot_state().map_err(|e| e.to_string())?;
        (game_state, plot_state)
    };

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
                        "{} 战力提升了 {}。",
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
    let timestamp = u64::from(game_state.game_time.total_days);

    let plot_update = plot_engine
        .advance_plot_async(&plot_state, &action_result)
        .await;

    let log_entry = if let Some(selected_option_id) = action.selected_option_id {
        if let Some(selected_option) = plot_state.current_scene.available_options.get(selected_option_id) {
            match &selected_option.action {
                Action::Combat { .. } => Some((
                    "combat",
                    format!("Player engaged in combat: {}", selected_option.description),
                    EventImportance::Important,
                )),
                Action::Breakthrough => Some((
                    "breakthrough_attempt",
                    format!("Player attempted breakthrough: {}", selected_option.description),
                    EventImportance::Important,
                )),
                Action::Custom { .. } | Action::Cultivate | Action::Rest => Some((
                    "player_action",
                    selected_option.description.clone(),
                    EventImportance::Normal,
                )),
            }
        } else {
            None
        }
    } else if matches!(action.action_type, crate::plot_engine::ActionType::FreeText) {
        Some((
            "player_free_text",
            action.content.clone(),
            EventImportance::Normal,
        ))
    } else {
        None
    };

    plot_state.last_action_result = Some(action_result);
    plot_state.append_segment(plot_update.plot_text.clone());

    if let Some(title) = plot_update.chapter_title.clone() {
        if !title.trim().is_empty() {
            plot_state.current_chapter.title = title.trim().to_string();
            plot_state.current_scene.name = plot_state.current_chapter.title.clone();
        }
    }

    if plot_update.is_waiting_for_input {
        plot_state.current_chapter.interaction_count = plot_state
            .current_chapter
            .interaction_count
            .saturating_add(1);
    }

    if plot_update.chapter_end {
        plot_state.finalize_chapter(plot_update.chapter_title, plot_update.chapter_summary);
    }

    plot_state.last_generation_diagnostics = plot_update.generation_diagnostics.clone();

    // 用最新段落更新场景描述，避免选项生成长期绑定旧描述导致“选项不变”。
    if !plot_update.plot_text.trim().is_empty() {
        plot_state.current_scene.description = plot_update.plot_text.trim().to_string();
    }

    let previous_options = plot_state.current_scene.available_options.clone();

    let mut option_source: Option<String> = None;

    if plot_update.is_waiting_for_input {
        if !plot_update.available_options.is_empty() {
            plot_state.current_scene.available_options = plot_update.available_options;
            option_source = Some("llm_structured".to_string());
        } else {
            let llm_regenerated = plot_engine.generate_player_options_with_llm(
                &plot_state.current_scene,
                &game_state.player.stats,
            );
            let mut regenerated_options = if let Some(options) = llm_regenerated {
                option_source = Some("llm_regenerated".to_string());
                options
            } else {
                option_source = Some("rule_fallback".to_string());
                plot_engine
                    .generate_player_options(&plot_state.current_scene, &game_state.player.stats)
            };

            if regenerated_options.is_empty() {
                regenerated_options = previous_options;
                option_source = Some("previous_reused".to_string());
            }

            // 通过时间推进对兜底选项做轻量轮转，确保连续交互时选项呈现有变化。
            if !regenerated_options.is_empty() {
                let rotation =
                    (game_state.game_time.total_days as usize) % regenerated_options.len();
                regenerated_options.rotate_left(rotation);
                for (idx, option) in regenerated_options.iter_mut().enumerate() {
                    option.id = idx;
                }
            }
            plot_state.current_scene.available_options = regenerated_options;
        }
    } else {
        plot_state.current_scene.available_options.clear();
        option_source = Some("not_waiting_for_input".to_string());
    }

    plot_state.last_option_generation_source = option_source.clone();
    if let Some(source) = option_source {
        match &mut plot_state.last_generation_diagnostics {
            Some(diag) => {
                diag.push_str(&format!("；选项来源：{}", source));
            }
            None => {
                plot_state.last_generation_diagnostics = Some(format!("选项来源：{}", source));
            }
        }
    }

    let mut engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    if let Some((event_type, message, importance)) = log_entry {
        engine.log_event(timestamp, event_type, message, importance);
    }

    let _npc_reactions = engine
        .process_npc_reactions_for_events(&plot_update.triggered_events)
        .map_err(|e| e.to_string())?;

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
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    engine.get_current_state().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_game(slot_id: u32, engine: State<'_, Mutex<GameEngine>>) -> Result<(), String> {
    validate_slot_id(slot_id).map_err(|e| map_error("保存存档失败", e))?;
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    engine.save_game(slot_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_game(
    slot_id: u32,
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<GameState, String> {
    validate_slot_id(slot_id).map_err(|e| map_error("加载存档失败", e))?;
    let mut engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    engine.load_game(slot_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_save_slots(engine: State<'_, Mutex<GameEngine>>) -> Result<Vec<SaveInfo>, String> {
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    engine.list_saves().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_script(
    script_path: String,
    _engine: State<'_, Mutex<GameEngine>>,
) -> Result<Script, String> {
    use crate::script_manager::ScriptManager;

    validate_file_path(&script_path, &["json"]).map_err(|e| map_error("加载剧本失败", e))?;
    let manager = ScriptManager::new();
    manager
        .load_custom_script(&script_path)
        .map_err(|e| map_error("加载剧本失败", e))
}

#[tauri::command]
pub async fn generate_random_script() -> Result<Script, String> {
    use crate::script_manager::ScriptManager;

    let manager = ScriptManager::new();
    manager
        .generate_random_script()
        .await
        .map_err(|e| map_error("随机剧本生成失败", e))
}

#[tauri::command]
pub async fn parse_novel_characters(novel_path: String) -> Result<Vec<String>, String> {
    use crate::script_manager::ScriptManager;

    validate_file_path(&novel_path, &["txt", "md"]).map_err(|e| map_error("解析小说角色失败", e))?;
    let manager = ScriptManager::new();
    manager
        .extract_novel_characters(&novel_path)
        .map_err(|e| map_error("解析小说角色失败", e))
}

#[tauri::command]
pub async fn load_existing_novel(
    novel_path: String,
    selected_character: String,
) -> Result<Script, String> {
    use crate::script_manager::ScriptManager;

    validate_file_path(&novel_path, &["txt", "md"]).map_err(|e| map_error("导入现有小说失败", e))?;
    if selected_character.trim().is_empty() {
        return Err(map_error(
            "导入现有小说失败",
            AppError::new(crate::app_error::AppErrorKind::InvalidInput, "请选择有效角色"),
        ));
    }
    let manager = ScriptManager::new();
    manager
        .load_existing_novel(&novel_path, &selected_character)
        .map_err(|e| map_error("导入现有小说失败", e))
}

#[tauri::command]
pub async fn get_player_options(
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<Vec<PlayerOption>, String> {
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let plot_state = engine.get_plot_state().map_err(|e| e.to_string())?;
    Ok(plot_state.current_scene.available_options)
}

#[tauri::command]
pub async fn initialize_plot(
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<PlotState, String> {
    let (player_name, realm_name, spiritual_root, location) = {
        let engine = match engine.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let state = engine.get_current_state().map_err(|e| e.to_string())?;
        (
            state.player.name,
            state.player.stats.cultivation_realm.name,
            format!("{:?}", state.player.stats.spiritual_root.element),
            state.player.location,
        )
    };

    let plot_engine = PlotEngine::new();
    let opening = plot_engine
        .generate_opening_plot_async(&player_name, &realm_name, &spiritual_root, &location)
        .await;

    let opening_options = if opening.options.is_empty() {
        None
    } else {
        Some(
            opening
                .options
                .iter()
                .enumerate()
                .map(|(idx, text)| PlayerOption {
                    id: idx,
                    description: text.clone(),
                    requirements: vec![],
                    action: Action::Custom {
                        description: text.clone(),
                    },
                })
                .collect(),
        )
    };

    let mut engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    engine
        .initialize_plot_with_opening(opening.text, opening_options)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_plot_state(
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<PlotState, String> {
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    engine.get_plot_state().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_plot_settings(
    settings: PlotSettings,
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<PlotState, String> {
    if settings.min_interactions_per_chapter == 0
        || settings.max_interactions_per_chapter == 0
        || settings.min_interactions_per_chapter > settings.max_interactions_per_chapter
    {
        return Err(map_error(
            "更新剧情设置失败",
            AppError::new(
                crate::app_error::AppErrorKind::InvalidInput,
                "每章互动次数范围不合法",
            ),
        ));
    }
    if settings.target_chapter_words_min == 0
        || settings.target_chapter_words_max == 0
        || settings.target_chapter_words_min > settings.target_chapter_words_max
    {
        return Err(map_error(
            "更新剧情设置失败",
            AppError::new(
                crate::app_error::AppErrorKind::InvalidInput,
                "章节字数范围不合法",
            ),
        ));
    }
    if settings.novel_style.trim().is_empty() {
        return Err(map_error(
            "更新剧情设置失败",
            AppError::new(crate::app_error::AppErrorKind::InvalidInput, "小说风格不能为空"),
        ));
    }
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    engine
        .update_plot_settings(settings)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_novel(
    title: String,
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<Novel, String> {
    validate_non_empty(&title, "小说标题").map_err(|e| map_error("生成小说失败", e))?;
    let events = {
        let engine = engine.lock().map_err(|e| e.to_string())?;
        let state = engine.get_current_state().map_err(|e| e.to_string())?;
        state.event_history
    };
    generate_novel_from_events(&title, &events).await
}

#[tauri::command]
pub async fn export_novel(novel: Novel, output_path: String) -> Result<(), String> {
    validate_output_path(&output_path, &["txt"]).map_err(|e| map_error("导出小说失败", e))?;
    export_novel_to_path(&novel, &output_path)
}

async fn generate_novel_from_events(title: &str, events: &[crate::event_log::GameEvent]) -> Result<Novel, String> {
    let generator = NovelGenerator::new();
    generator.generate_novel(title.to_string(), events).await
}

fn export_novel_to_path(novel: &Novel, output_path: &str) -> Result<(), String> {
    let generator = NovelGenerator::new();
    generator.export_to_file(novel, output_path)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_log::{EventImportance, GameEvent};
    use crate::models::{CultivationRealm, Element, Grade, SpiritualRoot};
    use crate::script::{InitialState, Location, ScriptType, WorldSetting};
    use tempfile::tempdir;

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

    #[test]
    fn test_validate_slot_id_bounds() {
        assert!(validate_slot_id(1).is_ok());
        assert!(validate_slot_id(99).is_ok());
        assert!(validate_slot_id(0).is_err());
        assert!(validate_slot_id(120).is_err());
    }

    #[test]
    fn test_validate_file_path_extension() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("sample.json");
        std::fs::write(&file, "{}").unwrap();
        let ok = validate_file_path(file.to_str().unwrap(), &["json"]);
        assert!(ok.is_ok());
        let bad = validate_file_path(file.to_str().unwrap(), &["txt"]);
        assert!(bad.is_err());
    }

    #[test]
    fn test_validate_endpoint_scheme() {
        assert!(validate_endpoint("https://example.com").is_ok());
        assert!(validate_endpoint("http://localhost").is_ok());
        assert!(validate_endpoint("ftp://example.com").is_err());
        assert!(validate_endpoint("example.com").is_err());
    }

    #[test]
    fn test_validate_llm_config_requires_api_key_for_remote() {
        let input = LLMConfigInput {
            endpoint: "https://api.example.com/v1".to_string(),
            api_key: "".to_string(),
            model: "test".to_string(),
            max_tokens: 128,
            temperature: 0.7,
        };
        assert!(validate_llm_config_input(&input).is_err());
    }

    #[test]
    fn test_validate_llm_config_allows_local_without_key() {
        let input = LLMConfigInput {
            endpoint: "http://localhost:8000/v1".to_string(),
            api_key: "".to_string(),
            model: "test".to_string(),
            max_tokens: 128,
            temperature: 0.7,
        };
        assert!(validate_llm_config_input(&input).is_ok());
    }

    #[test]
    fn test_validate_output_path_extension() {
        let dir = tempdir().unwrap();
        let out = dir.path().join("novel.txt");
        let ok = validate_output_path(out.to_str().unwrap(), &["txt"]);
        assert!(ok.is_ok());
        let bad = dir.path().join("novel.md");
        let err = validate_output_path(bad.to_str().unwrap(), &["txt"]);
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn test_generate_novel_command_logic() {
        let events = vec![
            GameEvent {
                id: 1,
                timestamp: 1,
                event_type: std::sync::Arc::from("cultivation"),
                description: std::sync::Arc::from("Player cultivated"),
                importance: EventImportance::Normal,
            },
            GameEvent {
                id: 2,
                timestamp: 2,
                event_type: std::sync::Arc::from("combat"),
                description: std::sync::Arc::from("Player won duel"),
                importance: EventImportance::Important,
            },
        ];

        let novel = generate_novel_from_events("Test Novel", &events).await.unwrap();
        assert_eq!(novel.title, "Test Novel");
        assert_eq!(novel.total_events, 2);
        assert!(!novel.chapters.is_empty());
    }

    #[test]
    fn test_export_novel_command_logic() {
        let novel = Novel {
            title: "Exported Novel".to_string(),
            chapters: vec![crate::novel_generator::Chapter {
                index: 1,
                title: "Start".to_string(),
                content: "A new journey starts.".to_string(),
                source_event_ids: vec![1],
            }],
            total_events: 1,
        };

        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("novel_out.txt");
        let result = export_novel_to_path(&novel, output.to_str().unwrap());
        assert!(result.is_ok());
        assert!(output.exists());
    }
}


