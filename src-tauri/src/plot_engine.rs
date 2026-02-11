use crate::models::CharacterStats;
use crate::numerical_system::{Action, ActionResult, Context, NumericalSystem};
use serde::{Deserialize, Serialize};

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
}

impl PlotEngine {
    pub fn new() -> Self {
        Self {
            numerical_system: NumericalSystem::new(),
        }
    }

    pub fn advance_plot(
        &self,
        _current_state: &PlotState,
        action_result: &ActionResult,
    ) -> PlotUpdate {
        let plot_text = self.generate_plot_text_from_result(action_result);
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

    fn generate_plot_text_from_result(&self, action_result: &ActionResult) -> String {
        if action_result.success {
            format!("{}.", action_result.description)
        } else {
            format!("Failed: {}.", action_result.description)
        }
    }

    pub fn generate_player_options(
        &self,
        scene: &Scene,
        _character: &CharacterStats,
    ) -> Vec<PlayerOption> {
        scene.available_options.clone()
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
                    Ok(())
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
                Err("Free text input is not supported in current version".to_string())
            }
        }
    }
}

impl Default for PlotEngine {
    fn default() -> Self {
        Self::new()
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
        let engine = PlotEngine::new();
        assert!(true);
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
    fn test_advance_plot() {
        let engine = PlotEngine::new();
        let scene = create_test_scene();
        let state = PlotState::new(scene);

        let action_result = ActionResult {
            success: true,
            description: "Cultivation successful".to_string(),
            stat_changes: vec![],
            events: vec!["Cultivation completed".to_string()],
        };

        let update = engine.advance_plot(&state, &action_result);
        assert!(update.plot_text.contains("Cultivation successful"));
        assert_eq!(update.triggered_events.len(), 1);
    }
}
