use crate::game_state::{Character, GameState, GameTime, WorldState};
use crate::models::{CharacterStats, Lifespan};
use crate::numerical_system::NumericalSystem;
use crate::script::Script;
use crate::script_manager::ScriptManager;
use anyhow::{anyhow, Result};
use std::sync::{Arc, Mutex};

/// Main game engine managing game state and logic
pub struct GameEngine {
    state: Arc<Mutex<Option<GameState>>>,
    script_manager: ScriptManager,
    numerical_system: NumericalSystem,
}

impl GameEngine {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(None)),
            script_manager: ScriptManager::new(),
            numerical_system: NumericalSystem::new(),
        }
    }

    /// Initialize game from a script
    pub fn initialize_game(&mut self, script: Script) -> Result<GameState> {
        // Validate script
        self.script_manager.validate_script(&script)?;

        // Create player character from initial state
        let player_stats = CharacterStats {
            spiritual_root: script.initial_state.player_spiritual_root.clone(),
            cultivation_realm: script
                .world_setting
                .cultivation_realms
                .first()
                .ok_or_else(|| anyhow!("No cultivation realms defined in script"))?
                .clone(),
            techniques: Vec::new(),
            lifespan: Lifespan {
                current_age: script.initial_state.starting_age,
                max_age: 100,
                realm_bonus: 0,
            },
            combat_power: self.numerical_system.calculate_initial_combat_power(
                &script.initial_state.player_spiritual_root,
                script
                    .world_setting
                    .cultivation_realms
                    .first()
                    .ok_or_else(|| anyhow!("No cultivation realms defined"))?,
            ),
        };

        let player = Character::new(
            "player".to_string(),
            script.initial_state.player_name.clone(),
            player_stats,
            script.initial_state.starting_location.clone(),
        );

        // Create world state from script
        let world_state = WorldState::from_script(&script);

        // Initialize game time
        let game_time = GameTime::new(1, 1, 1);

        // Create game state
        let game_state = GameState {
            script,
            player,
            world_state,
            game_time,
        };

        // Store state
        let mut state_lock = self.state.lock().unwrap();
        *state_lock = Some(game_state.clone());

        Ok(game_state)
    }

    /// Get current game state
    pub fn get_current_state(&self) -> Result<GameState> {
        let state_lock = self.state.lock().unwrap();
        state_lock
            .clone()
            .ok_or_else(|| anyhow!("Game not initialized"))
    }

    /// Check if game is initialized
    pub fn is_initialized(&self) -> bool {
        let state_lock = self.state.lock().unwrap();
        state_lock.is_some()
    }
}

impl Default for GameEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Element, Grade, SpiritualRoot};
    use crate::script::{InitialState, Location, ScriptType, WorldSetting};

    fn create_test_script() -> Script {
        let mut world_setting = WorldSetting::new();
        world_setting.cultivation_realms = vec![
            CultivationRealm::new("Qi Condensation".to_string(), 1, 0, 1.0),
            CultivationRealm::new("Foundation Establishment".to_string(), 2, 0, 2.0),
        ];
        world_setting.locations = vec![
            Location {
                id: "sect".to_string(),
                name: "Azure Cloud Sect".to_string(),
                description: "A peaceful cultivation sect".to_string(),
                spiritual_energy: 1.0,
            },
            Location {
                id: "city".to_string(),
                name: "Mortal City".to_string(),
                description: "A bustling mortal city".to_string(),
                spiritual_energy: 0.1,
            },
        ];

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
    fn test_game_engine_creation() {
        let engine = GameEngine::new();
        assert!(!engine.is_initialized());
    }

    #[test]
    fn test_initialize_game() {
        let mut engine = GameEngine::new();
        let script = create_test_script();

        let result = engine.initialize_game(script.clone());
        assert!(result.is_ok());

        let game_state = result.unwrap();
        assert_eq!(game_state.player.name, "Test Player");
        assert_eq!(game_state.player.location, "sect");
        assert_eq!(game_state.player.stats.lifespan.current_age, 16);
        assert_eq!(game_state.game_time.year, 1);
        assert_eq!(game_state.world_state.locations.len(), 2);
    }

    #[test]
    fn test_get_current_state() {
        let mut engine = GameEngine::new();
        let script = create_test_script();

        // Before initialization
        assert!(engine.get_current_state().is_err());

        // After initialization
        engine.initialize_game(script).unwrap();
        assert!(engine.get_current_state().is_ok());
        assert!(engine.is_initialized());
    }

    #[test]
    fn test_initialize_game_with_invalid_script() {
        let mut engine = GameEngine::new();
        let mut script = create_test_script();

        // Remove cultivation realms to make script invalid
        script.world_setting.cultivation_realms.clear();

        let result = engine.initialize_game(script);
        assert!(result.is_err());
    }

    #[test]
    fn test_player_initial_stats() {
        let mut engine = GameEngine::new();
        let script = create_test_script();

        let game_state = engine.initialize_game(script).unwrap();

        // Verify player has correct initial stats
        assert_eq!(
            game_state.player.stats.spiritual_root.element,
            Element::Fire
        );
        assert_eq!(game_state.player.stats.spiritual_root.grade, Grade::Heavenly);
        assert_eq!(game_state.player.stats.cultivation_realm.name, "Qi Condensation");
        assert!(game_state.player.stats.combat_power > 0);
        assert!(game_state.player.inventory.is_empty());
    }

    #[test]
    fn test_world_state_initialization() {
        let mut engine = GameEngine::new();
        let script = create_test_script();

        let game_state = engine.initialize_game(script).unwrap();

        // Verify world state is correctly initialized
        assert!(game_state.world_state.locations.contains_key("sect"));
        assert!(game_state.world_state.locations.contains_key("city"));
        assert!(game_state.world_state.global_events.is_empty());
    }
}
