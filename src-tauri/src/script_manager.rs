use crate::llm_runtime_config::resolve_llm_config;
use crate::llm_service::{LLMRequest, LLMService};
use crate::models::{Element, Grade, SpiritualRoot};
use crate::novel_parser::{NovelParser, ParsedNovelData};
use crate::script::{InitialState, Location, Script, ScriptType, WorldSetting};
use anyhow::{anyhow, Result};
use std::collections::HashSet;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::Duration;

// Script manager for loading and validating scripts
pub struct ScriptManager {
    llm_service: Option<LLMService>,
}

impl ScriptManager {
    fn resolve_random_script_long_timeout_secs() -> u64 {
        std::env::var("NOBODY_RANDOM_SCRIPT_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.trim().parse::<u64>().ok())
            .map(|v| v.clamp(10, 3000))
            .unwrap_or(3000)
    }

    pub fn new() -> Self {
        Self {
            llm_service: Self::initialize_llm_service_from_env(),
        }
    }

    fn initialize_llm_service_from_env() -> Option<LLMService> {
        let cfg = resolve_llm_config()?;
        LLMService::new(cfg).ok()
    }

    pub fn with_llm_service(llm_service: LLMService) -> Self {
        Self {
            llm_service: Some(llm_service),
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
                elements: vec![Element::Fire],
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
        if let Some(llm_service) = &self.llm_service {
            return self.generate_random_script_with_llm(llm_service).await;
        }
        Err(anyhow!("未检测到可用 LLM 配置，无法生成随机剧本"))
    }

    async fn generate_random_script_with_llm(&self, llm_service: &LLMService) -> Result<Script> {
        let mut errors: Vec<String> = Vec::new();
        let fast_prompt = Self::build_random_script_seed_prompt_fast();
        let compact_prompt = Self::build_random_script_seed_prompt_compact();

        let quick_a = async {
            tokio::time::timeout(
                Duration::from_secs(8),
                llm_service.generate(LLMRequest {
                    prompt: fast_prompt.clone(),
                    max_tokens: Some(128),
                    temperature: Some(0.45),
                }),
            )
            .await
        };
        let quick_b = async {
            tokio::time::timeout(
                Duration::from_secs(8),
                llm_service.generate(LLMRequest {
                    prompt: compact_prompt.clone(),
                    max_tokens: Some(96),
                    temperature: Some(0.3),
                }),
            )
            .await
        };

        let (res_a, res_b) = tokio::join!(quick_a, quick_b);
        for (tag, result) in [("A", res_a), ("B", res_b)] {
            let llm_text = match result {
                Ok(Ok(resp)) => resp.text,
                Ok(Err(err)) => {
                    errors.push(format!("LLM 请求失败({}): {}", tag, err));
                    continue;
                }
                Err(_) => {
                    errors.push(format!("LLM 请求超时({})(8s)", tag));
                    continue;
                }
            };

            let Some(payload) = Self::extract_json_value(&llm_text) else {
                errors.push(format!("种子 JSON 解析失败({})", tag));
                continue;
            };
            if let Ok(script) = self.build_script_from_seed_payload(&payload) {
                return Ok(script);
            }
            errors.push(format!("种子脚本构建失败({})", tag));
        }

        let emergency_prompt = Self::build_random_script_seed_prompt_emergency();
        let emergency = tokio::time::timeout(
            Duration::from_secs(6),
            llm_service.generate(LLMRequest {
                prompt: emergency_prompt,
                max_tokens: Some(72),
                temperature: Some(0.2),
            }),
        )
        .await;
        match emergency {
            Ok(Ok(resp)) => {
                if let Some(payload) = Self::extract_json_value(&resp.text) {
                    if let Ok(script) = self.build_script_from_seed_payload(&payload) {
                        return Ok(script);
                    }
                    errors.push("紧急请求返回不可构建".to_string());
                } else {
                    errors.push("紧急请求返回非 JSON".to_string());
                }
            }
            Ok(Err(err)) => errors.push(format!("LLM 请求失败(C): {}", err)),
            Err(_) => errors.push("LLM 请求超时(C)(6s)".to_string()),
        }

        let long_timeout = Self::resolve_random_script_long_timeout_secs();
        let long_prompt = Self::build_random_script_seed_prompt_fast();
        let long_try = tokio::time::timeout(
            Duration::from_secs(long_timeout),
            llm_service.generate(LLMRequest {
                prompt: long_prompt,
                max_tokens: Some(180),
                temperature: Some(0.4),
            }),
        )
        .await;
        match long_try {
            Ok(Ok(resp)) => {
                if let Some(payload) = Self::extract_json_value(&resp.text) {
                    if let Ok(script) = self.build_script_from_seed_payload(&payload) {
                        return Ok(script);
                    }
                    errors.push(format!("长超时请求返回不可构建({}s)", long_timeout));
                } else {
                    errors.push(format!("长超时请求返回非 JSON({}s)", long_timeout));
                }
            }
            Ok(Err(err)) => errors.push(format!("LLM 请求失败(D): {}", err)),
            Err(_) => errors.push(format!("LLM 请求超时(D)({}s)", long_timeout)),
        }

        Err(anyhow!(
            "随机剧本生成失败（重试后仍失败）：{}",
            errors.join(" | ")
        ))
    }

    fn build_random_script_seed_prompt_fast() -> String {
        [
            "只返回 JSON，不要解释，不要 markdown。",
            "字段固定：world_name,location_name,location_name_2,faction_name,technique_name,opening_hook。",
            "值必须是中文短语，长度 2-10 字。",
            "示例格式：{\"world_name\":\"...\",\"location_name\":\"...\",\"location_name_2\":\"...\",\"faction_name\":\"...\",\"technique_name\":\"...\",\"opening_hook\":\"...\"}",
        ]
        .join("\n")
    }

    fn build_random_script_seed_prompt_compact() -> String {
        "{\"task\":\"generate_xianxia_seed\",\"return\":\"json_only\",\"fields\":[\"world_name\",\"location_name\",\"location_name_2\",\"faction_name\",\"technique_name\",\"opening_hook\"],\"lang\":\"zh\"}".to_string()
    }

    fn build_random_script_seed_prompt_emergency() -> String {
        "输出 JSON: {\"world_name\":\"\",\"location_name\":\"\",\"location_name_2\":\"\",\"faction_name\":\"\",\"technique_name\":\"\",\"opening_hook\":\"\"}".to_string()
    }

    fn extract_json_value(raw_text: &str) -> Option<serde_json::Value> {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(raw_text) {
            return Some(v);
        }
        let start = raw_text.find('{')?;
        let end = raw_text.rfind('}')?;
        serde_json::from_str::<serde_json::Value>(&raw_text[start..=end]).ok()
    }

    fn build_script_from_seed_payload(&self, payload: &serde_json::Value) -> Result<Script> {
        let get_str = |k: &str, d: &str| -> String {
            payload
                .get(k)
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or(d)
                .to_string()
        };

        let world_name = get_str("world_name", "随机修仙开局");
        let location_1 = get_str("location_name", "宗门外谷");
        let location_2 = get_str("location_name_2", "乱石林");
        let faction_name = get_str("faction_name", "青云宗");
        let technique_name = get_str("technique_name", "基础吐纳诀");

        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow!("System clock error: {}", e))?
            .as_secs();
        let id = format!("random_{}", seed);
        let loc1_id = "loc_start";
        let loc2_id = "loc_outer";

        let script = serde_json::from_value::<Script>(serde_json::json!({
            "id": id,
            "name": world_name,
            "script_type": "RandomGenerated",
            "world_setting": {
                "cultivation_realms": [
                    { "name": "炼气", "level": 1, "sub_level": 0, "power_multiplier": 1.0 },
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
                        "id": "tech_start",
                        "name": technique_name,
                        "description": "入门可修行的基础功法。",
                        "required_realm_level": 1,
                        "element": null
                    }
                ],
                "locations": [
                    {
                        "id": loc1_id,
                        "name": location_1,
                        "description": "灵气较为平稳，适合作为开局据点。",
                        "spiritual_energy": 1.2
                    },
                    {
                        "id": loc2_id,
                        "name": location_2,
                        "description": "地形复杂，潜藏机缘与风险。",
                        "spiritual_energy": 1.5
                    }
                ],
                "factions": [
                    {
                        "id": "faction_main",
                        "name": faction_name,
                        "description": "在此地具有影响力的主要势力。",
                        "power_level": 65
                    }
                ]
            },
            "initial_state": {
                "player_name": "无名弟子",
                "player_spiritual_root": { "element": "Fire", "grade": "Double", "affinity": 0.75 },
                "starting_location": loc1_id,
                "starting_age": 16
            }
        }))
        .map_err(|e| anyhow!("Failed to build seed script: {}", e))?;

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
            "name": "闅忔満淇粰寮€灞€",
            "script_type": "RandomGenerated",
            "world_setting": {
                "cultivation_realms": [
                    { "name": "缁冩皵", "level": 1, "sub_level": 0, "power_multiplier": 1.0 },
                    { "name": "绛戝熀", "level": 2, "sub_level": 0, "power_multiplier": 2.0 },
                    { "name": "閲戜腹", "level": 3, "sub_level": 0, "power_multiplier": 4.0 }
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
                "player_name": "鏃犲悕寮熷瓙",
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
                name: "灏忚璧风偣".to_string(),
                description: "浠庡皬璇村鍏ョ殑榛樿璧风偣".to_string(),
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
            } else if (ch.is_whitespace() || ch == '-' || ch == '_') && !last_was_sep && !out.is_empty() {
                out.push('_');
                last_was_sep = true;
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
                elements: vec![Element::Fire],
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
                elements: vec![Element::Fire],
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
    fn test_load_custom_script_valid_file() {
        let manager = ScriptManager::new();
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("valid_script.json");
        let script = create_valid_script();
        std::fs::write(&file_path, serde_json::to_string(&script).unwrap()).unwrap();

        let loaded = manager.load_custom_script(file_path.to_str().unwrap()).unwrap();
        assert_eq!(loaded.id, script.id);
        assert_eq!(loaded.initial_state.player_name, script.initial_state.player_name);
        assert!(manager.validate_script(&loaded).is_ok());
    }

    #[test]
    fn test_load_custom_script_invalid_json_file() {
        let manager = ScriptManager::new();
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("invalid_script.json");
        std::fs::write(&file_path, "{invalid json}").unwrap();

        let result = manager.load_custom_script(file_path.to_str().unwrap());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse script JSON"));
    }

    #[test]
    fn test_load_custom_script_missing_required_fields() {
        let manager = ScriptManager::new();
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("missing_fields.json");
        std::fs::write(
            &file_path,
            r#"{
                "id": "broken_script",
                "name": "Broken Script",
                "script_type": "custom",
                "world_setting": {
                    "cultivation_realms": [],
                    "spiritual_roots": [],
                    "techniques": [],
                    "locations": [],
                    "factions": []
                },
                "initial_state": {
                    "player_name": "Tester",
                    "player_spiritual_root": { "element": "Fire", "grade": "Double", "affinity": 0.6 },
                    "starting_location": "missing_location",
                    "starting_age": 16
                }
            }"#,
        )
        .unwrap();

        let result = manager.load_custom_script(file_path.to_str().unwrap());
        assert!(result.is_err());
        let message = result.unwrap_err().to_string();
        assert!(
            message.contains("Script validation failed")
                || message.contains("Failed to parse script JSON")
                || message.contains("missing field"),
            "unexpected error message: {}",
            message
        );
    }

    #[test]
    fn test_load_custom_script_file_not_found() {
        let manager = ScriptManager::new();
        let result = manager.load_custom_script("nonexistent_script.json");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Script file not found"));
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
    use proptest::test_runner::TestRunner;
    use proptest::strategy::ValueTree;
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
                elements: vec![element],
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

    fn arb_script_with_type(script_type: ScriptType) -> impl Strategy<Value = Script> {
        arb_world_setting().prop_flat_map(move |world_setting| {
            let initial_state = arb_initial_state(&world_setting);
            (
                "[a-z]{3,10}",
                "[a-zA-Z ]{5,20}",
                Just(script_type.clone()),
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

        // Feature: Nobody, Property 1: Script type support completeness
        #[test]
        fn prop_script_type_support_completeness(
            script_type in prop_oneof![
                Just(ScriptType::Custom),
                Just(ScriptType::RandomGenerated),
                Just(ScriptType::ExistingNovel)
            ]
        ) {
            let manager = ScriptManager::new();
            let script = arb_script_with_type(script_type)
                .new_tree(&mut TestRunner::default())
                .unwrap()
                .current();
            let result = manager.validate_script(&script);
            prop_assert!(result.is_ok(), "Supported script type should pass validation");
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


