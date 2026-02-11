use crate::script::Script;
use anyhow::{anyhow, Result};
use std::path::Path;

// Script manager for loading and validating scripts
pub struct ScriptManager {}

impl ScriptManager {
    pub fn new() -> Self {
        Self {}
    }

    // Load custom script from file
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

    // Validate script has all required fields
    pub fn validate_script(&self, script: &Script) -> Result<()> {
        // Check cultivation realms exist
        if script.world_setting.cultivation_realms.is_empty() {
            return Err(anyhow!(
                "Script validation failed: No cultivation realms defined"
            ));
        }

        // Check at least one location exists
        if script.world_setting.locations.is_empty() {
            return Err(anyhow!(
                "Script validation failed: No locations defined"
            ));
        }

        // Check starting location is valid
        let location_exists = script
            .world_setting
            .locations
            .iter()
            .any(|loc| loc.id == script.initial_state.starting_location);

        if !location_exists {
            return Err(anyhow!(
                "Script validation failed: Starting location '{}' not found in world settings",
                script.initial_state.starting_location
            ));
        }

        // Check starting age is reasonable
        if script.initial_state.starting_age < 10 || script.initial_state.starting_age > 100 {
            return Err(anyhow!(
                "Script validation failed: Starting age {} is invalid (should be 10-100)",
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
        world_setting.spiritual_roots = vec![
            SpiritualRoot {
                element: Element::Fire,
                grade: Grade::Heavenly,
                affinity: 0.8,
            },
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
            "test_script".to_string(),
            "Test Script".to_string(),
            ScriptType::Custom,
            world_setting,
            initial_state,
        )
    }

    #[test]
    fn test_validate_valid_script() {
        let manager = ScriptManager::new();
        let script = create_valid_script();
        assert!(manager.validate_script(&script).is_ok());
    }

    #[test]
    fn test_validate_script_missing_realms() {
        let manager = ScriptManager::new();
        let mut script = create_valid_script();
        script.world_setting.cultivation_realms.clear();
        
        let result = manager.validate_script(&script);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cultivation realms"));
    }

    #[test]
    fn test_validate_script_missing_locations() {
        let manager = ScriptManager::new();
        let mut script = create_valid_script();
        script.world_setting.locations.clear();
        
        let result = manager.validate_script(&script);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("locations"));
    }

    #[test]
    fn test_validate_script_invalid_starting_location() {
        let manager = ScriptManager::new();
        let mut script = create_valid_script();
        script.initial_state.starting_location = "nonexistent".to_string();
        
        let result = manager.validate_script(&script);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Starting location"));
    }

    #[test]
    fn test_validate_script_invalid_starting_age() {
        let manager = ScriptManager::new();
        let mut script = create_valid_script();
        script.initial_state.starting_age = 5;
        
        let result = manager.validate_script(&script);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Starting age"));
    }
}

// Property-based tests
#[cfg(test)]
mod proptests {
    use super::*;
    use crate::models::{CultivationRealm, Element, Grade, SpiritualRoot};
    use crate::script::{Faction, InitialState, Location, ScriptType, Technique, WorldSetting};
    use proptest::prelude::*;

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
            Just(Grade::Earthly),
            Just(Grade::Mortal),
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
        ("[a-zA-Z ]{5,20}", 1u32..=10, 0u32..=9, 1.0f32..=10.0f32).prop_map(
            |(name, level, sub_level, power_multiplier)| {
                CultivationRealm::new(name, level, sub_level, power_multiplier)
            },
        )
    }

    fn arb_location() -> impl Strategy<Value = Location> {
        ("[a-z]{3,10}", "[a-zA-Z ]{5,20}", "[a-zA-Z ]{10,50}", 0.0f32..=10.0f32).prop_map(
            |(id, name, description, spiritual_energy)| Location {
                id,
                name,
                description,
                spiritual_energy,
            },
        )
    }

    fn arb_faction() -> impl Strategy<Value = Faction> {
        ("[a-z]{3,10}", "[a-zA-Z ]{5,20}", "[a-zA-Z ]{10,50}", 1u32..=100).prop_map(
            |(id, name, description, power_level)| Faction {
                id,
                name,
                description,
                power_level,
            },
        )
    }

    fn arb_technique() -> impl Strategy<Value = Technique> {
        (
            "[a-z]{3,10}",
            "[a-zA-Z ]{5,20}",
            "[a-zA-Z ]{10,50}",
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

    fn arb_world_setting() -> impl Strategy<Value = WorldSetting> {
        (
            prop::collection::vec(arb_cultivation_realm(), 1..=5),
            prop::collection::vec(arb_spiritual_root(), 1..=5),
            prop::collection::vec(arb_technique(), 0..=5),
            prop::collection::vec(arb_location(), 1..=5),
            prop::collection::vec(arb_faction(), 0..=5),
        )
            .prop_map(
                |(cultivation_realms, spiritual_roots, techniques, locations, factions)| {
                    WorldSetting {
                        cultivation_realms,
                        spiritual_roots,
                        techniques,
                        locations,
                        factions,
                    }
                },
            )
    }

    fn arb_initial_state(world_setting: &WorldSetting) -> impl Strategy<Value = InitialState> {
        let location_ids: Vec<String> = world_setting
            .locations
            .iter()
            .map(|loc| loc.id.clone())
            .collect();

        let starting_location = if !location_ids.is_empty() {
            prop::sample::select(location_ids)
        } else {
            Just("default".to_string()).boxed()
        };

        (
            "[a-zA-Z ]{3,20}",
            arb_spiritual_root(),
            starting_location,
            10u32..=100,
        )
            .prop_map(
                |(player_name, player_spiritual_root, starting_location, starting_age)| {
                    InitialState {
                        player_name,
                        player_spiritual_root,
                        starting_location,
                        starting_age,
                    }
                },
            )
    }

    fn arb_valid_script() -> impl Strategy<Value = Script> {
        arb_world_setting().prop_flat_map(|world_setting| {
            let initial_state = arb_initial_state(&world_setting);
            (
                "[a-z]{3,10}",
                "[a-zA-Z ]{5,20}",
                Just(ScriptType::Custom),
                Just(world_setting.clone()),
                initial_state,
            )
                .prop_map(|(id, name, script_type, world_setting, initial_state)| {
                    Script::new(id, name, script_type, world_setting, initial_state)
                })
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        // Property 3: Script validation consistency
        // Feature: Nobody, Property 3: Script validation consistency
        #[test]
        fn prop_script_validation_consistency(script in arb_valid_script()) {
            let manager = ScriptManager::new();
            let result = manager.validate_script(&script);
            prop_assert!(result.is_ok(), "Valid script should pass validation");
        }

        // Test that scripts with missing realms are rejected
        #[test]
        fn prop_script_missing_realms_rejected(mut script in arb_valid_script()) {
            script.world_setting.cultivation_realms.clear();
            let manager = ScriptManager::new();
            let result = manager.validate_script(&script);
            prop_assert!(result.is_err(), "Script without realms should be rejected");
        }

        // Test that scripts with missing locations are rejected
        #[test]
        fn prop_script_missing_locations_rejected(mut script in arb_valid_script()) {
            script.world_setting.locations.clear();
            let manager = ScriptManager::new();
            let result = manager.validate_script(&script);
            prop_assert!(result.is_err(), "Script without locations should be rejected");
        }

        // Test that scripts with invalid starting age are rejected
        #[test]
        fn prop_script_invalid_age_rejected(mut script in arb_valid_script()) {
            script.initial_state.starting_age = 5;
            let manager = ScriptManager::new();
            let result = manager.validate_script(&script);
            prop_assert!(result.is_err(), "Script with invalid age should be rejected");
        }
    }
}
