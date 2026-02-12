use crate::llm_runtime_config::resolve_llm_config;
use crate::llm_service::{LLMRequest, LLMService};
use crate::models::{Element, Grade, SpiritualRoot};
use crate::novel_parser::{NovelParser, ParsedNovelData};
use crate::prompt_builder::{PromptBuilder, PromptConstraints, PromptContext, PromptTemplate};
use crate::response_validator::{ResponseValidator, ValidationConstraints};
use crate::script::{InitialState, Location, Script, ScriptType, WorldSetting};
use anyhow::{anyhow, Result};
use std::collections::HashSet;
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
        let cfg = resolve_llm_config()?;
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

    pub fn extract_novel_characters(&self, file_path: &str) -> Result<Vec<String>> {
        let parser = NovelParser::new();
        let parsed = parser
            .parse_novel_file(file_path)
            .map_err(|e| anyhow!("Failed to parse novel file: {}", e))?;

        if parsed.characters.is_empty() {
            return Err(anyhow!("未能从小说中解析出角色列表"));
        }

        Ok(parsed.characters)
    }

    pub fn load_existing_novel(&self, file_path: &str, selected_character: &str) -> Result<Script> {
        let parser = NovelParser::new();
        let parsed = parser
            .parse_novel_file(file_path)
            .map_err(|e| anyhow!("Failed to parse novel file: {}", e))?;

        let player_name = self.select_character_from_novel(&parsed, selected_character)?;
        let world_setting = self.build_world_setting_from_novel(&parsed);

        let starting_location = world_setting
            .locations
            .first()
            .map(|loc| loc.id.clone())
            .ok_or_else(|| anyhow!("无法为小说生成起始地点"))?;

        let player_spiritual_root = world_setting
            .spiritual_roots
            .first()
            .cloned()
            .unwrap_or(SpiritualRoot {
                element: Element::Fire,
                grade: Grade::Double,
                affinity: 0.6,
            });

        let initial_state = InitialState {
            player_name,
            player_spiritual_root,
            starting_location,
            starting_age: 16,
        };

        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow!("System clock error: {}", e))?
            .as_secs();
        let script = Script::new(
            format!("novel_{}", seed),
            parsed.title.clone(),
            ScriptType::ExistingNovel,
            world_setting,
            initial_state,
        );

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
                "未检测到 LLM 配置，使用本地随机剧本模板"
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
                "境界等级必须按顺序提升".to_string(),
                "初始年龄必须在 10 到 100 之间".to_string(),
            ],
            world_rules: vec![
                "剧本至少包含一个修炼境界与一个地点".to_string(),
                "initial_state.starting_location 必须存在于 world_setting.locations".to_string(),
                "所有文本字段使用中文".to_string(),
            ],
            output_schema_hint: Some(
                "返回严格 JSON，必须匹配 Script 结构，不要 markdown 包裹。".to_string(),
            ),
        };
        let context = PromptContext {
            scene: Some("生成一个可游玩的随机修仙世界剧本".to_string()),
            location: None,
            actor_name: None,
            actor_realm: None,
            actor_combat_power: None,
            history_events: Vec::new(),
            world_setting_summary: Some(
                "需要一个适合新手开局、设定自洽、可直接进入游戏的中文场景".to_string(),
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
            .map_err(|e| anyhow!("LLM 随机剧本生成失败: {}", e))?;

        let validation_constraints = ValidationConstraints {
            require_json: true,
            max_realm_level: None,
            min_combat_power: None,
            max_combat_power: None,
            max_current_age: Some(120),
        };
        self.response_validator
            .validate_response(&llm_response, &validation_constraints)
            .map_err(|e| anyhow!("生成结果校验失败: {}", e))?;

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
            "name": "随机修仙开局",
            "script_type": "RandomGenerated",
            "world_setting": {
                "cultivation_realms": [
                    { "name": "练气", "level": 1, "sub_level": 0, "power_multiplier": 1.0 },
                    { "name": "筑基", "level": 2, "sub_level": 0, "power_multiplier": 2.0 },
                    { "name": "金丹", "level": 3, "sub_level": 0, "power_multiplier": 4.0 }
                ],
                "spiritual_roots": [
                    { "element": "Fire", "grade": "Double", "affinity": 0.75 },
                    { "element": "Water", "grade": "Triple", "affinity": 0.62 },
                    { "element": "Wood", "grade": "Pseudo", "affinity": 0.58 }
                ],
                "techniques": [
                    {
                        "id": "breathing_technique",
                        "name": "基础吐纳诀",
                        "description": "适合初学者的基础修炼功法。",
                        "required_realm_level": 1,
                        "element": null
                    }
                ],
                "locations": [
                    {
                        "id": "sect_valley",
                        "name": "宗门外谷",
                        "description": "灵气温和，外门弟子常在此修炼。",
                        "spiritual_energy": 1.2
                    },
                    {
                        "id": "stone_forest",
                        "name": "乱石林",
                        "description": "怪石嶙峋，潜伏着低阶灵兽与隐秘机缘。",
                        "spiritual_energy": 1.5
                    }
                ],
                "factions": [
                    {
                        "id": "qingyun_sect",
                        "name": "青云宗",
                        "description": "以门规严谨著称的正道宗门。",
                        "power_level": 65
                    }
                ]
            },
            "initial_state": {
                "player_name": "无名弟子",
                "player_spiritual_root": { "element": "Fire", "grade": "Double", "affinity": 0.75 },
                "starting_location": "sect_valley",
                "starting_age": 16
            }
        }))
        .map_err(|e| anyhow!("Failed to build fallback random script: {}", e))?;

        self.validate_script(&script)?;
        Ok(script)
    }

    fn select_character_from_novel(
        &self,
        parsed: &ParsedNovelData,
        selected_character: &str,
    ) -> Result<String> {
        if parsed.characters.is_empty() {
            return Err(anyhow!("未能从小说中解析出角色列表"));
        }

        let trimmed = selected_character.trim();
        if trimmed.is_empty() {
            return Err(anyhow!("请选择一个角色"));
        }

        if !parsed.characters.iter().any(|c| c == trimmed) {
            return Err(anyhow!("选择的角色不存在: {}", trimmed));
        }

        Ok(trimmed.to_string())
    }

    fn build_world_setting_from_novel(&self, parsed: &ParsedNovelData) -> WorldSetting {
        let mut setting = WorldSetting::with_default_realms();
        setting.spiritual_roots = WorldSetting::with_default_spiritual_roots().spiritual_roots;
        setting.locations = self.build_locations_from_novel(&parsed.locations);
        setting.techniques = Vec::new();
        setting.factions = Vec::new();
        setting
    }

    fn build_locations_from_novel(&self, locations: &[String]) -> Vec<Location> {
        let mut results = Vec::new();
        let mut seen = HashSet::new();

        for (idx, name) in locations.iter().enumerate() {
            let base_id = self
                .normalize_identifier(name)
                .unwrap_or_else(|| format!("location_{}", idx + 1));
            let mut unique_id = base_id.clone();
            let mut suffix = 1;
            while seen.contains(&unique_id) {
                suffix += 1;
                unique_id = format!("{}_{}", base_id, suffix);
            }
            seen.insert(unique_id.clone());

            results.push(Location {
                id: unique_id,
                name: name.clone(),
                description: format!("从小说导入的地点：{}", name),
                spiritual_energy: 1.0,
            });
        }

        if results.is_empty() {
            results.push(Location {
                id: "novel_origin".to_string(),
                name: "小说起点".to_string(),
                description: "从小说导入的默认起点".to_string(),
                spiritual_energy: 1.0,
            });
        }

        results
    }

    fn normalize_identifier(&self, value: &str) -> Option<String> {
        let mut out = String::new();
        let mut last_was_sep = false;
        for ch in value.chars() {
            if ch.is_ascii_alphanumeric() {
                out.push(ch.to_ascii_lowercase());
                last_was_sep = false;
            } else if ch.is_whitespace() || ch == '-' || ch == '_' {
                if !last_was_sep && !out.is_empty() {
                    out.push('_');
                    last_was_sep = true;
                }
            }
        }

        let trimmed = out.trim_matches('_').to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
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

    #[test]
    fn test_extract_novel_characters() {
        let manager = ScriptManager::new();
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("novel.txt");
        std::fs::write(
            &file_path,
            "World: A cultivation world\nCharacter: Lin Mo\nCharacter: Su Wan\nLocation: Azure Cloud Sect\nLocation: Spirit Valley",
        )
        .unwrap();

        let characters = manager
            .extract_novel_characters(file_path.to_str().unwrap())
            .unwrap();
        assert!(characters.iter().any(|c| c == "Lin Mo"));
        assert!(characters.iter().any(|c| c == "Su Wan"));
    }

    #[test]
    fn test_load_existing_novel_builds_initial_state() {
        let manager = ScriptManager::new();
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("novel.txt");
        std::fs::write(
            &file_path,
            "World: A cultivation world\nCharacter: Lin Mo\nCharacter: Su Wan\nLocation: Azure Cloud Sect\nLocation: Spirit Valley",
        )
        .unwrap();

        let script = manager
            .load_existing_novel(file_path.to_str().unwrap(), "Lin Mo")
            .unwrap();
        assert_eq!(script.script_type, ScriptType::ExistingNovel);
        assert_eq!(script.initial_state.player_name, "Lin Mo");
        assert_eq!(script.initial_state.starting_location, "azure_cloud_sect");
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
