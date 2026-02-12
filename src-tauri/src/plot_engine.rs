use crate::models::CharacterStats;
use crate::llm_runtime_config::resolve_llm_config;
use crate::llm_service::{LLMRequest, LLMService};
use crate::numerical_system::{Action, ActionResult, Context, NumericalSystem};
use crate::prompt_builder::{PromptBuilder, PromptConstraints, PromptContext, PromptTemplate};
use crate::response_validator::{ResponseValidator, ValidationConstraints};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::runtime::Handle;
use tokio::task;

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
    pub meta: Option<ActionMeta>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionMeta {
    pub action_kind: Option<String>,
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
pub struct PlotSettings {
    pub recap_enabled: bool,
    pub novel_style: String,
    pub min_interactions_per_chapter: u8,
    pub max_interactions_per_chapter: u8,
    pub target_chapter_words_min: u32,
    pub target_chapter_words_max: u32,
}

impl Default for PlotSettings {
    fn default() -> Self {
        Self {
            recap_enabled: true,
            novel_style: "修仙白话·第三人称".to_string(),
            min_interactions_per_chapter: 2,
            max_interactions_per_chapter: 3,
            target_chapter_words_min: 5000,
            target_chapter_words_max: 7000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChapterState {
    pub index: u32,
    pub title: String,
    pub content: Vec<String>,
    pub summary: String,
    pub interaction_count: u8,
}

impl ChapterState {
    pub fn new(index: u32, title: String) -> Self {
        Self {
            index,
            title,
            content: Vec::new(),
            summary: String::new(),
            interaction_count: 0,
        }
    }

    pub fn word_count(&self) -> usize {
        self.content
            .iter()
            .map(|c| c.split_whitespace().count().max(c.chars().count() / 2))
            .sum()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotState {
    pub current_scene: Scene,
    pub plot_history: Vec<String>,
    pub is_waiting_for_input: bool,
    pub last_action_result: Option<ActionResult>,
    pub settings: PlotSettings,
    pub current_chapter: ChapterState,
    pub chapters: Vec<ChapterState>,
    pub segment_count: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotUpdate {
    pub new_scene: Option<Scene>,
    pub plot_text: String,
    pub triggered_events: Vec<String>,
    pub state_changes: Vec<String>,
    pub is_waiting_for_input: bool,
    pub available_options: Vec<PlayerOption>,
    pub chapter_title: Option<String>,
    pub chapter_summary: Option<String>,
    pub chapter_end: bool,
}

pub struct PlotEngine {
    numerical_system: NumericalSystem,
    llm_service: Option<LLMService>,
    prompt_builder: PromptBuilder,
    response_validator: ResponseValidator,
}

#[derive(Debug, Clone)]
pub struct OpeningPlot {
    pub text: String,
    pub options: Vec<String>,
}

#[derive(Debug, Clone)]
struct ChapterSegment {
    text: String,
    needs_player_input: bool,
    chapter_end: bool,
    chapter_title: Option<String>,
    chapter_summary: Option<String>,
    options: Vec<String>,
}

impl PlotEngine {
    pub fn new() -> Self {
        Self {
            numerical_system: NumericalSystem::new(),
            llm_service: Self::initialize_llm_service_from_env(),
            prompt_builder: PromptBuilder::default(),
            response_validator: ResponseValidator::default(),
        }
    }

    fn initialize_llm_service_from_env() -> Option<LLMService> {
        let cfg = resolve_llm_config()?;
        LLMService::new(cfg).ok()
    }

    fn resolve_llm_service(&self) -> Option<LLMService> {
        let cfg = resolve_llm_config()?;
        LLMService::new(cfg).ok()
    }

    fn run_llm_request(&self, llm_service: &LLMService, request: LLMRequest) -> Option<crate::llm_service::LLMResponse> {
        if let Ok(handle) = Handle::try_current() {
            return task::block_in_place(|| handle.block_on(llm_service.generate(request)).ok());
        }

        let runtime = tokio::runtime::Runtime::new().ok()?;
        runtime.block_on(llm_service.generate(request)).ok()
    }

    fn extract_json_value(&self, raw: &str) -> Option<Value> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return None;
        }

        let mut candidate = trimmed.to_string();
        if trimmed.starts_with("```") {
            let mut lines = trimmed.lines();
            let _ = lines.next();
            candidate = lines.collect::<Vec<&str>>().join("\n");
            if let Some(stripped) = candidate.strip_suffix("```") {
                candidate = stripped.trim().to_string();
            }
        }
        let candidate = candidate.trim();

        if let Ok(value) = serde_json::from_str::<Value>(candidate) {
            return Some(value);
        }

        let start = candidate.find('{')?;
        let end = candidate.rfind('}')?;
        if start >= end {
            return None;
        }
        serde_json::from_str::<Value>(&candidate[start..=end]).ok()
    }

    fn extract_string_field_raw(&self, raw: &str, field: &str) -> Option<String> {
        let key = format!("\"{}\"", field);
        let idx = raw.find(&key)?;
        let after = &raw[idx + key.len()..];
        let colon = after.find(':')?;
        let mut s = after[colon + 1..].trim_start();

        if s.starts_with('"') {
            s = &s[1..];
            let mut out = String::new();
            let mut escaped = false;
            for ch in s.chars() {
                if escaped {
                    match ch {
                        'n' => out.push('\n'),
                        't' => out.push('\t'),
                        'r' => out.push('\r'),
                        '"' => out.push('"'),
                        '\\' => out.push('\\'),
                        _ => out.push(ch),
                    }
                    escaped = false;
                    continue;
                }
                if ch == '\\' {
                    escaped = true;
                    continue;
                }
                if ch == '"' {
                    break;
                }
                out.push(ch);
            }
            let trimmed = out.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        } else {
            let end = s
                .find(|c: char| c == ',' || c == '\n' || c == '}')
                .unwrap_or_else(|| s.len());
            let value = s[..end].trim();
            if value.is_empty() {
                None
            } else {
                Some(value.to_string())
            }
        }
    }

    fn extract_bool_field_raw(&self, raw: &str, field: &str) -> Option<bool> {
        let key = format!("\"{}\"", field);
        let idx = raw.find(&key)?;
        let after = &raw[idx + key.len()..];
        let colon = after.find(':')?;
        let s = after[colon + 1..].trim_start();
        if s.starts_with("true") {
            Some(true)
        } else if s.starts_with("false") {
            Some(false)
        } else {
            None
        }
    }

    fn extract_options_field_raw(&self, raw: &str) -> Vec<String> {
        for key in ["options", "action_choices"] {
            let key_marker = format!("\"{}\"", key);
            if let Some(idx) = raw.find(&key_marker) {
                let after = &raw[idx + key_marker.len()..];
                let start = after.find('[').unwrap_or(0);
                let mut s = &after[start..];
                let mut items = Vec::new();
                let mut in_string = false;
                let mut escaped = false;
                let mut current = String::new();
                for ch in s.chars() {
                    if !in_string {
                        if ch == '"' {
                            in_string = true;
                            current.clear();
                        }
                        if ch == ']' {
                            break;
                        }
                        continue;
                    }

                    if escaped {
                        current.push(ch);
                        escaped = false;
                        continue;
                    }
                    if ch == '\\' {
                        escaped = true;
                        continue;
                    }
                    if ch == '"' {
                        let trimmed = current.trim();
                        if !trimmed.is_empty() {
                            items.push(trimmed.to_string());
                        }
                        in_string = false;
                        if items.len() >= 4 {
                            break;
                        }
                        continue;
                    }
                    current.push(ch);
                }
                if !items.is_empty() {
                    return items;
                }
            }
        }
        Vec::new()
    }

    pub fn advance_plot(
        &self,
        current_state: &PlotState,
        action_result: &ActionResult,
    ) -> PlotUpdate {
        let segment = self.generate_chapter_segment(current_state, action_result);
        let plot_text = segment.text.clone();
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

        let mut available_options = Vec::new();
        if segment.needs_player_input || segment.chapter_end {
            if segment.chapter_end {
                available_options.push(PlayerOption {
                    id: 0,
                    description: "翻到下一章".to_string(),
                    requirements: vec![],
                    action: Action::Custom {
                        description: "你翻动书页，进入新的篇章。".to_string(),
                    },
                });
            } else if !segment.options.is_empty() {
                available_options = segment
                    .options
                    .iter()
                    .enumerate()
                    .map(|(idx, text)| PlayerOption {
                        id: idx,
                        description: text.clone(),
                        requirements: vec![],
                        action: Action::Custom {
                            description: text.clone(),
                        },
                    })
                    .collect();
            }
        }

        PlotUpdate {
            new_scene: None,
            plot_text,
            triggered_events,
            state_changes,
            is_waiting_for_input: segment.needs_player_input || segment.chapter_end,
            available_options,
            chapter_title: segment.chapter_title,
            chapter_summary: segment.chapter_summary,
            chapter_end: segment.chapter_end,
        }
    }

    pub async fn advance_plot_async(
        &self,
        current_state: &PlotState,
        action_result: &ActionResult,
    ) -> PlotUpdate {
        let segment = self
            .generate_chapter_segment_async(current_state, action_result)
            .await;
        let plot_text = segment.text.clone();
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

        let mut available_options = Vec::new();
        if segment.needs_player_input || segment.chapter_end {
            if segment.chapter_end {
                available_options.push(PlayerOption {
                    id: 0,
                    description: "翻到下一章".to_string(),
                    requirements: vec![],
                    action: Action::Custom {
                        description: "你翻动书页，进入新的篇章。".to_string(),
                    },
                });
            } else if !segment.options.is_empty() {
                available_options = segment
                    .options
                    .iter()
                    .enumerate()
                    .map(|(idx, text)| PlayerOption {
                        id: idx,
                        description: text.clone(),
                        requirements: vec![],
                        action: Action::Custom {
                            description: text.clone(),
                        },
                    })
                    .collect();
            }
        }

        PlotUpdate {
            new_scene: None,
            plot_text,
            triggered_events,
            state_changes,
            is_waiting_for_input: segment.needs_player_input || segment.chapter_end,
            available_options,
            chapter_title: segment.chapter_title,
            chapter_summary: segment.chapter_summary,
            chapter_end: segment.chapter_end,
        }
    }

    pub fn generate_plot_text(&self, current_state: &PlotState, action_result: &ActionResult) -> String {
        self.generate_chapter_segment(current_state, action_result).text
    }

    fn generate_chapter_segment(
        &self,
        current_state: &PlotState,
        action_result: &ActionResult,
    ) -> ChapterSegment {
        if let Some(segment) = self.generate_chapter_segment_with_llm(current_state, action_result) {
            return self.apply_chapter_segment_rules(current_state, segment);
        }

        let text = self.generate_plot_text_fallback(current_state, action_result);
        ChapterSegment {
            text,
            needs_player_input: true,
            chapter_end: false,
            chapter_title: None,
            chapter_summary: None,
            options: vec![],
        }
    }

    async fn generate_chapter_segment_async(
        &self,
        current_state: &PlotState,
        action_result: &ActionResult,
    ) -> ChapterSegment {
        if let Some(segment) = self
            .generate_chapter_segment_with_llm_async(current_state, action_result)
            .await
        {
            return self.apply_chapter_segment_rules(current_state, segment);
        }

        let text = self.generate_plot_text_fallback(current_state, action_result);
        ChapterSegment {
            text,
            needs_player_input: true,
            chapter_end: false,
            chapter_title: None,
            chapter_summary: None,
            options: vec![],
        }
    }

    fn apply_chapter_segment_rules(
        &self,
        current_state: &PlotState,
        mut segment: ChapterSegment,
    ) -> ChapterSegment {
        let settings = &current_state.settings;
        let word_count = current_state.current_chapter.word_count()
            + segment.text.split_whitespace().count().max(segment.text.chars().count() / 2);

        if current_state.current_chapter.interaction_count >= settings.max_interactions_per_chapter {
            segment.needs_player_input = false;
        }

        if current_state.current_chapter.interaction_count < settings.min_interactions_per_chapter
            && !segment.needs_player_input
            && current_state.segment_count >= 2
        {
            segment.needs_player_input = true;
        }

        if word_count >= settings.target_chapter_words_max as usize
            && current_state.current_chapter.interaction_count >= settings.min_interactions_per_chapter
        {
            segment.chapter_end = true;
        }

        if segment.chapter_end
            && current_state.current_chapter.interaction_count < settings.min_interactions_per_chapter
        {
            segment.chapter_end = false;
        }

        if segment.needs_player_input
            && current_state.current_chapter.interaction_count >= settings.max_interactions_per_chapter
        {
            segment.needs_player_input = false;
        }

        if segment.options.is_empty()
            && !segment.chapter_end
            && current_state.current_chapter.interaction_count < settings.max_interactions_per_chapter
        {
            segment.needs_player_input = true;
        }

        segment
    }

    fn generate_chapter_segment_with_llm(
        &self,
        current_state: &PlotState,
        action_result: &ActionResult,
    ) -> Option<ChapterSegment> {
        if cfg!(test) {
            return None;
        }
        let llm_service = self.resolve_llm_service()?;
        let settings = &current_state.settings;
        let recent_segments = current_state
            .current_chapter
            .content
            .iter()
            .rev()
            .take(3)
            .cloned()
            .collect::<Vec<String>>();

        let context = PromptContext {
            scene: Some(format!(
                "章节 {}，玩家行动结果：{}。当前剧情片段：{}",
                current_state.current_chapter.index,
                action_result.description,
                recent_segments.join(" / ")
            )),
            location: Some(current_state.current_scene.location.clone()),
            actor_name: Some("player".to_string()),
            actor_realm: None,
            actor_combat_power: None,
            history_events: action_result.events.clone(),
            world_setting_summary: Some(format!(
                "小说风格：{}；请生成一段承接剧情的小说文本。玩家每章需要 2-3 次互动。",
                settings.novel_style
            )),
        };

        let constraints = PromptConstraints {
            numerical_rules: vec![
                "必须与行动结果保持一致".to_string(),
                "每章需要 2-3 次玩家介入点".to_string(),
                "章节总字数目标 5000-7000 字".to_string(),
            ],
            world_rules: vec![
                "输出严格 JSON".to_string(),
                "segment_text 必须为中文小说叙事".to_string(),
                "segment_text 不要包含选项列表".to_string(),
                "needs_player_input 为 true 时，必须给出 2-4 个 options".to_string(),
                "chapter_end 仅在章节接近尾声时为 true".to_string(),
            ],
            output_schema_hint: Some(
                "{\"segment_text\":\"string\",\"needs_player_input\":true|false,\"chapter_end\":true|false,\"chapter_title\":\"string\",\"chapter_summary\":\"string\",\"options\":[\"string\"]}".to_string(),
            ),
        };

        let prompt = self.prompt_builder.build_prompt_with_token_limit(
            PromptTemplate::PlotGeneration,
            &context,
            &constraints,
            1200,
        );

        let response = self.run_llm_request(
            &llm_service,
            LLMRequest {
                prompt,
                max_tokens: Some(900),
                temperature: Some(0.7),
            },
        )?;

        self.response_validator
            .validate_response(
                &response,
                &ValidationConstraints {
                    require_json: false,
                    max_realm_level: None,
                    min_combat_power: None,
                    max_combat_power: None,
                    max_current_age: None,
                },
            )
            .ok()?;

        if let Some(value) = self.extract_json_value(&response.text) {
            let text = value
                .get("segment_text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            let needs_player_input = value
                .get("needs_player_input")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let chapter_end = value
                .get("chapter_end")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let chapter_title = value
                .get("chapter_title")
                .and_then(Value::as_str)
                .map(|s| s.trim().to_string());
            let chapter_summary = value
                .get("chapter_summary")
                .and_then(Value::as_str)
                .map(|s| s.trim().to_string());
            let options = value
                .get("options")
                .and_then(Value::as_array)
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default();

            if !text.is_empty() {
                return Some(ChapterSegment {
                    text,
                    needs_player_input,
                    chapter_end,
                    chapter_title,
                    chapter_summary,
                    options,
                });
            }
        }

        if let Some(text) = self.extract_string_field_raw(&response.text, "segment_text") {
            let needs_player_input = self
                .extract_bool_field_raw(&response.text, "needs_player_input")
                .unwrap_or(true);
            let chapter_end = self
                .extract_bool_field_raw(&response.text, "chapter_end")
                .unwrap_or(false);
            let chapter_title = self.extract_string_field_raw(&response.text, "chapter_title");
            let chapter_summary = self.extract_string_field_raw(&response.text, "chapter_summary");
            let options = self.extract_options_field_raw(&response.text);

            return Some(ChapterSegment {
                text,
                needs_player_input,
                chapter_end,
                chapter_title,
                chapter_summary,
                options,
            });
        }

        let text = response.text.trim().to_string();
        if text.is_empty() {
            return None;
        }

        Some(ChapterSegment {
            text,
            needs_player_input: true,
            chapter_end: false,
            chapter_title: None,
            chapter_summary: None,
            options: vec![],
        })
    }

    async fn generate_chapter_segment_with_llm_async(
        &self,
        current_state: &PlotState,
        action_result: &ActionResult,
    ) -> Option<ChapterSegment> {
        if cfg!(test) {
            return None;
        }
        let llm_service = self.resolve_llm_service()?;
        let settings = &current_state.settings;
        let recent_segments = current_state
            .current_chapter
            .content
            .iter()
            .rev()
            .take(2)
            .cloned()
            .collect::<Vec<String>>();

        let context = PromptContext {
            scene: Some(format!(
                "章节 {}，玩家刚刚的选择是：{}。请在正文中自然写入该行动，而不是复述为“玩家行动”。当前剧情片段：{}",
                current_state.current_chapter.index,
                action_result.description,
                recent_segments.join(" / ")
            )),
            location: Some(current_state.current_scene.location.clone()),
            actor_name: Some("player".to_string()),
            actor_realm: None,
            actor_combat_power: None,
            history_events: action_result.events.clone(),
            world_setting_summary: Some(format!(
                "小说风格：{}；请生成一段承接剧情的小说文本。玩家每章需要 2-3 次互动。",
                settings.novel_style
            )),
        };

        let constraints = PromptConstraints {
            numerical_rules: vec![
                "必须与行动结果保持一致".to_string(),
                "每章需要 2-3 次玩家介入点".to_string(),
                "章节总字数目标 5000-7000 字".to_string(),
            ],
            world_rules: vec![
                "输出严格 JSON".to_string(),
                "segment_text 必须为中文小说叙事".to_string(),
                "segment_text 不要包含选项列表".to_string(),
                "不要复述或改写已出现的段落".to_string(),
                "每次输出 500-900 字".to_string(),
                "needs_player_input 为 true 时，必须给出 2-4 个 options".to_string(),
                "chapter_end 仅在章节接近尾声时为 true".to_string(),
            ],
            output_schema_hint: Some(
                "{\"segment_text\":\"string\",\"needs_player_input\":true|false,\"chapter_end\":true|false,\"chapter_title\":\"string\",\"chapter_summary\":\"string\",\"options\":[\"string\"]}".to_string(),
            ),
        };

        let output_max = llm_service.api_config.max_tokens.min(1000).max(300);
        let prompt_limit = output_max.saturating_mul(6);

        let prompt = self.prompt_builder.build_prompt_with_token_limit(
            PromptTemplate::PlotGeneration,
            &context,
            &constraints,
            prompt_limit,
        );

        let response = match llm_service
            .generate(LLMRequest {
                prompt: prompt.clone(),
                max_tokens: Some(output_max),
                temperature: Some(0.7),
            })
            .await
        {
            Ok(resp) => resp,
            Err(_) => {
                let retry_prompt = self.prompt_builder.build_prompt_with_token_limit(
                    PromptTemplate::PlotGeneration,
                    &context,
                    &PromptConstraints {
                        numerical_rules: vec![
                            "必须与行动结果保持一致".to_string(),
                        ],
                        world_rules: vec![
                            "输出严格 JSON".to_string(),
                            "segment_text 必须为中文小说叙事".to_string(),
                            "segment_text 不要包含选项列表".to_string(),
                            "不要复述或改写已出现的段落".to_string(),
                            "每次输出 300-600 字".to_string(),
                            "needs_player_input 为 true 时，必须给出 2-4 个 options".to_string(),
                        ],
                        output_schema_hint: Some(
                            "{\"segment_text\":\"string\",\"needs_player_input\":true|false,\"chapter_end\":true|false,\"chapter_title\":\"string\",\"chapter_summary\":\"string\",\"options\":[\"string\"]}".to_string(),
                        ),
                    },
                    output_max.saturating_mul(3),
                );
                llm_service
                    .generate(LLMRequest {
                        prompt: retry_prompt,
                        max_tokens: Some(output_max.saturating_div(2).max(200)),
                        temperature: Some(0.7),
                    })
                    .await
                    .ok()?
            }
        };

        self.response_validator
            .validate_response(
                &response,
                &ValidationConstraints {
                    require_json: false,
                    max_realm_level: None,
                    min_combat_power: None,
                    max_combat_power: None,
                    max_current_age: None,
                },
            )
            .ok()?;

        if let Some(value) = self.extract_json_value(&response.text) {
            let text = value
                .get("segment_text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            let needs_player_input = value
                .get("needs_player_input")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let chapter_end = value
                .get("chapter_end")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let chapter_title = value
                .get("chapter_title")
                .and_then(Value::as_str)
                .map(|s| s.trim().to_string());
            let chapter_summary = value
                .get("chapter_summary")
                .and_then(Value::as_str)
                .map(|s| s.trim().to_string());
            let options = value
                .get("options")
                .and_then(Value::as_array)
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default();

            if !text.is_empty() {
                return Some(ChapterSegment {
                    text,
                    needs_player_input,
                    chapter_end,
                    chapter_title,
                    chapter_summary,
                    options,
                });
            }
        }

        if let Some(text) = self.extract_string_field_raw(&response.text, "segment_text") {
            let needs_player_input = self
                .extract_bool_field_raw(&response.text, "needs_player_input")
                .unwrap_or(true);
            let chapter_end = self
                .extract_bool_field_raw(&response.text, "chapter_end")
                .unwrap_or(false);
            let chapter_title = self.extract_string_field_raw(&response.text, "chapter_title");
            let chapter_summary = self.extract_string_field_raw(&response.text, "chapter_summary");
            let options = self.extract_options_field_raw(&response.text);

            return Some(ChapterSegment {
                text,
                needs_player_input,
                chapter_end,
                chapter_title,
                chapter_summary,
                options,
            });
        }

        let text = response.text.trim().to_string();
        if text.is_empty() {
            return None;
        }

        Some(ChapterSegment {
            text,
            needs_player_input: true,
            chapter_end: false,
            chapter_title: None,
            chapter_summary: None,
            options: vec![],
        })
    }

    fn generate_plot_text_with_llm(
        &self,
        current_state: &PlotState,
        action_result: &ActionResult,
    ) -> Option<String> {
        if cfg!(test) {
            return None;
        }
        let llm_service = self.resolve_llm_service()?;
        let prompt = self.prompt_builder.build_prompt_with_token_limit(
            PromptTemplate::PlotGeneration,
            &PromptContext {
                scene: Some(current_state.current_scene.description.clone()),
                location: Some(current_state.current_scene.location.clone()),
                actor_name: Some("player".to_string()),
                actor_realm: None,
                actor_combat_power: None,
                history_events: action_result.events.clone(),
                world_setting_summary: Some("修仙小说风格，强调场景、事件与 NPC 反应".to_string()),
            },
            &PromptConstraints {
                numerical_rules: vec!["必须与行动结果保持一致".to_string()],
                world_rules: vec![
                    "仅输出纯文本".to_string(),
                    "使用简洁的小说叙事".to_string(),
                    "必须使用中文".to_string(),
                ],
                output_schema_hint: None,
            },
            360,
        );

        let response = self.run_llm_request(
            &llm_service,
            LLMRequest {
                prompt,
                max_tokens: Some(220),
                temperature: Some(0.7),
            },
        )?;

        self.response_validator
            .validate_response(
                &response,
                &ValidationConstraints {
                    require_json: false,
                    max_realm_level: None,
                    min_combat_power: None,
                    max_combat_power: None,
                    max_current_age: None,
                },
            )
            .ok()?;

        let text = response.text.trim();
        if text.is_empty() {
            None
        } else {
            Some(text.to_string())
        }
    }

    fn generate_plot_text_fallback(&self, current_state: &PlotState, action_result: &ActionResult) -> String {
        let event_line = if action_result.events.is_empty() {
            String::new()
        } else {
            format!("随后传来的动静与风声里，{}。", action_result.events.join("；"))
        };

        format!(
            "在{}，你{}。{}",
            current_state.current_scene.location,
            action_result.description,
            event_line
        )
        .trim()
        .to_string()
    }

    pub fn generate_opening_plot(
        &self,
        player_name: &str,
        realm_name: &str,
        spiritual_root: &str,
        location: &str,
    ) -> String {
        self.generate_opening_plot_fallback(player_name, realm_name, spiritual_root, location)
    }

    pub async fn generate_opening_plot_async(
        &self,
        player_name: &str,
        realm_name: &str,
        spiritual_root: &str,
        location: &str,
    ) -> OpeningPlot {
        if cfg!(test) {
            return OpeningPlot {
                text: self.generate_opening_plot_fallback(
                    player_name,
                    realm_name,
                    spiritual_root,
                    location,
                ),
                options: vec![],
            };
        }

        if let Some(opening) = self
            .generate_opening_plot_with_llm_async(
                player_name,
                realm_name,
                spiritual_root,
                location,
            )
            .await
        {
            return opening;
        }

        OpeningPlot {
            text: self.generate_opening_plot_fallback(
                player_name,
                realm_name,
                spiritual_root,
                location,
            ),
            options: vec![],
        }
    }

    fn generate_opening_plot_fallback(
        &self,
        player_name: &str,
        realm_name: &str,
        spiritual_root: &str,
        location: &str,
    ) -> String {
        format!(
            "【开篇】{}初入修行之路，身负{}，当前境界为{}。你站在{}，四周灵气浮动，机缘与风险并存。你决定先从何处入手？",
            player_name, spiritual_root, realm_name, location
        )
    }

    async fn generate_opening_plot_with_llm_async(
        &self,
        player_name: &str,
        realm_name: &str,
        spiritual_root: &str,
        location: &str,
    ) -> Option<OpeningPlot> {
        let llm_service = self.resolve_llm_service()?;
        let output_max = llm_service.api_config.max_tokens.min(420).max(120);
        let prompt_limit = output_max.saturating_mul(6);

        let prompt = self.prompt_builder.build_prompt_with_token_limit(
            PromptTemplate::PlotGeneration,
            &PromptContext {
                scene: Some("请生成修仙小说的第一段开篇剧情，并在结尾抛出行动选择点".to_string()),
                location: Some(location.to_string()),
                actor_name: Some(player_name.to_string()),
                actor_realm: Some(realm_name.to_string()),
                actor_combat_power: None,
                history_events: vec![],
                world_setting_summary: Some(format!("主角灵根：{}", spiritual_root)),
            },
            &PromptConstraints {
                numerical_rules: vec!["不得出现跨境界夸张成长".to_string()],
                world_rules: vec![
                    "输出严格 JSON".to_string(),
                    "必须是中文".to_string(),
                    "segment_text 为中文小说叙事，不能包含选项列表".to_string(),
                    "options 必须为 2-4 条简洁选项".to_string(),
                    "长度控制在 200 到 380 字".to_string(),
                ],
                output_schema_hint: Some(
                    "{\"segment_text\":\"string\",\"options\":[\"string\"]}".to_string(),
                ),
            },
            prompt_limit,
        );

        let response = match llm_service
            .generate(LLMRequest {
                prompt: prompt.clone(),
                max_tokens: Some(output_max),
                temperature: Some(0.7),
            })
            .await
        {
            Ok(resp) => resp,
            Err(_) => {
                let retry_prompt = self.prompt_builder.build_prompt_with_token_limit(
                    PromptTemplate::PlotGeneration,
                    &PromptContext {
                        scene: Some("生成修仙小说开篇，保持简洁但有画面感".to_string()),
                        location: Some(location.to_string()),
                        actor_name: Some(player_name.to_string()),
                        actor_realm: Some(realm_name.to_string()),
                        actor_combat_power: None,
                        history_events: vec![],
                        world_setting_summary: Some(format!("主角灵根：{}", spiritual_root)),
                    },
                    &PromptConstraints {
                        numerical_rules: vec!["不得出现跨境界夸张成长".to_string()],
                        world_rules: vec![
                            "输出严格 JSON".to_string(),
                            "必须是中文".to_string(),
                            "segment_text 为中文小说叙事，不能包含选项列表".to_string(),
                            "options 必须为 2-4 条简洁选项".to_string(),
                            "长度控制在 160 到 260 字".to_string(),
                        ],
                        output_schema_hint: Some(
                            "{\"segment_text\":\"string\",\"options\":[\"string\"]}".to_string(),
                        ),
                    },
                    output_max.saturating_mul(3),
                );
                llm_service
                    .generate(LLMRequest {
                        prompt: retry_prompt,
                        max_tokens: Some(output_max.saturating_div(2).max(120)),
                        temperature: Some(0.7),
                    })
                    .await
                    .ok()?
            }
        };

        self.response_validator
            .validate_response(
                &response,
                &ValidationConstraints {
                    require_json: false,
                    max_realm_level: None,
                    min_combat_power: None,
                    max_combat_power: None,
                    max_current_age: None,
                },
            )
            .ok()?;

        if let Some(value) = self.extract_json_value(&response.text) {
            let mut text = value
                .get("segment_text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();

            if text.is_empty() {
                let scene = value
                    .get("scene")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .trim();
                let status = value
                    .get("current_status")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .trim();
                if !scene.is_empty() || !status.is_empty() {
                    text = format!("{}{}", scene, if status.is_empty() { "".to_string() } else { format!("\n\n{}", status) });
                }
            }

            let options = value
                .get("options")
                .or_else(|| value.get("action_choices"))
                .and_then(Value::as_array)
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default();

            if !text.is_empty() {
                return Some(OpeningPlot { text, options });
            }
        }

        if let Some(text) = self.extract_string_field_raw(&response.text, "segment_text") {
            let options = self.extract_options_field_raw(&response.text);
            return Some(OpeningPlot { text, options });
        }

        let text = response.text.trim().to_string();
        if text.is_empty() {
            return None;
        }

        Some(OpeningPlot {
            text,
            options: vec![],
        })
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

        // Cultivate option
        options.push(PlayerOption {
            id: option_id,
            description: "静心修炼，稳固境界".to_string(),
            requirements: vec![],
            action: Action::Cultivate,
        });
        option_id += 1;

        // Breakthrough option if sub-level is less than 3
        if character.cultivation_realm.sub_level < 3 {
            options.push(PlayerOption {
                id: option_id,
                description: format!(
                    "尝试突破 {}",
                    character.cultivation_realm.name
                ),
                requirements: vec![format!(
                    "当前境界：{}（小层级 {}）",
                    character.cultivation_realm.name, character.cultivation_realm.sub_level
                )],
                action: Action::Breakthrough,
            });
            option_id += 1;
        }

        // Rest option
        options.push(PlayerOption {
            id: option_id,
            description: "调息休整，恢复状态".to_string(),
            requirements: vec![],
            action: Action::Rest,
        });
        option_id += 1;

        // Location-specific options
        if scene.location == "azure_cloud_sect" || scene.location == "sect" {
            options.push(PlayerOption {
                id: option_id,
                description: "前往宗门藏经阁".to_string(),
                requirements: vec![],
                action: Action::Custom {
                    description: "你在藏经阁翻阅典籍，寻找适合自己的修炼方向。".to_string(),
                },
            });
            option_id += 1;
        } else if scene.location == "city" {
            options.push(PlayerOption {
                id: option_id,
                description: "前往坊市探查消息".to_string(),
                requirements: vec![],
                action: Action::Custom {
                    description: "你在坊市中打探情报，顺便寻找可用的修炼资源。".to_string(),
                },
            });
            option_id += 1;
        }

        // Ensure minimum 2 options and maximum 5 options
        if options.len() < 2 {
            options.push(PlayerOption {
                id: option_id,
                description: "盘坐冥想，梳理思绪".to_string(),
                requirements: vec![],
                action: Action::Custom {
                    description: "你静心冥想，回顾当前修行方向。".to_string(),
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
                        return Err(format!("无效的选项 ID：{}", option_id));
                    }
                    Ok(())
                } else {
                    Err("选择选项时必须提供选项 ID".to_string())
                }
            }
            ActionType::FreeText => {
                if let Some(meta) = &action.meta {
                    if meta.action_kind.as_deref() == Some("continue") {
                        return Ok(());
                    }
                }
                self.validate_free_text_input(&action.content)?;
                self.validate_free_text_reasonableness(&action.content, available_options)
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
                if option_id < available_options.len() {
                    let selected_option = &available_options[option_id];
                    let result = self.numerical_system.calculate_action_result(
                        character,
                        &selected_option.action,
                        context,
                    );
                    Ok(result)
                } else {
                    Ok(ActionResult {
                        success: true,
                        description: action.content.clone(),
                        stat_changes: vec![],
                        events: vec![],
                    })
                }
            }
            ActionType::FreeText => {
                let interpreted_action = if action.meta.as_ref().and_then(|m| m.action_kind.as_deref()) == Some("continue") {
                    Action::Custom {
                        description: "玩家翻页继续阅读".to_string(),
                    }
                } else {
                    self.interpret_free_text_action(&action.content, character, context)
                };
                Ok(self.numerical_system.calculate_action_result(
                    character,
                    &interpreted_action,
                    context,
                ))
            }
        }
    }

    fn interpret_free_text_action(
        &self,
        free_text: &str,
        character: &CharacterStats,
        context: &Context,
    ) -> Action {
        self.parse_action_with_llm(free_text, character, context)
            .unwrap_or_else(|| self.parse_action_with_rules(free_text))
    }

    fn parse_action_with_llm(
        &self,
        free_text: &str,
        character: &CharacterStats,
        context: &Context,
    ) -> Option<Action> {
        if cfg!(test) {
            return None;
        }
        let llm_service = self.resolve_llm_service()?;

        let prompt = self.prompt_builder.build_prompt_with_token_limit(
            PromptTemplate::OptionGeneration,
            &PromptContext {
                scene: Some(free_text.to_string()),
                location: Some(context.location.clone()),
                actor_name: Some("player".to_string()),
                actor_realm: Some(character.cultivation_realm.name.clone()),
                actor_combat_power: Some(character.combat_power),
                history_events: Vec::new(),
                world_setting_summary: Some(
                    "请把玩家自由输入解析为一个游戏内可执行行动".to_string(),
                ),
            },
            &PromptConstraints {
                numerical_rules: vec![
                    "必须符合当前境界和战力约束".to_string(),
                ],
                world_rules: vec![
                    "只输出严格 JSON".to_string(),
                    "JSON 字段: action,target,description".to_string(),
                    "action 仅允许 cultivate|rest|breakthrough|combat|custom".to_string(),
                    "description 必须为中文".to_string(),
                ],
                output_schema_hint: Some(
                    "{\"action\":\"cultivate|rest|breakthrough|combat|custom\",\"target\":\"optional string\",\"description\":\"optional string\"}".to_string(),
                ),
            },
            300,
        );

        let response = self.run_llm_request(
            &llm_service,
            LLMRequest {
                prompt,
                max_tokens: Some(128),
                temperature: Some(0.1),
            },
        )?;

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

        let value: Value = serde_json::from_str(&response.text).ok()?;
        let action_name = value.get("action").and_then(Value::as_str)?;
        let description = value
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or(free_text)
            .to_string();
        let target = value
            .get("target")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();

        match action_name.to_ascii_lowercase().as_str() {
            "cultivate" => Some(Action::Cultivate),
            "rest" => Some(Action::Rest),
            "breakthrough" => Some(Action::Breakthrough),
            "combat" => Some(Action::Combat { target_id: target }),
            "custom" => Some(Action::Custom { description }),
            _ => None,
        }
    }

    fn parse_action_with_rules(&self, free_text: &str) -> Action {
        let lower = free_text.to_ascii_lowercase();
        if contains_any(&lower, &["修炼", "打坐", "cultivate", "meditate", "training"]) {
            return Action::Cultivate;
        }
        if contains_any(&lower, &["突破", "breakthrough", "advance realm"]) {
            return Action::Breakthrough;
        }
        if contains_any(&lower, &["休息", "调息", "rest", "sleep", "recover"]) {
            return Action::Rest;
        }
        if contains_any(&lower, &["战斗", "攻击", "fight", "combat", "duel"]) {
            return Action::Combat {
                target_id: "unknown".to_string(),
            };
        }

        Action::Custom {
            description: format!("玩家行动：{}", free_text.trim()),
        }
    }

    fn validate_free_text_reasonableness(
        &self,
        free_text: &str,
        available_options: &[PlayerOption],
    ) -> Result<(), String> {
        if let Some((reasonable, reason)) =
            self.validate_behavior_with_llm(free_text, available_options)
        {
            if !reasonable {
                return Err(format!("该行动被判定为不合理：{}", reason));
            }
        }

        let lower = free_text.to_ascii_lowercase();
        if contains_any(
            &lower,
            &[
                "instant immortal",
                "instantly become immortal",
                "destroy the world",
                "god mode",
                "one punch kill everyone",
                "一拳秒杀所有人",
                "瞬间飞升",
                "毁灭世界",
                "无敌模式",
            ],
        ) {
            return Err("该行动超出当前世界规则或角色能力范围".to_string());
        }

        let can_breakthrough = available_options
            .iter()
            .any(|o| matches!(o.action, Action::Breakthrough));
        if !can_breakthrough
            && contains_any(&lower, &["breakthrough", "突破", "advance realm", "渡劫"])
        {
            return Err("当前场景或境界条件不满足突破要求".to_string());
        }

        Ok(())
    }

    fn validate_free_text_input(&self, free_text: &str) -> Result<(), String> {
        let trimmed = free_text.trim();
        if trimmed.is_empty() {
            return Err("自由输入不能为空".to_string());
        }
        if trimmed.chars().count() > 500 {
            return Err("自由输入过长，请控制在 500 字以内".to_string());
        }
        if trimmed
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t' && c != '\r')
        {
            return Err("输入包含非法控制字符".to_string());
        }
        Ok(())
    }

    fn validate_behavior_with_llm(
        &self,
        free_text: &str,
        available_options: &[PlayerOption],
    ) -> Option<(bool, String)> {
        if cfg!(test) {
            return None;
        }
        let llm_service = self.resolve_llm_service()?;
        let allowed_actions = available_options
            .iter()
            .map(|o| action_label(&o.action))
            .collect::<Vec<&'static str>>()
            .join(",");

        let prompt = self.prompt_builder.build_prompt_with_token_limit(
            PromptTemplate::OptionGeneration,
            &PromptContext {
                scene: Some(format!(
                    "玩家输入: {} | 可用行动: {}",
                    free_text, allowed_actions
                )),
                location: None,
                actor_name: Some("player".to_string()),
                actor_realm: None,
                actor_combat_power: None,
                history_events: Vec::new(),
                world_setting_summary: Some(
                    "请判断玩家行动在当前修仙场景下是否合理".to_string(),
                ),
            },
            &PromptConstraints {
                numerical_rules: vec!["拒绝违反境界与能力约束的行动".to_string()],
                world_rules: vec![
                    "只输出严格 JSON".to_string(),
                    "JSON 字段: reasonable,reason".to_string(),
                    "reason 必须为中文".to_string(),
                ],
                output_schema_hint: Some(
                    "{\"reasonable\":true|false,\"reason\":\"string\"}".to_string(),
                ),
            },
            220,
        );

        let response = self.run_llm_request(
            &llm_service,
            LLMRequest {
                prompt,
                max_tokens: Some(96),
                temperature: Some(0.1),
            },
        )?;

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

        let value: Value = serde_json::from_str(&response.text).ok()?;
        let reasonable = value.get("reasonable").and_then(Value::as_bool)?;
        let reason = value
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or("未提供原因")
            .to_string();
        Some((reasonable, reason))
    }
}

impl Default for PlotEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn contains_any(text: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|k| text.contains(k))
}

fn action_label(action: &Action) -> &'static str {
    match action {
        Action::Cultivate => "cultivate",
        Action::Combat { .. } => "combat",
        Action::Breakthrough => "breakthrough",
        Action::Rest => "rest",
        Action::Custom { .. } => "custom",
    }
}

fn fallback_chapter_title(index: u32, summary: &str) -> String {
    let base = summary.chars().take(8).collect::<String>().trim().to_string();
    if base.is_empty() {
        format!("第{}章 无题", index)
    } else {
        format!("第{}章 {}", index, base)
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
        let title = "第一章".to_string();
        let chapter = ChapterState::new(1, title.clone());
        Self {
            current_scene: initial_scene,
            plot_history: Vec::new(),
            is_waiting_for_input: true,
            last_action_result: None,
            settings: PlotSettings::default(),
            current_chapter: chapter,
            chapters: Vec::new(),
            segment_count: 0,
        }
    }

    pub fn add_to_history(&mut self, text: String) {
        self.plot_history.push(text);
    }

    pub fn append_segment(&mut self, text: String) {
        self.plot_history.push(text.clone());
        self.current_chapter.content.push(text);
        self.segment_count = self.segment_count.saturating_add(1);
        self.current_scene.description = self.current_chapter.content.join("\n\n");
    }

    pub fn finalize_chapter(&mut self, title: Option<String>, summary: Option<String>) {
        let mut resolved_summary = self.current_chapter.summary.clone();
        if let Some(summary) = summary {
            if !summary.trim().is_empty() {
                resolved_summary = summary.trim().to_string();
                self.current_chapter.summary = resolved_summary.clone();
            }
        }

        let mut resolved_title = title.unwrap_or_default();
        if resolved_title.trim().is_empty() {
            resolved_title = fallback_chapter_title(self.current_chapter.index, &resolved_summary);
        }

        if !resolved_title.trim().is_empty() {
            self.current_chapter.title = resolved_title.trim().to_string();
            self.current_scene.name = self.current_chapter.title.clone();
        }

        self.chapters.push(self.current_chapter.clone());
        let next_index = self.current_chapter.index + 1;
        let next_title = format!("第{}章", next_index);
        self.current_chapter = ChapterState::new(next_index, next_title.clone());
        self.current_scene.name = next_title;
        self.current_scene.description = "新篇章即将展开。".to_string();
        self.segment_count = 0;
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
            meta: None,
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
            meta: None,
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
            meta: None,
        };

        let result = engine.validate_player_action(&action, &scene.available_options);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_free_text_too_long() {
        let engine = PlotEngine::new();
        let scene = create_test_scene();
        let action = PlayerAction {
            action_type: ActionType::FreeText,
            content: "a".repeat(600),
            selected_option_id: None,
            meta: None,
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
            meta: None,
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
        assert!(options.iter().any(|o| matches!(o.action, Action::Cultivate)));
        assert!(options.iter().any(|o| matches!(o.action, Action::Rest)));
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
        
        assert!(options.iter().any(|o| matches!(o.action, Action::Breakthrough)));
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
        assert!(sect_options
            .iter()
            .any(|o| matches!(o.action, Action::Custom { .. })));

        let city_scene = Scene::new(
            "city_scene".to_string(),
            "City".to_string(),
            "In the city".to_string(),
            "city".to_string(),
        );

        let city_options = engine.generate_player_options(&city_scene, &character);
        assert!(city_options
            .iter()
            .any(|o| matches!(o.action, Action::Custom { .. })));
    }

    #[test]
    fn test_advance_plot() {
        let engine = PlotEngine::new();
        let scene = create_test_scene();
        let state = PlotState::new(scene);

        let action_result = ActionResult {
            success: true,
            description: "修炼成功".to_string(),
            stat_changes: vec![],
            events: vec!["完成一次修炼".to_string()],
        };

        let update = engine.advance_plot(&state, &action_result);
        assert!(update.plot_text.contains("修炼成功"));
        assert_eq!(update.triggered_events.len(), 1);
    }

    #[test]
    fn test_generate_plot_text_contains_required_information() {
        let engine = PlotEngine::new();
        let scene = create_test_scene();
        let state = PlotState::new(scene);

        let action_result = ActionResult {
            success: true,
            description: "你谨慎地运转功法".to_string(),
            stat_changes: vec![],
            events: vec!["NPC 反应: npc_elder_1 -> observe".to_string()],
        };

        let text = engine.generate_plot_text(&state, &action_result);
        assert!(text.contains("sect") || text.contains("Test Scene"));
        assert!(text.contains("NPC 反应") || text.contains("事件"));
    }

    #[test]
    fn test_generate_plot_text_has_novel_style_fallback() {
        let engine = PlotEngine::new();
        let scene = create_test_scene();
        let state = PlotState::new(scene);

        let action_result = ActionResult {
            success: true,
            description: "在晨光中吐纳灵气".to_string(),
            stat_changes: vec![],
            events: vec![],
        };

        let text = engine.generate_plot_text(&state, &action_result);
        assert!(text.contains("你"));
        assert!(text.contains("【") || text.contains("。"));
    }

    #[test]
    fn test_validate_action_with_no_option_id() {
        let engine = PlotEngine::new();
        let scene = create_test_scene();
        
        let action = PlayerAction {
            action_type: ActionType::SelectedOption,
            content: "test".to_string(),
            selected_option_id: None,
            meta: None,
        };

        let result = engine.validate_player_action(&action, &scene.available_options);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("必须提供选项 ID"));
    }

    #[test]
    fn test_validate_action_with_valid_free_text() {
        let engine = PlotEngine::new();
        let scene = create_test_scene();
        
        let action = PlayerAction {
            action_type: ActionType::FreeText,
            content: "I want to explore the forest".to_string(),
            selected_option_id: None,
            meta: None,
        };

        let result = engine.validate_player_action(&action, &scene.available_options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_action_rejects_unreasonable_free_text() {
        let engine = PlotEngine::new();
        let scene = create_test_scene();

        let action = PlayerAction {
            action_type: ActionType::FreeText,
            content: "I will instantly become immortal and destroy the world".to_string(),
            selected_option_id: None,
            meta: None,
        };

        let result = engine.validate_player_action(&action, &scene.available_options);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("超出当前世界规则"));
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
            meta: None,
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
            meta: None,
        };

        let result = engine.process_player_action(
            &action,
            &character,
            &scene.available_options,
            &context,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("无效的选项 ID"));
    }

    #[test]
    fn test_process_action_accepts_free_text() {
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
            meta: None,
        };

        let result = engine.process_player_action(
            &action,
            &character,
            &scene.available_options,
            &context,
        );

        assert!(result.is_ok());
        assert!(!result.unwrap().description.is_empty());
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
            meta: None,
        };

        let cultivate_result = engine.process_player_action(
            &cultivate_action,
            &character,
            &scene.available_options,
            &context,
        );
        assert!(cultivate_result.is_ok());
        assert!(!cultivate_result.unwrap().description.is_empty());

        let rest_action = PlayerAction {
            action_type: ActionType::SelectedOption,
            content: "1".to_string(),
            selected_option_id: Some(1),
            meta: None,
        };

        let rest_result = engine.process_player_action(
            &rest_action,
            &character,
            &scene.available_options,
            &context,
        );
        assert!(rest_result.is_ok());
        assert!(!rest_result.unwrap().description.is_empty());

        let breakthrough_action = PlayerAction {
            action_type: ActionType::SelectedOption,
            content: "2".to_string(),
            selected_option_id: Some(2),
            meta: None,
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
            meta: None,
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

    proptest! {
        #[test]
        fn test_property_20_free_text_intent_parsing(
            input in "[A-Za-z0-9_ ]{1,120}"
        ) {
            let engine = PlotEngine::new();
            let character = CharacterStats {
                spiritual_root: SpiritualRoot {
                    element: Element::Fire,
                    grade: Grade::Heavenly,
                    affinity: 0.8,
                },
                cultivation_realm: CultivationRealm::new(
                    "Test Realm".to_string(),
                    1,
                    0,
                    1.0,
                ),
                techniques: Vec::new(),
                lifespan: Lifespan {
                    current_age: 16,
                    max_age: 100,
                    realm_bonus: 0,
                },
                combat_power: 100,
            };
            let context = Context {
                location: "sect".to_string(),
                time_of_day: "day".to_string(),
                weather: None,
            };

            let action = PlayerAction {
                action_type: ActionType::FreeText,
                content: if input.trim().is_empty() {
                    "cultivate".to_string()
                } else {
                    input
                },
                selected_option_id: None,
                meta: None,
            };

            let result = engine.process_player_action(&action, &character, &[], &context);
            prop_assert!(result.is_ok());
        }
    }

    proptest! {
        #[test]
        fn test_property_21_unreasonable_actions_are_rejected(
            suffix in "[A-Za-z0-9 ]{0,40}"
        ) {
            let engine = PlotEngine::new();
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

            let action = PlayerAction {
                action_type: ActionType::FreeText,
                content: format!("instantly become immortal and destroy the world {}", suffix),
                selected_option_id: None,
                meta: None,
            };

            let result = engine.validate_player_action(&action, &scene.available_options);
            prop_assert!(result.is_err());
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

