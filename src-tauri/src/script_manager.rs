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
    use crate::script::{InitialState, Location, ScriptType, WorldSetting};

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

#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::models::{CultivationRealm, Element, Grade, SpiritualRoot};
    use crate::script::{Faction, InitialState, Location, ScriptType, Technique, WorldSetting};
    use proptest::prelude::*;

    // Arbitrary generators for script components
    fn arb_element() -> impl Strategy<Value = Element> {
        prop_oneof![
            Just(Element::Fire),
            Just(Element::Water),
            Just(Element::Wood),
            Just(Element::Metal),
            Just(Element::Earth),
        ]
    }

    fn arb_grade() -> impl Strategy<Value = Grade> {
        prop_oneof![
            Just(Grade::Heavenly),
            Just(Grade::Earth),
            Just(Grade::Human),
        ]
    }

    fn arb_spiritual_root() -> impl Strategy<Value = SpiritualRoot> {
        (arb_element(), arb_grade(), 0.0f32..=1.0f32).prop_map(|(element, grade, affinity)| {
            SpiritualRoot {
                element,
                grade,
                affinity,
            }
        })
    }

    fn arb_cultivation_realm() -> impl Strategy<Value = CultivationRealm> {
        ("[a-zA-Z ]{3,20}", 1u32..=10, 0u32..=4, 0.1f32..=10.0f32).prop_map(
            |(name, level, sub_level, power_multiplier)| {
                CultivationRealm::new(name, level, sub_level, power_multiplier)
            },
        )
    }

    fn arb_location() -> impl Strategy<Value = Location> {
        (
            "[a-z]{3,10}",
            "[a-zA-Z ]{3,20}",
            "[a-zA-Z ]{5,50}",
            0.0f32..=10.0f32,
        )
            .prop_map(|(id, name, description, spiritual_energy)| Location {
                id,
                name,
                description,
                spiritual_energy,
            })
    }

    fn arb_faction() -> impl Strategy<Value = Faction> {
        (
            "[a-z]{3,10}",
            "[a-zA-Z ]{3,20}",
            "[a-zA-Z ]{5,50}",
            1u32..=1000,
        )
            .prop_map(|(id, name, description, power_level)| Faction {
                id,
                name,
                description,
                power_level,
            })
    }

    fn arb_technique() -> impl Strategy<Value = Technique> {
        (
            "[a-z]{3,10}",
            "[a-zA-Z ]{3,20}",
            "[a-zA-Z ]{5,50}",
            1u32..=10,
            prop::option::of(arb_element()),
        )
            .prop_map(|(id, name, description, required_realm_level, element)| Technique {
                id,
                name,
                description,
                required_realm_level,
                element,
            })
    }

    fn arb_script_type() -> impl Strategy<Value = ScriptType> {
        prop_oneof![
            Just(ScriptType::ExistingNovel),
            Just(ScriptType::RandomGenerated),
            Just(ScriptType::Custom),
        ]
    }

    // Strategy for generating scripts with missing required fields
    fn arb_invalid_script() -> impl Strategy<Value = Script> {
        prop_oneof![
            // Script with no cultivation realms
            arb_valid_script_base().prop_map(|mut script| {
                script.world_setting.cultivation_realms.clear();
                script
            }),
            // Script with no locations
            arb_valid_script_base().prop_map(|mut script| {
                script.world_setting.locations.clear();
                script
            }),
            // Script with invalid starting location
            arb_valid_script_base().prop_map(|mut script| {
                script.initial_state.starting_location = "nonexistent_location".to_string();
                script
            }),
            // Script with invalid age (too young)
            arb_valid_script_base().prop_map(|mut script| {
                script.initial_state.starting_age = 5;
                script
            }),
            // Script with invalid age (too old)
            arb_valid_script_base().prop_map(|mut script| {
                script.initial_state.starting_age = 150;
                script
            }),
        ]
    }

    fn arb_valid_script_base() -> impl Strategy<Value = Script> {
        (
            "[a-z]{3,10}",
            "[a-zA-Z ]{3,20}",
            arb_script_type(),
            prop::collection::vec(arb_cultivation_realm(), 1..5),
            prop::collection::vec(arb_spiritual_root(), 0..5),
            prop::collection::vec(arb_technique(), 0..10),
            prop::collection::vec(arb_location(), 1..5),
            prop::collection::vec(arb_faction(), 0..5),
            "[a-zA-Z ]{3,20}",
            arb_spiritual_root(),
            10u32..=100,
        )
            .prop_map(
                |(
                    id,
                    name,
                    script_type,
                    cultivation_realms,
                    spiritual_roots,
                    techniques,
                    locations,
                    factions,
                    player_name,
                    player_spiritual_root,
                    starting_age,
                )| {
                    let starting_location = locations[0].id.clone();

                    let world_setting = WorldSetting {
                        cultivation_realms,
                        spiritual_roots,
                        techniques,
                        locations,
                        factions,
                    };

                    let initial_state = InitialState {
                        player_name,
                        player_spiritual_root,
                        starting_location,
                        starting_age,
                    };

                    Script::new(id, name, script_type, world_setting, initial_state)
                },
            )
    }

    // Feature: Nobody, Property 3: Script validation consistency
    // For any script, if it lacks necessary world settings or numerical system parameters,
    // the system should reject loading and return a descriptive error message
    // Validates Requirements: 1.6, 1.7
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_property_script_validation_rejects_invalid_scripts(
            invalid_script in arb_invalid_script()
        ) {
            let manager = ScriptManager::new();
            let result = manager.validate_script(&invalid_script);

            // Property: All invalid scripts should be rejected
            prop_assert!(
                result.is_err(),
                "Script with missing required fields should be rejected"
            );

            // Verify error message is descriptive
            let error_msg = result.unwrap_err().to_string();
            prop_assert!(
                !error_msg.is_empty(),
                "Error message should not be empty"
            );
            prop_assert!(
                error_msg.contains("Script validation failed"),
                "Error message should indicate validation failure: {}",
                error_msg
            );
        }
    }
}
