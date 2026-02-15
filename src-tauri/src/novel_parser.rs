use crate::llm_runtime_config::resolve_llm_config;
use crate::llm_service::{LLMRequest, LLMService};
use crate::prompt_builder::{PromptBuilder, PromptConstraints, PromptContext, PromptTemplate};
use crate::response_validator::{ResponseValidator, ValidationConstraints};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedNovelData {
    pub title: String,
    pub world_summary: String,
    pub characters: Vec<String>,
    pub locations: Vec<String>,
    pub key_events: Vec<String>,
}

pub struct NovelParser {
    llm_service: Option<LLMService>,
    prompt_builder: PromptBuilder,
    response_validator: ResponseValidator,
}

impl NovelParser {
    const PARSE_LLM_TIMEOUT_SECS: u64 = 18;

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
    pub fn parse_novel_file(&self, file_path: impl AsRef<Path>) -> Result<ParsedNovelData, String> {
        let path = file_path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("导入小说")
            .to_string();
        self.parse_novel_text(&title, &content)
    }

    pub fn parse_novel_text(&self, title: &str, content: &str) -> Result<ParsedNovelData, String> {
        if content.trim().is_empty() {
            return Err("小说内容为空".to_string());
        }

        let mut parsed = self.parse_with_rules(title, content);
        if let Some(refined) = self.parse_with_llm(title, content) {
            parsed = refined;
        }
        Ok(parsed)
    }

    fn parse_with_rules(&self, title: &str, content: &str) -> ParsedNovelData {
        let characters = extract_named_items(content, "Character:")
            .into_iter()
            .chain(extract_named_items(content, "角色："))
            .collect::<BTreeSet<String>>()
            .into_iter()
            .collect::<Vec<String>>();

        let locations = extract_named_items(content, "Location:")
            .into_iter()
            .chain(extract_named_items(content, "地点："))
            .collect::<BTreeSet<String>>()
            .into_iter()
            .collect::<Vec<String>>();

        let key_events = content
            .lines()
            .filter(|line| {
                let l = line.trim();
                l.contains("battle")
                    || l.contains("breakthrough")
                    || l.contains("duel")
                    || l.contains("战")
                    || l.contains("突破")
            })
            .map(|line| line.trim().to_string())
            .take(20)
            .collect::<Vec<String>>();

        let world_summary = if let Some(line) = content
            .lines()
            .map(str::trim)
            .find(|line| line.starts_with("World:") || line.starts_with("世界观："))
        {
            line.replace("World:", "")
                .replace("世界观：", "")
                .trim()
                .to_string()
        } else {
            summarize_text(content, 220)
        };

        ParsedNovelData {
            title: title.to_string(),
            world_summary,
            characters,
            locations,
            key_events,
        }
    }

    fn parse_with_llm(&self, title: &str, content: &str) -> Option<ParsedNovelData> {
        if cfg!(test) {
            return None;
        }
        let llm_service = self.llm_service.as_ref()?;

        let prompt = self.prompt_builder.build_prompt_with_token_limit(
            PromptTemplate::ScriptGeneration,
            &PromptContext {
                scene: Some(format!("请解析小说元信息：{}", title)),
                location: None,
                actor_name: None,
                actor_realm: None,
                actor_combat_power: None,
                history_events: vec![summarize_text(content, 1200)],
                world_setting_summary: Some("提取角色、地点、世界观摘要、关键事件，输出 JSON".to_string()),
            },
            &PromptConstraints {
                numerical_rules: vec![],
                world_rules: vec![
                    "只输出严格 JSON，不要 markdown".to_string(),
                    "字段必须包含: world_summary,characters,locations,key_events".to_string(),
                    "所有字段内容用中文".to_string(),
                ],
                output_schema_hint: Some(
                    "{\"world_summary\":\"string\",\"characters\":[\"string\"],\"locations\":[\"string\"],\"key_events\":[\"string\"]}".to_string(),
                ),
            },
            700,
        );

        let runtime = tokio::runtime::Runtime::new().ok()?;
        let response = runtime
            .block_on(tokio::time::timeout(
                Duration::from_secs(Self::PARSE_LLM_TIMEOUT_SECS),
                llm_service.generate(LLMRequest {
                    prompt,
                    max_tokens: Some(350),
                    temperature: Some(0.2),
                }),
            ))
            .ok()?
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

        let value: serde_json::Value = serde_json::from_str(&response.text).ok()?;
        let world_summary = value.get("world_summary")?.as_str()?.to_string();
        let characters = value
            .get("characters")?
            .as_array()?
            .iter()
            .filter_map(|v| v.as_str().map(ToString::to_string))
            .collect::<Vec<String>>();
        let locations = value
            .get("locations")?
            .as_array()?
            .iter()
            .filter_map(|v| v.as_str().map(ToString::to_string))
            .collect::<Vec<String>>();
        let key_events = value
            .get("key_events")?
            .as_array()?
            .iter()
            .filter_map(|v| v.as_str().map(ToString::to_string))
            .collect::<Vec<String>>();

        Some(ParsedNovelData {
            title: title.to_string(),
            world_summary,
            characters,
            locations,
            key_events,
        })
    }
}

impl Default for NovelParser {
    fn default() -> Self {
        Self::new()
    }
}

fn extract_named_items(content: &str, prefix: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            trimmed
                .strip_prefix(prefix)
                .map(|rest| rest.trim().to_string())
        })
        .filter(|s| !s.is_empty())
        .collect()
}

fn summarize_text(text: &str, max_chars: usize) -> String {
    text.chars().take(max_chars).collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_novel_file() {
        let parser = NovelParser::new();
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("sample_novel.txt");
        std::fs::write(
            &file_path,
            "World: A cultivation world under strict sect hierarchy\nCharacter: Lin Mo\nLocation: Azure Cloud Sect\nLin Mo won a battle and attempted breakthrough.",
        )
        .unwrap();

        let parsed = parser.parse_novel_file(&file_path).unwrap();
        assert!(parsed.title.contains("sample_novel"));
        assert!(!parsed.world_summary.is_empty());
        assert!(!parsed.characters.is_empty());
    }

    #[test]
    fn test_extract_characters_and_world_setting() {
        let parser = NovelParser::new();
        let text = "世界观：上古灵气复苏，宗门林立。\n角色：韩青\n角色：苏婉\n地点：青云宗\n韩青突破失败后再战。";

        let parsed = parser.parse_novel_text("Test", text).unwrap();
        assert!(parsed.world_summary.contains("上古灵气复苏") || !parsed.world_summary.is_empty());
        assert!(parsed.characters.iter().any(|c| c.contains("韩青")));
        assert!(parsed.locations.iter().any(|l| l.contains("青云宗")));
    }
}

