use crate::script::Script;
use anyhow::{anyhow, Result};
use std::path::Path;

/// Script manager for loading and validating scripts
pub struct ScriptManager {}

impl ScriptManager {
    pub fn new() -> Self {
        Self {}
    }

    /// Load custom script from file
    pub fn load_custom_script(&self, file_path: &str) -> Result<Script> {
        let path = Path::new(file_path);
        
        if !path.exists() {
            return Err(anyhow!("Script file not found: {}", file_path));
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read script file: {}", e))?;

        let script: Script = serde_json::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse script JSON: {}", e))?;

        self.validate_script(&script)?;

        Ok(script)
    }

    /// Validate script has all required fields
    pub fn validate_script(&self, script: &Script) -> Result<()> {
        // Check world setting has cultivation realms
        if script.world_setting.cultivation_realms.is_empty() {
            return Err(anyhow!(
                "Script validation failed: No cultivation realms defined"
            ));
        }

        // Check world setting has at least one location
        if script.world_setting.locations.is_empty() {
            return Err(anyhow!(
                "Script validation failed: No locations defined"
            ));
        }

        // Check initial state has valid starting location
        let location_exists = script
            .world_setting
            .locations
            .iter()
            .any(|loc| loc.id == script.initial_state.starting_location);

        if !location_exists {
            return Err(anyhow!(
                "Script validation failed: Starting location '{}' not found in world setting",
                script.initial_state.starting_location
            ));
        }

        // Check player age is reasonable
        if script.initial_state.starting_age < 10 || script.initial_state.starting_age > 100 {
            return Err(anyhow!(
                "Script validation failed: Starting age {} is not reasonable (should be 10-100)",
                script.initial_state.starting_age
            ));
        }

        Ok(())
    }
}

impl Default for ScriptManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CultivationRealm, Element, Grade, SpiritualRoot};
    use crate::script::{InitialState, Location, ScriptType};

    fn create_valid_script() -> Script {
        let mut world_setting = WorldSetting::new();
        world_setting.cultivation_realms = vec![
            CultivationRealm::new("Qi Condensation".to_string(), 1, 0, 1.0),
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
                affinity: 0.8,
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
    fn test_validate_script_success() {
        let manager = ScriptManager::new();
        let script = create_valid_script();

        assert!(manager.validate_script(&script).is_ok());
    }

    #[test]
    fn test_validate_script_no_realms() {
        let manager = ScriptManager::new();
        let mut script = create_valid_script();
        script.world_setting.cultivation_realms.clear();

        let result = manager.validate_script(&script);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No cultivation realms"));
    }

    #[test]
    fn test_validate_script_no_locations() {
        let manager = ScriptManager::new();
        let mut script = create_valid_script();
        script.world_setting.locations.clear();

        let result = manager.validate_script(&script);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No locations"));
    }

    #[test]
    fn test_validate_script_invalid_starting_location() {
        let manager = ScriptManager::new();
        let mut script = create_valid_script();
        script.initial_state.starting_location = "nonexistent".to_string();

        let result = manager.validate_script(&script);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Starting location"));
    }

    #[test]
    fn test_validate_script_invalid_age() {
        let manager = ScriptManager::new();
        let mut script = create_valid_script();
        script.initial_state.starting_age = 5;

        let result = manager.validate_script(&script);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Starting age"));
    }

    #[test]
    fn test_load_custom_script_file_not_found() {
        let manager = ScriptManager::new();
        let result = manager.load_custom_script("nonexistent.json");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
