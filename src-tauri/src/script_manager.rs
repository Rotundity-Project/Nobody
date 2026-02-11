use crate::llm_service::{LLMConfig, LLMRequest, LLMService};
use crate::prompt_builder::{PromptBuilder, PromptConstraints, PromptContext, PromptTemplate};
use crate::response_validator::{ResponseValidator, ValidationConstraints};
use crate::script::Script;
use anyhow::{anyhow, Result};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

// Script manager for loading and validating scripts
pub struct ScriptManager {
    llm_service: Option<LLMService>,
    prompt_builder: PromptBuilder,
    response_validator: ResponseValidator,
}

impl ScriptManager {
    pub fn new() -> Self {
        Self {
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
            .unwrap_or(1024);
        let temperature = std::env::var("NOBODY_LLM_TEMPERATURE")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(0.7);

        let cfg = LLMConfig {
            endpoint,
            api_key,
            model,
            max_tokens,
            temperature,
        };

        LLMService::new(cfg).ok()
    }

    pub fn with_llm_service(llm_service: LLMService) -> Self {
        Self {
            llm_service: Some(llm_service),
            prompt_builder: PromptBuilder::default(),
            response_validator: ResponseValidator::default(),
        }
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

    pub async fn generate_random_script(&self) -> Result<Script> {
        let generated = if let Some(llm_service) = &self.llm_service {
            self.generate_random_script_with_llm(llm_service).await
        } else {
            Err(anyhow!(
                "LLM service is not configured, falling back to local random script template"
            ))
        };

        match generated {
            Ok(script) => Ok(script),
            Err(_) => self.generate_fallback_random_script(),
        }
    }

    async fn generate_random_script_with_llm(&self, llm_service: &LLMService) -> Result<Script> {
        let constraints = PromptConstraints {
            numerical_rules: vec![
                "realm_level progression must be sequential".to_string(),
                "starting_age must be between 10 and 100".to_string(),
            ],
            world_rules: vec![
                "script must include at least one cultivation realm and one location".to_string(),
                "initial_state.starting_location must exist in world_setting.locations".to_string(),
            ],
            output_schema_hint: Some(
                "Return strict JSON matching Script schema. No markdown fences.".to_string(),
            ),
        };
        let context = PromptContext {
            scene: Some("Generate a random cultivation world script".to_string()),
            location: None,
            actor_name: None,
            actor_realm: None,
            actor_combat_power: None,
            history_events: Vec::new(),
            world_setting_summary: Some(
                "Need a playable beginner scenario with coherent world settings".to_string(),
            ),
        };

        let prompt = self.prompt_builder.build_prompt_with_token_limit(
            PromptTemplate::ScriptGeneration,
            &context,
            &constraints,
            700,
        );

        let llm_response = llm_service
            .generate(LLMRequest {
                prompt,
                max_tokens: Some(700),
                temperature: Some(0.7),
            })
            .await
            .map_err(|e| anyhow!("Failed to generate random script with LLM: {}", e))?;

        let validation_constraints = ValidationConstraints {
            require_json: true,
            max_realm_level: None,
            min_combat_power: None,
            max_combat_power: None,
            max_current_age: Some(120),
        };
        self.response_validator
            .validate_response(&llm_response, &validation_constraints)
            .map_err(|e| anyhow!("Generated script response failed validation: {}", e))?;

        let script = self.parse_generated_script_response(&llm_response.text)?;
        self.validate_script(&script)?;
        Ok(script)
    }

    fn parse_generated_script_response(&self, raw_text: &str) -> Result<Script> {
        if let Ok(script) = serde_json::from_str::<Script>(raw_text) {
            return Ok(script);
        }

        let json_start = raw_text
            .find('{')
            .ok_or_else(|| anyhow!("Generated response does not contain JSON object"))?;
        let json_end = raw_text
            .rfind('}')
            .ok_or_else(|| anyhow!("Generated response does not contain JSON object end"))?;

        let json_slice = &raw_text[json_start..=json_end];
        let script: Script = serde_json::from_str(json_slice)
            .map_err(|e| anyhow!("Failed to parse generated script JSON: {}", e))?;
        Ok(script)
    }

    fn generate_fallback_random_script(&self) -> Result<Script> {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow!("System clock error: {}", e))?
            .as_secs();
        let id = format!("random_{}", seed);

        let script = serde_json::from_value::<Script>(serde_json::json!({
            "id": id,
            "name": "Random Cultivation Start",
            "script_type": "RandomGenerated",
            "world_setting": {
                "cultivation_realms": [
                    { "name": "Qi Condensation", "level": 1, "sub_level": 0, "power_multiplier": 1.0 },
                    { "name": "Foundation Establishment", "level": 2, "sub_level": 0, "power_multiplier": 2.0 },
                    { "name": "Golden Core", "level": 3, "sub_level": 0, "power_multiplier": 4.0 }
                ],
                "spiritual_roots": [
                    { "element": "Fire", "grade": "Double", "affinity": 0.75 },
                    { "element": "Water", "grade": "Triple", "affinity": 0.62 },
                    { "element": "Wood", "grade": "Pseudo", "affinity": 0.58 }
                ],
                "techniques": [
                    {
                        "id": "breathing_technique",
                        "name": "Basic Breathing Technique",
                        "description": "A basic cultivation method for beginners.",
                        "required_realm_level": 1,
                        "element": null
                    }
                ],
                "locations": [
                    {
                        "id": "sect_valley",
                        "name": "Sect Valley",
                        "description": "A valley with mild spiritual energy where outer disciples train.",
                        "spiritual_energy": 1.2
                    },
                    {
                        "id": "stone_forest",
                        "name": "Stone Forest",
                        "description": "Rock formations filled with hidden dangers and minor spirit beasts.",
                        "spiritual_energy": 1.5
                    }
                ],
                "factions": [
                    {
                        "id": "qingyun_sect",
                        "name": "Qingyun Sect",
                        "description": "A medium-sized righteous sect known for strict discipline.",
                        "power_level": 65
                    }
                ]
            },
            "initial_state": {
                "player_name": "Wanderer",
                "player_spiritual_root": { "element": "Fire", "grade": "Double", "affinity": 0.75 },
                "starting_location": "sect_valley",
                "starting_age": 16
            }
        }))
        .map_err(|e| anyhow!("Failed to build fallback random script: {}", e))?;

        self.validate_script(&script)?;
        Ok(script)
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

    #[test]
    fn test_parse_generated_script_from_embedded_json() {
        let manager = ScriptManager::new();
        let script = create_valid_script();
        let raw = format!("Here is script:\n```json\n{}\n```", serde_json::to_string(&script).unwrap());

        let parsed = manager.parse_generated_script_response(&raw).unwrap();
        assert_eq!(parsed.id, script.id);
        assert_eq!(parsed.initial_state.starting_location, script.initial_state.starting_location);
    }

    #[tokio::test]
    async fn test_generate_random_script_fallback_when_llm_missing() {
        let manager = ScriptManager::new();
        let script = manager.generate_random_script().await.unwrap();

        assert_eq!(script.script_type, ScriptType::RandomGenerated);
        assert!(!script.world_setting.cultivation_realms.is_empty());
        assert!(!script.world_setting.locations.is_empty());
        assert!(manager.validate_script(&script).is_ok());
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
            Just(Grade::Pseudo),
            Just(Grade::Double),
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
            prop::sample::select(location_ids).boxed()
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

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        // Feature: Nobody, Property 2: Random script generation completeness
        #[test]
        fn prop_fallback_random_script_has_required_elements(_seed in 0u32..=1000) {
            let manager = ScriptManager::new();
            let script = manager.generate_fallback_random_script().unwrap();

            prop_assert!(!script.id.is_empty());
            prop_assert!(!script.name.is_empty());
            prop_assert!(!script.world_setting.cultivation_realms.is_empty());
            prop_assert!(!script.world_setting.locations.is_empty());
            prop_assert!(!script.initial_state.starting_location.is_empty());
            prop_assert!(manager.validate_script(&script).is_ok());
        }
    }
}
