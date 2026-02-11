use crate::models::CharacterStats;
use crate::llm_service::{LLMConfig, LLMRequest, LLMService};
use crate::numerical_system::{Action, ActionResult, Context, NumericalSystem};
use crate::prompt_builder::{PromptBuilder, PromptConstraints, PromptContext, PromptTemplate};
use crate::response_validator::{ResponseValidator, ValidationConstraints};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionType {
    FreeText,
    SelectedOption,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerAction {
    pub action_type: ActionType,
    pub content: String,
    pub selected_option_id: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerOption {
    pub id: usize,
    pub description: String,
    pub requirements: Vec<String>,
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scene {
    pub id: String,
    pub name: String,
    pub description: String,
    pub location: String,
    pub available_options: Vec<PlayerOption>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotState {
    pub current_scene: Scene,
    pub plot_history: Vec<String>,
    pub is_waiting_for_input: bool,
    pub last_action_result: Option<ActionResult>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotUpdate {
    pub new_scene: Option<Scene>,
    pub plot_text: String,
    pub triggered_events: Vec<String>,
    pub state_changes: Vec<String>,
}

pub struct PlotEngine {
    numerical_system: NumericalSystem,
    llm_service: Option<LLMService>,
    prompt_builder: PromptBuilder,
    response_validator: ResponseValidator,
}

impl PlotEngine {
    pub fn new() -> Self {
        Self {
            numerical_system: NumericalSystem::new(),
            llm_service: Self::initialize_llm_service_from_env(),
            prompt_builder: PromptBuilder::default(),
            response_validator: ResponseValidator::default(),
        }
    }

    fn initialize_llm_service_from_env() -> Option<LLMService> {
        let endpoint = std::env::var("NOBODY_LLM_ENDPOINT").ok()?;
        let api_key = std::env::var("NOBODY_LLM_API_KEY").ok()?;
        let model = std::env::var("NOBODY_LLM_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
        let max_tokens = std::env::var("NOBODY_LLM_MAX_TOKENS")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(512);
        let temperature = std::env::var("NOBODY_LLM_TEMPERATURE")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(0.2);

        let cfg = LLMConfig {
            endpoint,
            api_key,
            model,
            max_tokens,
            temperature,
        };

        LLMService::new(cfg).ok()
    }

    pub fn advance_plot(
        &self,
        current_state: &PlotState,
        action_result: &ActionResult,
    ) -> PlotUpdate {
        let plot_text = self.generate_plot_text(current_state, action_result);
        let triggered_events = action_result.events.clone();

        let state_changes: Vec<String> = action_result
            .stat_changes
            .iter()
            .map(|change| {
                format!(
                    "{}: {} -> {}",
                    change.stat_name, change.old_value, change.new_value
                )
            })
            .collect();

        PlotUpdate {
            new_scene: None,
            plot_text,
            triggered_events,
            state_changes,
        }
    }

    pub fn generate_plot_text(&self, current_state: &PlotState, action_result: &ActionResult) -> String {
        if let Some(text) = self.generate_plot_text_with_llm(current_state, action_result) {
            return text;
        }
        self.generate_plot_text_fallback(current_state, action_result)
    }

    fn generate_plot_text_with_llm(
        &self,
        current_state: &PlotState,
        action_result: &ActionResult,
    ) -> Option<String> {
        if tokio::runtime::Handle::try_current().is_ok() {
            return None;
        }

        let llm_service = self.llm_service.as_ref()?;
        let prompt = self.prompt_builder.build_prompt_with_token_limit(
            PromptTemplate::PlotGeneration,
            &PromptContext {
                scene: Some(current_state.current_scene.description.clone()),
                location: Some(current_state.current_scene.location.clone()),
                actor_name: Some("player".to_string()),
                actor_realm: None,
                actor_combat_power: None,
                history_events: action_result.events.clone(),
                world_setting_summary: Some("Cultivation novel style with scene, events, and NPC reactions".to_string()),
            },
            &PromptConstraints {
                numerical_rules: vec!["must remain consistent with action result".to_string()],
                world_rules: vec![
                    "return plain text only".to_string(),
                    "write concise novel-style narration".to_string(),
                ],
                output_schema_hint: None,
            },
            360,
        );

        let runtime = tokio::runtime::Runtime::new().ok()?;
        let response = runtime
            .block_on(llm_service.generate(LLMRequest {
                prompt,
                max_tokens: Some(220),
                temperature: Some(0.7),
            }))
            .ok()?;

        self.response_validator
            .validate_response(
                &response,
                &ValidationConstraints {
                    require_json: false,
                    max_realm_level: None,
                    min_combat_power: None,
                    max_combat_power: None,
                    max_current_age: None,
                },
            )
            .ok()?;

        let text = response.text.trim();
        if text.is_empty() {
            None
        } else {
            Some(text.to_string())
        }
    }

    fn generate_plot_text_fallback(&self, current_state: &PlotState, action_result: &ActionResult) -> String {
        let status = if action_result.success { "行动成功" } else { "行动受挫" };
        let event_line = if action_result.events.is_empty() {
            "暂无额外事件。".to_string()
        } else {
            format!("事件：{}。", action_result.events.join("；"))
        };

        format!(
            "【{}】在{}，你{}。{} {}",
            current_state.current_scene.name,
            current_state.current_scene.location,
            action_result.description,
            event_line,
            status
        )
    }
    pub fn generate_player_options(
        &self,
        scene: &Scene,
        character: &CharacterStats,
    ) -> Vec<PlayerOption> {
        if !scene.available_options.is_empty() {
            return scene.available_options.clone();
        }

        let mut options = Vec::new();
        let mut option_id = 0;

        // Cultivate option
        options.push(PlayerOption {
            id: option_id,
            description: "Cultivate and improve realm".to_string(),
            requirements: vec![],
            action: Action::Cultivate,
        });
        option_id += 1;

        // Breakthrough option if sub-level is less than 3
        if character.cultivation_realm.sub_level < 3 {
            options.push(PlayerOption {
                id: option_id,
                description: format!(
                    "Attempt breakthrough in {}",
                    character.cultivation_realm.name
                ),
                requirements: vec![format!(
                    "Current realm: {} (sub-level {})",
                    character.cultivation_realm.name, character.cultivation_realm.sub_level
                )],
                action: Action::Breakthrough,
            });
            option_id += 1;
        }

        // Rest option
        options.push(PlayerOption {
            id: option_id,
            description: "Rest and recover".to_string(),
            requirements: vec![],
            action: Action::Rest,
        });
        option_id += 1;

        // Location-specific options
        if scene.location == "azure_cloud_sect" || scene.location == "sect" {
            options.push(PlayerOption {
                id: option_id,
                description: "Visit sect library".to_string(),
                requirements: vec![],
                action: Action::Custom {
                    description: "You study cultivation techniques at the sect library".to_string(),
                },
            });
            option_id += 1;
        } else if scene.location == "city" {
            options.push(PlayerOption {
                id: option_id,
                description: "Explore the marketplace".to_string(),
                requirements: vec![],
                action: Action::Custom {
                    description: "You explore the busy city market".to_string(),
                },
            });
            option_id += 1;
        }

        // Ensure minimum 2 options and maximum 5 options
        if options.len() < 2 {
            options.push(PlayerOption {
                id: option_id,
                description: "Meditate quietly".to_string(),
                requirements: vec![],
                action: Action::Custom {
                    description: "You meditate and reflect on your cultivation path".to_string(),
                },
            });
        } else if options.len() > 5 {
            options.truncate(5);
        }

        options
    }

    pub fn validate_player_action(
        &self,
        action: &PlayerAction,
        available_options: &[PlayerOption],
    ) -> Result<(), String> {
        match action.action_type {
            ActionType::SelectedOption => {
                if let Some(option_id) = action.selected_option_id {
                    if option_id >= available_options.len() {
                        return Err(format!("Invalid option ID: {}", option_id));
                    }
                    Ok(())
                } else {
                    Err("Option ID must be provided when selecting an option".to_string())
                }
            }
            ActionType::FreeText => {
                if action.content.trim().is_empty() {
                    Err("Free text cannot be empty".to_string())
                } else {
                    self.validate_free_text_reasonableness(&action.content, available_options)
                }
            }
        }
    }

    pub fn process_player_action(
        &self,
        action: &PlayerAction,
        character: &CharacterStats,
        available_options: &[PlayerOption],
        context: &Context,
    ) -> Result<ActionResult, String> {
        self.validate_player_action(action, available_options)?;

        match action.action_type {
            ActionType::SelectedOption => {
                let option_id = action.selected_option_id.unwrap();
                let selected_option = &available_options[option_id];

                let result = self.numerical_system.calculate_action_result(
                    character,
                    &selected_option.action,
                    context,
                );

                Ok(result)
            }
            ActionType::FreeText => {
                let interpreted_action = self.interpret_free_text_action(&action.content, character, context);
                Ok(self.numerical_system.calculate_action_result(
                    character,
                    &interpreted_action,
                    context,
                ))
            }
        }
    }

    fn interpret_free_text_action(
        &self,
        free_text: &str,
        character: &CharacterStats,
        context: &Context,
    ) -> Action {
        self.parse_action_with_llm(free_text, character, context)
            .unwrap_or_else(|| self.parse_action_with_rules(free_text))
    }

    fn parse_action_with_llm(
        &self,
        free_text: &str,
        character: &CharacterStats,
        context: &Context,
    ) -> Option<Action> {
        if tokio::runtime::Handle::try_current().is_ok() {
            return None;
        }

        let llm_service = self.llm_service.as_ref()?;

        let prompt = self.prompt_builder.build_prompt_with_token_limit(
            PromptTemplate::OptionGeneration,
            &PromptContext {
                scene: Some(free_text.to_string()),
                location: Some(context.location.clone()),
                actor_name: Some("player".to_string()),
                actor_realm: Some(character.cultivation_realm.name.clone()),
                actor_combat_power: Some(character.combat_power),
                history_events: Vec::new(),
                world_setting_summary: Some(
                    "Interpret user intent into one in-game action".to_string(),
                ),
            },
            &PromptConstraints {
                numerical_rules: vec![
                    "respect current realm and combat power".to_string(),
                ],
                world_rules: vec![
                    "output strict JSON only".to_string(),
                    "json keys: action,target,description".to_string(),
                    "action must be cultivate|rest|breakthrough|combat|custom".to_string(),
                ],
                output_schema_hint: Some(
                    "{\"action\":\"cultivate|rest|breakthrough|combat|custom\",\"target\":\"optional string\",\"description\":\"optional string\"}".to_string(),
                ),
            },
            300,
        );

        let runtime = tokio::runtime::Runtime::new().ok()?;
        let response = runtime
            .block_on(llm_service.generate(LLMRequest {
                prompt,
                max_tokens: Some(128),
                temperature: Some(0.1),
            }))
            .ok()?;

        self.response_validator
            .validate_response(
                &response,
                &ValidationConstraints {
                    require_json: true,
                    max_realm_level: None,
                    min_combat_power: None,
                    max_combat_power: None,
                    max_current_age: None,
                },
            )
            .ok()?;

        let value: Value = serde_json::from_str(&response.text).ok()?;
        let action_name = value.get("action").and_then(Value::as_str)?;
        let description = value
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or(free_text)
            .to_string();
        let target = value
            .get("target")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();

        match action_name.to_ascii_lowercase().as_str() {
            "cultivate" => Some(Action::Cultivate),
            "rest" => Some(Action::Rest),
            "breakthrough" => Some(Action::Breakthrough),
            "combat" => Some(Action::Combat { target_id: target }),
            "custom" => Some(Action::Custom { description }),
            _ => None,
        }
    }

    fn parse_action_with_rules(&self, free_text: &str) -> Action {
        let lower = free_text.to_ascii_lowercase();
        if contains_any(&lower, &["淇偧", "鎵撳潗", "cultivate", "meditate", "training"]) {
            return Action::Cultivate;
        }
        if contains_any(&lower, &["绐佺牬", "breakthrough", "advance realm"]) {
            return Action::Breakthrough;
        }
        if contains_any(&lower, &["浼戞伅", "rest", "sleep", "recover"]) {
            return Action::Rest;
        }
        if contains_any(&lower, &["鎴樻枟", "鏀诲嚮", "fight", "combat", "duel"]) {
            return Action::Combat {
                target_id: "unknown".to_string(),
            };
        }

        Action::Custom {
            description: format!("Player free text action: {}", free_text.trim()),
        }
    }

    fn validate_free_text_reasonableness(
        &self,
        free_text: &str,
        available_options: &[PlayerOption],
    ) -> Result<(), String> {
        if let Some((reasonable, reason)) =
            self.validate_behavior_with_llm(free_text, available_options)
        {
            if !reasonable {
                return Err(format!("Unreasonable action rejected: {}", reason));
            }
        }

        let lower = free_text.to_ascii_lowercase();
        if contains_any(
            &lower,
            &[
                "instant immortal",
                "instantly become immortal",
                "destroy the world",
                "god mode",
                "one punch kill everyone",
                "涓€鎷崇鏉€鎵€鏈変汉",
                "鐬棿椋炲崌",
                "姣佺伃涓栫晫",
                "鏃犳晫妯″紡",
            ],
        ) {
            return Err("Action exceeds current world and character constraints".to_string());
        }

        let can_breakthrough = available_options
            .iter()
            .any(|o| matches!(o.action, Action::Breakthrough));
        if !can_breakthrough
            && contains_any(&lower, &["breakthrough", "绐佺牬", "advance realm", "娓″姭"])
        {
            return Err("Current scene/realm does not allow breakthrough action".to_string());
        }

        Ok(())
    }

    fn validate_behavior_with_llm(
        &self,
        free_text: &str,
        available_options: &[PlayerOption],
    ) -> Option<(bool, String)> {
        if tokio::runtime::Handle::try_current().is_ok() {
            return None;
        }

        let llm_service = self.llm_service.as_ref()?;
        let allowed_actions = available_options
            .iter()
            .map(|o| action_label(&o.action))
            .collect::<Vec<&'static str>>()
            .join(",");

        let prompt = self.prompt_builder.build_prompt_with_token_limit(
            PromptTemplate::OptionGeneration,
            &PromptContext {
                scene: Some(format!(
                    "Player input: {} | allowed actions: {}",
                    free_text, allowed_actions
                )),
                location: None,
                actor_name: Some("player".to_string()),
                actor_realm: None,
                actor_combat_power: None,
                history_events: Vec::new(),
                world_setting_summary: Some(
                    "Judge if player action is reasonable in current cultivation scene".to_string(),
                ),
            },
            &PromptConstraints {
                numerical_rules: vec!["reject actions violating realm/capability".to_string()],
                world_rules: vec![
                    "output strict JSON only".to_string(),
                    "json keys: reasonable,reason".to_string(),
                ],
                output_schema_hint: Some(
                    "{\"reasonable\":true|false,\"reason\":\"string\"}".to_string(),
                ),
            },
            220,
        );

        let runtime = tokio::runtime::Runtime::new().ok()?;
        let response = runtime
            .block_on(llm_service.generate(LLMRequest {
                prompt,
                max_tokens: Some(96),
                temperature: Some(0.1),
            }))
            .ok()?;

        self.response_validator
            .validate_response(
                &response,
                &ValidationConstraints {
                    require_json: true,
                    max_realm_level: None,
                    min_combat_power: None,
                    max_combat_power: None,
                    max_current_age: None,
                },
            )
            .ok()?;

        let value: Value = serde_json::from_str(&response.text).ok()?;
        let reasonable = value.get("reasonable").and_then(Value::as_bool)?;
        let reason = value
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or("no reason")
            .to_string();
        Some((reasonable, reason))
    }
}

impl Default for PlotEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn contains_any(text: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|k| text.contains(k))
}

fn action_label(action: &Action) -> &'static str {
    match action {
        Action::Cultivate => "cultivate",
        Action::Combat { .. } => "combat",
        Action::Breakthrough => "breakthrough",
        Action::Rest => "rest",
        Action::Custom { .. } => "custom",
    }
}

impl Scene {
    pub fn new(id: String, name: String, description: String, location: String) -> Self {
        Self {
            id,
            name,
            description,
            location,
            available_options: Vec::new(),
        }
    }

    pub fn add_option(&mut self, option: PlayerOption) {
        self.available_options.push(option);
    }
}

impl PlotState {
    pub fn new(initial_scene: Scene) -> Self {
        Self {
            current_scene: initial_scene,
            plot_history: Vec::new(),
            is_waiting_for_input: true,
            last_action_result: None,
        }
    }

    pub fn add_to_history(&mut self, text: String) {
        self.plot_history.push(text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CultivationRealm, Element, Grade, Lifespan, SpiritualRoot};

    fn create_test_character() -> CharacterStats {
        CharacterStats {
            spiritual_root: SpiritualRoot {
                element: Element::Fire,
                grade: Grade::Heavenly,
                affinity: 0.8,
            },
            cultivation_realm: CultivationRealm::new("Qi Condensation".to_string(), 1, 0, 1.0),
            techniques: Vec::new(),
            lifespan: Lifespan {
                current_age: 16,
                max_age: 100,
                realm_bonus: 0,
            },
            combat_power: 100,
        }
    }

    fn create_test_scene() -> Scene {
        let mut scene = Scene::new(
            "test_scene".to_string(),
            "Test Scene".to_string(),
            "This is a test scene".to_string(),
            "sect".to_string(),
        );

        scene.add_option(PlayerOption {
            id: 0,
            description: "Cultivate".to_string(),
            requirements: vec![],
            action: Action::Cultivate,
        });

        scene.add_option(PlayerOption {
            id: 1,
            description: "Rest".to_string(),
            requirements: vec![],
            action: Action::Rest,
        });

        scene
    }

    #[test]
    fn test_plot_engine_creation() {
        let _engine = PlotEngine::new();
    }

    #[test]
    fn test_scene_creation() {
        let scene = create_test_scene();
        assert_eq!(scene.id, "test_scene");
        assert_eq!(scene.available_options.len(), 2);
    }

    #[test]
    fn test_plot_state_creation() {
        let scene = create_test_scene();
        let state = PlotState::new(scene);
        assert!(state.is_waiting_for_input);
        assert!(state.plot_history.is_empty());
    }

    #[test]
    fn test_validate_selected_option_valid() {
        let engine = PlotEngine::new();
        let scene = create_test_scene();
        let action = PlayerAction {
            action_type: ActionType::SelectedOption,
            content: "0".to_string(),
            selected_option_id: Some(0),
        };

        let result = engine.validate_player_action(&action, &scene.available_options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_selected_option_invalid_id() {
        let engine = PlotEngine::new();
        let scene = create_test_scene();
        let action = PlayerAction {
            action_type: ActionType::SelectedOption,
            content: "999".to_string(),
            selected_option_id: Some(999),
        };

        let result = engine.validate_player_action(&action, &scene.available_options);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_free_text_empty() {
        let engine = PlotEngine::new();
        let scene = create_test_scene();
        let action = PlayerAction {
            action_type: ActionType::FreeText,
            content: "   ".to_string(),
            selected_option_id: None,
        };

        let result = engine.validate_player_action(&action, &scene.available_options);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_player_action_selected_option() {
        let engine = PlotEngine::new();
        let character = create_test_character();
        let scene = create_test_scene();
        let context = Context {
            location: "sect".to_string(),
            time_of_day: "morning".to_string(),
            weather: None,
        };

        let action = PlayerAction {
            action_type: ActionType::SelectedOption,
            content: "0".to_string(),
            selected_option_id: Some(0),
        };

        let result = engine.process_player_action(
            &action,
            &character,
            &scene.available_options,
            &context,
        );

        assert!(result.is_ok());
        let action_result = result.unwrap();
        assert!(action_result.success);
    }

    #[test]
    fn test_generate_player_options() {
        let engine = PlotEngine::new();
        let character = create_test_character();
        let scene = create_test_scene();

        let options = engine.generate_player_options(&scene, &character);
        assert_eq!(options.len(), 2);
        assert_eq!(options[0].description, "Cultivate");
        assert_eq!(options[1].description, "Rest");
    }

    #[test]
    fn test_generate_player_options_empty_scene() {
        let engine = PlotEngine::new();
        let character = create_test_character();
        let scene = Scene::new(
            "empty".to_string(),
            "Empty Scene".to_string(),
            "Scene with no predefined options".to_string(),
            "sect".to_string(),
        );

        let options = engine.generate_player_options(&scene, &character);
        
        assert!(options.len() >= 2 && options.len() <= 5);
        assert!(options.iter().any(|o| matches!(o.action, Action::Cultivate)));
        assert!(options.iter().any(|o| matches!(o.action, Action::Rest)));
    }

    #[test]
    fn test_generate_options_with_breakthrough() {
        let engine = PlotEngine::new();
        let mut character = create_test_character();
        character.cultivation_realm.sub_level = 1;
        
        let scene = Scene::new(
            "test".to_string(),
            "Test".to_string(),
            "Test scene".to_string(),
            "sect".to_string(),
        );

        let options = engine.generate_player_options(&scene, &character);
        
        assert!(options.iter().any(|o| matches!(o.action, Action::Breakthrough)));
    }

    #[test]
    fn test_generate_options_location_specific() {
        let engine = PlotEngine::new();
        let character = create_test_character();
        
        let sect_scene = Scene::new(
            "sect_scene".to_string(),
            "Sect".to_string(),
            "At the sect".to_string(),
            "sect".to_string(),
        );

        let sect_options = engine.generate_player_options(&sect_scene, &character);
        assert!(sect_options
            .iter()
            .any(|o| matches!(o.action, Action::Custom { .. })));

        let city_scene = Scene::new(
            "city_scene".to_string(),
            "City".to_string(),
            "In the city".to_string(),
            "city".to_string(),
        );

        let city_options = engine.generate_player_options(&city_scene, &character);
        assert!(city_options
            .iter()
            .any(|o| matches!(o.action, Action::Custom { .. })));
    }

    #[test]
    fn test_advance_plot() {
        let engine = PlotEngine::new();
        let scene = create_test_scene();
        let state = PlotState::new(scene);

        let action_result = ActionResult {
            success: true,
            description: "淇偧鎴愬姛".to_string(),
            stat_changes: vec![],
            events: vec!["Cultivation completed".to_string()],
        };

        let update = engine.advance_plot(&state, &action_result);
        assert!(update.plot_text.contains("淇偧鎴愬姛"));
        assert_eq!(update.triggered_events.len(), 1);
    }

    #[test]
    fn test_generate_plot_text_contains_required_information() {
        let engine = PlotEngine::new();
        let scene = create_test_scene();
        let state = PlotState::new(scene);

        let action_result = ActionResult {
            success: true,
            description: "你谨慎地运转功法".to_string(),
            stat_changes: vec![],
            events: vec!["NPC reactions: npc_elder_1 -> observe".to_string()],
        };

        let text = engine.generate_plot_text(&state, &action_result);
        assert!(text.contains("sect") || text.contains("Test Scene"));
        assert!(text.contains("NPC reactions") || text.contains("事件"));
    }

    #[test]
    fn test_generate_plot_text_has_novel_style_fallback() {
        let engine = PlotEngine::new();
        let scene = create_test_scene();
        let state = PlotState::new(scene);

        let action_result = ActionResult {
            success: true,
            description: "在晨光中吐纳灵气".to_string(),
            stat_changes: vec![],
            events: vec![],
        };

        let text = engine.generate_plot_text(&state, &action_result);
        assert!(text.contains("你"));
        assert!(text.contains("【") || text.contains("。"));
    }

    #[test]
    fn test_validate_action_with_no_option_id() {
        let engine = PlotEngine::new();
        let scene = create_test_scene();
        
        let action = PlayerAction {
            action_type: ActionType::SelectedOption,
            content: "test".to_string(),
            selected_option_id: None,
        };

        let result = engine.validate_player_action(&action, &scene.available_options);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Option ID must be provided"));
    }

    #[test]
    fn test_validate_action_with_valid_free_text() {
        let engine = PlotEngine::new();
        let scene = create_test_scene();
        
        let action = PlayerAction {
            action_type: ActionType::FreeText,
            content: "I want to explore the forest".to_string(),
            selected_option_id: None,
        };

        let result = engine.validate_player_action(&action, &scene.available_options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_action_rejects_unreasonable_free_text() {
        let engine = PlotEngine::new();
        let scene = create_test_scene();

        let action = PlayerAction {
            action_type: ActionType::FreeText,
            content: "I will instantly become immortal and destroy the world".to_string(),
            selected_option_id: None,
        };

        let result = engine.validate_player_action(&action, &scene.available_options);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("constraints"));
    }

    #[test]
    fn test_process_action_calculates_result_correctly() {
        let engine = PlotEngine::new();
        let character = create_test_character();
        let scene = create_test_scene();
        let context = Context {
            location: "sect".to_string(),
            time_of_day: "morning".to_string(),
            weather: None,
        };

        let action = PlayerAction {
            action_type: ActionType::SelectedOption,
            content: "0".to_string(),
            selected_option_id: Some(0),
        };

        let result = engine.process_player_action(
            &action,
            &character,
            &scene.available_options,
            &context,
        );

        assert!(result.is_ok());
        let action_result = result.unwrap();
        assert!(action_result.success);
        assert!(!action_result.description.is_empty());
    }

    #[test]
    fn test_process_action_rejects_invalid_option() {
        let engine = PlotEngine::new();
        let character = create_test_character();
        let scene = create_test_scene();
        let context = Context {
            location: "sect".to_string(),
            time_of_day: "morning".to_string(),
            weather: None,
        };

        let action = PlayerAction {
            action_type: ActionType::SelectedOption,
            content: "999".to_string(),
            selected_option_id: Some(999),
        };

        let result = engine.process_player_action(
            &action,
            &character,
            &scene.available_options,
            &context,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid option ID"));
    }

    #[test]
    fn test_process_action_accepts_free_text() {
        let engine = PlotEngine::new();
        let character = create_test_character();
        let scene = create_test_scene();
        let context = Context {
            location: "sect".to_string(),
            time_of_day: "morning".to_string(),
            weather: None,
        };

        let action = PlayerAction {
            action_type: ActionType::FreeText,
            content: "I want to explore".to_string(),
            selected_option_id: None,
        };

        let result = engine.process_player_action(
            &action,
            &character,
            &scene.available_options,
            &context,
        );

        assert!(result.is_ok());
        assert!(!result.unwrap().description.is_empty());
    }

    #[test]
    fn test_process_different_actions() {
        let engine = PlotEngine::new();
        let character = create_test_character();
        let context = Context {
            location: "sect".to_string(),
            time_of_day: "morning".to_string(),
            weather: None,
        };

        let mut scene = Scene::new(
            "test".to_string(),
            "Test".to_string(),
            "Test scene".to_string(),
            "sect".to_string(),
        );

        scene.add_option(PlayerOption {
            id: 0,
            description: "Cultivate".to_string(),
            requirements: vec![],
            action: Action::Cultivate,
        });

        scene.add_option(PlayerOption {
            id: 1,
            description: "Rest".to_string(),
            requirements: vec![],
            action: Action::Rest,
        });

        scene.add_option(PlayerOption {
            id: 2,
            description: "Breakthrough".to_string(),
            requirements: vec![],
            action: Action::Breakthrough,
        });

        let cultivate_action = PlayerAction {
            action_type: ActionType::SelectedOption,
            content: "0".to_string(),
            selected_option_id: Some(0),
        };

        let cultivate_result = engine.process_player_action(
            &cultivate_action,
            &character,
            &scene.available_options,
            &context,
        );
        assert!(cultivate_result.is_ok());
        assert!(!cultivate_result.unwrap().description.is_empty());

        let rest_action = PlayerAction {
            action_type: ActionType::SelectedOption,
            content: "1".to_string(),
            selected_option_id: Some(1),
        };

        let rest_result = engine.process_player_action(
            &rest_action,
            &character,
            &scene.available_options,
            &context,
        );
        assert!(rest_result.is_ok());
        assert!(!rest_result.unwrap().description.is_empty());

        let breakthrough_action = PlayerAction {
            action_type: ActionType::SelectedOption,
            content: "2".to_string(),
            selected_option_id: Some(2),
        };

        let breakthrough_result = engine.process_player_action(
            &breakthrough_action,
            &character,
            &scene.available_options,
            &context,
        );
        assert!(breakthrough_result.is_ok());
    }

    #[test]
    fn test_action_result_includes_events() {
        let engine = PlotEngine::new();
        let character = create_test_character();
        let scene = create_test_scene();
        let context = Context {
            location: "sect".to_string(),
            time_of_day: "morning".to_string(),
            weather: None,
        };

        let action = PlayerAction {
            action_type: ActionType::SelectedOption,
            content: "0".to_string(),
            selected_option_id: Some(0),
        };

        let result = engine.process_player_action(
            &action,
            &character,
            &scene.available_options,
            &context,
        );

        assert!(result.is_ok());
        let action_result = result.unwrap();
        assert!(!action_result.events.is_empty());
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::models::{CultivationRealm, Element, Grade, Lifespan, SpiritualRoot};
    use proptest::prelude::*;

    fn arb_scene() -> impl Strategy<Value = Scene> {
        ("[a-z_]+", "[A-Z][a-z ]+", "[A-Za-z ]+", "[a-z]+").prop_map(
            |(id, name, description, location)| {
                let mut scene = Scene::new(id, name, description, location);
                
                scene.add_option(PlayerOption {
                    id: 0,
                    description: "Cultivate".to_string(),
                    requirements: vec![],
                    action: Action::Cultivate,
                });
                
                scene.add_option(PlayerOption {
                    id: 1,
                    description: "Rest".to_string(),
                    requirements: vec![],
                    action: Action::Rest,
                });
                
                scene
            },
        )
    }

    fn arb_character() -> impl Strategy<Value = CharacterStats> {
        (0u32..=3, 0u32..=3).prop_map(|(level, sub_level)| {
            CharacterStats {
                spiritual_root: SpiritualRoot {
                    element: Element::Fire,
                    grade: Grade::Heavenly,
                    affinity: 0.8,
                },
                cultivation_realm: CultivationRealm::new(
                    "Test Realm".to_string(),
                    level,
                    sub_level,
                    1.0,
                ),
                techniques: Vec::new(),
                lifespan: Lifespan {
                    current_age: 16,
                    max_age: 100,
                    realm_bonus: 0,
                },
                combat_power: 100,
            }
        })
    }

    proptest! {
        #[test]
        fn test_property_18_plot_pauses_at_decision_points(
            scene in arb_scene()
        ) {
            let plot_state = PlotState::new(scene.clone());
            
            prop_assert!(plot_state.is_waiting_for_input, 
                "Plot should pause at decision points waiting for input");
            
            prop_assert!(!plot_state.current_scene.available_options.is_empty(),
                "Decision points should have available options for player to choose");
            
            prop_assert!(plot_state.last_action_result.is_none(),
                "No automatic action should be executed while waiting for player input");
        }
    }

    proptest! {
        #[test]
        fn test_plot_pauses_after_action(
            scene in arb_scene()
        ) {
            let engine = PlotEngine::new();
            let mut plot_state = PlotState::new(scene);
            
            let action_result = ActionResult {
                success: true,
                description: "Action completed".to_string(),
                stat_changes: vec![],
                events: vec![],
            };
            
            let update = engine.advance_plot(&plot_state, &action_result);
            
            plot_state.last_action_result = Some(action_result);
            plot_state.add_to_history(update.plot_text);
            
            prop_assert!(plot_state.is_waiting_for_input,
                "Plot should continue waiting for player input after advancing");
        }
    }

    proptest! {
        #[test]
        fn test_property_19_option_count_constraint(
            character in arb_character()
        ) {
            let engine = PlotEngine::new();
            
            let scene = Scene::new(
                "test".to_string(),
                "Test".to_string(),
                "Test scene".to_string(),
                "sect".to_string(),
            );
            
            let options = engine.generate_player_options(&scene, &character);
            
            prop_assert!(options.len() >= 2 && options.len() <= 5,
                "Generated options count should be between 2 and 5, got {}", options.len());
        }
    }

    proptest! {
        #[test]
        fn test_property_20_free_text_intent_parsing(
            input in "[A-Za-z0-9_ ]{1,120}"
        ) {
            let engine = PlotEngine::new();
            let character = CharacterStats {
                spiritual_root: SpiritualRoot {
                    element: Element::Fire,
                    grade: Grade::Heavenly,
                    affinity: 0.8,
                },
                cultivation_realm: CultivationRealm::new(
                    "Test Realm".to_string(),
                    1,
                    0,
                    1.0,
                ),
                techniques: Vec::new(),
                lifespan: Lifespan {
                    current_age: 16,
                    max_age: 100,
                    realm_bonus: 0,
                },
                combat_power: 100,
            };
            let context = Context {
                location: "sect".to_string(),
                time_of_day: "day".to_string(),
                weather: None,
            };

            let action = PlayerAction {
                action_type: ActionType::FreeText,
                content: if input.trim().is_empty() {
                    "cultivate".to_string()
                } else {
                    input
                },
                selected_option_id: None,
            };

            let result = engine.process_player_action(&action, &character, &[], &context);
            prop_assert!(result.is_ok());
        }
    }

    proptest! {
        #[test]
        fn test_property_21_unreasonable_actions_are_rejected(
            suffix in "[A-Za-z0-9 ]{0,40}"
        ) {
            let engine = PlotEngine::new();
            let mut scene = Scene::new(
                "test".to_string(),
                "Test".to_string(),
                "Test scene".to_string(),
                "sect".to_string(),
            );
            scene.add_option(PlayerOption {
                id: 0,
                description: "Cultivate".to_string(),
                requirements: vec![],
                action: Action::Cultivate,
            });

            let action = PlayerAction {
                action_type: ActionType::FreeText,
                content: format!("instantly become immortal and destroy the world {}", suffix),
                selected_option_id: None,
            };

            let result = engine.validate_player_action(&action, &scene.available_options);
            prop_assert!(result.is_err());
        }
    }

    #[test]
    fn test_plot_only_advances_with_player_action() {
        let mut scene = Scene::new(
            "test".to_string(),
            "Test".to_string(),
            "Test scene".to_string(),
            "location".to_string(),
        );
        
        scene.add_option(PlayerOption {
            id: 0,
            description: "Option 1".to_string(),
            requirements: vec![],
            action: Action::Cultivate,
        });
        
        let plot_state = PlotState::new(scene);
        
        assert!(plot_state.is_waiting_for_input);
        assert!(plot_state.last_action_result.is_none());
        assert!(plot_state.plot_history.is_empty());
    }
}

