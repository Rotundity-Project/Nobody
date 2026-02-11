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
        character: &CharacterStats,
    ) -> Vec<PlayerOption> {
        if !scene.available_options.is_empty() {
            return scene.available_options.clone();
        }

        let mut options = Vec::new();
        let mut option_id = 0;

        options.push(PlayerOption {
            id: option_id,
            description: "Cultivate to improve your realm".to_string(),
            requirements: vec![],
            action: Action::Cultivate,
        });
        option_id += 1;

        if character.cultivation_realm.sub_level < 3 {
            options.push(PlayerOption {
                id: option_id,
                description: format!(
                    "Attempt breakthrough to next sub-level of {}",
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

        options.push(PlayerOption {
            id: option_id,
            description: "Rest and recover energy".to_string(),
            requirements: vec![],
            action: Action::Rest,
        });
        option_id += 1;

        if scene.location == "sect" {
            options.push(PlayerOption {
                id: option_id,
                description: "Visit the sect library".to_string(),
                requirements: vec![],
                action: Action::Custom {
                    description: "You visit the sect library to study cultivation techniques".to_string(),
                },
            });
            option_id += 1;
        } else if scene.location == "city" {
            options.push(PlayerOption {
                id: option_id,
                description: "Explore the marketplace".to_string(),
                requirements: vec![],
                action: Action::Custom {
                    description: "You explore the bustling marketplace".to_string(),
                },
            });
        }

        if options.len() < 2 {
            options.push(PlayerOption {
                id: option_id,
                description: "Meditate and reflect".to_string(),
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
        assert!(options.iter().any(|o| o.description.contains("Cultivate")));
        assert!(options.iter().any(|o| o.description.contains("Rest")));
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
        
        assert!(options.iter().any(|o| o.description.contains("breakthrough")));
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
        assert!(sect_options.iter().any(|o| o.description.contains("library")));

        let city_scene = Scene::new(
            "city_scene".to_string(),
            "City".to_string(),
            "In the city".to_string(),
            "city".to_string(),
        );

        let city_options = engine.generate_player_options(&city_scene, &character);
        assert!(city_options.iter().any(|o| o.description.contains("marketplace")));
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
    fn test_process_action_rejects_free_text() {
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

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not supported"));
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
        assert!(cultivate_result.unwrap().description.contains("Cultivation"));

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
        assert!(rest_result.unwrap().description.contains("rest"));

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
