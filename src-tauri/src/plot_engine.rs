use crate::models::CharacterStats;
use crate::llm_runtime_config::resolve_llm_config;
use crate::llm_service::{LLMRequest, LLMService, LLMServiceError};
use crate::numerical_system::{Action, ActionResult, Context, NumericalSystem};
use crate::prompt_builder::{PromptBuilder, PromptConstraints, PromptContext, PromptTemplate};
use crate::response_validator::{ResponseValidator, ValidationConstraints};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
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
    #[serde(default = "default_llm_priority_mode")]
    pub llm_priority_mode: bool,
    #[serde(default = "default_llm_strict_mode")]
    pub llm_strict_mode: bool,
    pub min_interactions_per_chapter: u8,
    pub max_interactions_per_chapter: u8,
    pub target_chapter_words_min: u32,
    pub target_chapter_words_max: u32,
}

fn default_llm_priority_mode() -> bool {
    true
}

fn default_llm_strict_mode() -> bool {
    true
}

impl Default for PlotSettings {
    fn default() -> Self {
        Self {
            recap_enabled: true,
            novel_style: "修仙白话·第三人称".to_string(),
            llm_priority_mode: true,
            llm_strict_mode: true,
            min_interactions_per_chapter: 2,
            max_interactions_per_chapter: 3,
            target_chapter_words_min: 5000,
            target_chapter_words_max: 7000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ChapterLifecycle {
    #[default]
    InProgress,
    Closed,
    Exported,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PlotInteractionState {
    AutoAdvance,
    #[default]
    WaitingForChoice,
    WaitingForFreeText,
    Resolving,
    Cooldown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChapterState {
    pub index: u32,
    pub title: String,
    pub content: Vec<String>,
    pub summary: String,
    pub interaction_count: u8,
    #[serde(default)]
    pub status: ChapterLifecycle,
}

impl ChapterState {
    pub fn new(index: u32, title: String) -> Self {
        Self {
            index,
            title,
            content: Vec::new(),
            summary: String::new(),
            interaction_count: 0,
            status: ChapterLifecycle::InProgress,
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
    #[serde(default)]
    pub interaction_state: PlotInteractionState,
    pub last_action_result: Option<ActionResult>,
    pub settings: PlotSettings,
    pub current_chapter: ChapterState,
    pub chapters: Vec<ChapterState>,
    pub segment_count: u32,
    #[serde(default)]
    pub last_generation_diagnostics: Option<String>,
    #[serde(default)]
    pub last_option_generation_source: Option<String>,
    #[serde(default)]
    pub last_consistency_risk_score: Option<i32>,
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
    pub generation_diagnostics: Option<String>,
}

pub struct PlotEngine {
    numerical_system: NumericalSystem,
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
    generation_diagnostics: Option<String>,
}

impl PlotEngine {
    pub fn new() -> Self {
        Self {
            numerical_system: NumericalSystem::new(),
            prompt_builder: PromptBuilder::default(),
            response_validator: ResponseValidator::default(),
        }
    }

    fn resolve_llm_service(&self) -> Option<LLMService> {
        let cfg = resolve_llm_config()?;
        LLMService::new(cfg).ok()
    }

    fn run_llm_request(&self, llm_service: &LLMService, request: LLMRequest) -> Option<crate::llm_service::LLMResponse> {
        if let Ok(handle) = Handle::try_current() {
            return task::block_in_place(|| {
                handle
                    .block_on(tokio::time::timeout(
                        Duration::from_secs(45),
                        llm_service.generate(request),
                    ))
                    .ok()
                    .and_then(Result::ok)
            });
        }

        let runtime = tokio::runtime::Runtime::new().ok()?;
        runtime
            .block_on(tokio::time::timeout(
                Duration::from_secs(45),
                llm_service.generate(request),
            ))
            .ok()
            .and_then(Result::ok)
    }

    fn llm_error_reason(err: &LLMServiceError) -> String {
        match err {
            LLMServiceError::Timeout => "请求超时".to_string(),
            LLMServiceError::Api(msg) => format!("API 错误({msg})"),
            LLMServiceError::Http(http) => format!("HTTP 错误({http})"),
            LLMServiceError::InvalidResponse(msg) => format!("响应解析失败({msg})"),
            LLMServiceError::InvalidRequest(msg) => format!("请求参数无效({msg})"),
            LLMServiceError::InvalidConfig(msg) => format!("配置无效({msg})"),
        }
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
                .find([',', '\n', '}'])
                .unwrap_or(s.len());
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
                let s = &after[start..];
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

    fn compose_segment_text_from_json(&self, value: &Value) -> Option<String> {
        let direct = value
            .get("segment_text")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToString::to_string);
        if direct.is_some() {
            return direct;
        }

        let mut parts = Vec::new();
        for key in [
            "scene_description",
            "environment_detail",
            "npc_reaction",
            "event_development",
            "player_action_consequence",
        ] {
            if let Some(text) = value
                .get(key)
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                parts.push(text.to_string());
            }
        }

        if parts.is_empty() {
            None
        } else {
            Some(parts.join("\n\n"))
        }
    }

    fn sanitize_llm_plain_text(&self, raw: &str) -> Option<String> {
        let mut candidate = raw.trim().to_string();
        if candidate.starts_with("```") {
            let mut lines = candidate.lines();
            let _ = lines.next();
            candidate = lines.collect::<Vec<&str>>().join("\n");
            if let Some(stripped) = candidate.strip_suffix("```") {
                candidate = stripped.trim().to_string();
            }
        }

        let cleaned = candidate.trim();
        if cleaned.is_empty() {
            return None;
        }

        // Avoid rendering raw JSON blobs into chapter body.
        if (cleaned.starts_with('{') && cleaned.ends_with('}'))
            || cleaned.contains("\"scene_description\"")
            || cleaned.contains("\"segment_text\"")
            || cleaned.contains("\"player_action_consequence\"")
        {
            return None;
        }

        Some(cleaned.to_string())
    }

    fn normalize_story_text(&self, raw: &str) -> String {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        // If model output is cut mid-sentence, trim to last complete sentence punctuation.
        let mut last_punct_idx = None;
        for (idx, ch) in trimmed.char_indices() {
            if matches!(ch, '。' | '！' | '？' | '；' | '.' | '!' | '?') {
                last_punct_idx = Some(idx + ch.len_utf8());
            }
        }
        if let Some(end) = last_punct_idx {
            let candidate = trimmed[..end].trim();
            if !candidate.is_empty() {
                return candidate.to_string();
            }
        }
        trimmed.to_string()
    }

    fn truncate_chars(&self, text: &str, max_chars: usize) -> String {
        let mut out = String::new();
        for (idx, ch) in text.chars().enumerate() {
            if idx >= max_chars {
                break;
            }
            out.push(ch);
        }
        out
    }

    fn count_fact_anchors(&self, text: &str) -> usize {
        let anchors = [
            "获得", "失去", "消耗", "到达", "离开", "位于", "灵石", "功法", "背包", "境界", "关系",
            "伤势", "地图", "地点", "目标", "资源", "线索", "时辰", "天后", "日后",
        ];
        let mut hits = 0usize;
        for anchor in anchors {
            hits += text.match_indices(anchor).count();
        }
        if text.chars().any(|c| c.is_ascii_digit()) {
            hits += 1;
        }
        hits
    }

    fn validate_story_text_contract(&self, text: &str) -> Result<(), String> {
        let chars = text.chars().count();
        if !(700..=1000).contains(&chars) {
            return Err(format!("segment_text 字数为 {}，未命中 700-1000", chars));
        }
        let required_fact_hits = (chars / 140).max(2);
        let fact_hits = self.count_fact_anchors(text);
        if fact_hits < required_fact_hits {
            return Err(format!(
                "segment_text 事实锚点不足：{} < {}",
                fact_hits, required_fact_hits
            ));
        }
        Ok(())
    }

    fn parse_chapter_segment_response(&self, raw: &str) -> Result<ChapterSegment, String> {
        if let Some(value) = self.extract_json_value(raw) {
            let text = self
                .compose_segment_text_from_json(&value)
                .unwrap_or_default();
            let text = self.normalize_story_text(&text);
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
                        .take(4)
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default();

            if text.is_empty() {
                return Err("segment_text 为空".to_string());
            }
            self.validate_story_text_contract(&text)?;
            if needs_player_input && options.len() < 2 {
                return Err("needs_player_input=true 但 options 少于 2 个".to_string());
            }
            return Ok(ChapterSegment {
                text,
                needs_player_input,
                chapter_end,
                chapter_title,
                chapter_summary,
                options,
                generation_diagnostics: None,
            });
        }

        if let Some(text) = self.extract_string_field_raw(raw, "segment_text") {
            let text = self.normalize_story_text(&text);
            let needs_player_input = self
                .extract_bool_field_raw(raw, "needs_player_input")
                .unwrap_or(true);
            let chapter_end = self.extract_bool_field_raw(raw, "chapter_end").unwrap_or(false);
            let chapter_title = self.extract_string_field_raw(raw, "chapter_title");
            let chapter_summary = self.extract_string_field_raw(raw, "chapter_summary");
            let options = self.extract_options_field_raw(raw);
            self.validate_story_text_contract(&text)?;
            if needs_player_input && options.len() < 2 {
                return Err("needs_player_input=true 但 options 少于 2 个".to_string());
            }
            return Ok(ChapterSegment {
                text,
                needs_player_input,
                chapter_end,
                chapter_title,
                chapter_summary,
                options,
                generation_diagnostics: None,
            });
        }

        if let Some(text) = self.sanitize_llm_plain_text(raw) {
            let text = self.normalize_story_text(&text);
            self.validate_story_text_contract(&text)?;
            return Ok(ChapterSegment {
                text,
                needs_player_input: true,
                chapter_end: false,
                chapter_title: None,
                chapter_summary: None,
                options: vec![],
                generation_diagnostics: None,
            });
        }

        Err("LLM 返回内容无法解析为剧情文本".to_string())
    }

    fn parse_plain_text_response(&self, raw: &str) -> Result<String, String> {
        if let Some(value) = self.extract_json_value(raw) {
            if let Some(text) = self.compose_segment_text_from_json(&value) {
                let normalized = self.normalize_story_text(&text);
                self.validate_story_text_contract(&normalized)?;
                return Ok(normalized);
            }
        }
        let text = self
            .sanitize_llm_plain_text(raw)
            .map(|text| self.normalize_story_text(&text))
            .filter(|text| !text.is_empty())
            .ok_or_else(|| "LLM 返回内容无法解析为纯文本剧情".to_string())?;
        self.validate_story_text_contract(&text)?;
        Ok(text)
    }

    fn find_matching_option_by_text<'a>(
        &self,
        free_text: &str,
        available_options: &'a [PlayerOption],
    ) -> Option<&'a PlayerOption> {
        let input = free_text.trim();
        if input.is_empty() {
            return None;
        }

        available_options.iter().find(|option| {
            let text = option.description.trim();
            text == input || text.eq_ignore_ascii_case(input)
        })
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
            generation_diagnostics: segment.generation_diagnostics,
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
            generation_diagnostics: segment.generation_diagnostics,
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
            generation_diagnostics: Some("回退：同步剧情生成未命中 LLM，已使用预设文本".to_string()),
        }
    }

    async fn generate_chapter_segment_async(
        &self,
        current_state: &PlotState,
        action_result: &ActionResult,
    ) -> ChapterSegment {
        let (segment_from_llm, llm_reason) = self
            .generate_chapter_segment_with_llm_async(current_state, action_result)
            .await;
        if let Some(segment) = segment_from_llm {
            return self.apply_chapter_segment_rules(current_state, segment);
        }

        let (plain_text, plain_reason) = self
            .generate_plot_text_with_llm_async(current_state, action_result)
            .await;
        if let Some(text) = plain_text {
            return self.apply_chapter_segment_rules(
                current_state,
                ChapterSegment {
                    text,
                    needs_player_input: true,
                    chapter_end: false,
                    chapter_title: None,
                    chapter_summary: None,
                    options: vec![],
                    generation_diagnostics: Some(match llm_reason {
                        Some(reason) => format!("回退：{}；已降级为纯文本续写", reason),
                        None => "已降级为纯文本续写".to_string(),
                    }),
                },
            );
        }

        let text = self.generate_plot_text_fallback(current_state, action_result);
        let fallback_reason = match (llm_reason, plain_reason) {
            (Some(s), Some(p)) => format!("{s}；纯文本续写失败({p})"),
            (Some(s), None) => s,
            (None, Some(p)) => format!("纯文本续写失败({p})"),
            (None, None) => "LLM 续写不可用（可能无配置或返回不可解析）".to_string(),
        };
        ChapterSegment {
            text,
            needs_player_input: true,
            chapter_end: false,
            chapter_title: None,
            chapter_summary: None,
            options: vec![],
            generation_diagnostics: Some(format!(
                "回退：{}；纯文本续写也失败，已使用预设文本",
                fallback_reason
            )),
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
                "小说风格：{}；参考设定文档 78 与补充稿，采用克制、具体、可验证的修仙叙事。请生成一段承接剧情的小说文本。玩家每章需要 2-3 次互动。",
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
            let text = self
                .compose_segment_text_from_json(&value)
                .unwrap_or_default();
            let text = self.normalize_story_text(&text);
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
                    generation_diagnostics: None,
                });
            }
        }

        if let Some(text) = self.extract_string_field_raw(&response.text, "segment_text") {
            let text = self.normalize_story_text(&text);
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
                generation_diagnostics: None,
            });
        }

        self.sanitize_llm_plain_text(&response.text).map(|text| ChapterSegment {
            text: self.normalize_story_text(&text),
            needs_player_input: true,
            chapter_end: false,
            chapter_title: None,
            chapter_summary: None,
            options: vec![],
            generation_diagnostics: None,
        })
    }

    async fn generate_chapter_segment_with_llm_async(
        &self,
        current_state: &PlotState,
        action_result: &ActionResult,
    ) -> (Option<ChapterSegment>, Option<String>) {
        if cfg!(test) {
            return (None, None);
        }
        let llm_service = match self.resolve_llm_service() {
            Some(service) => service,
            None => return (None, Some("未检测到可用 LLM 配置".to_string())),
        };
        let settings = &current_state.settings;
        let llm_priority_mode = settings.llm_priority_mode;
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
                "每次输出 700-1000 字".to_string(),
                "语言平实克制，禁止空泛口号与夸张修辞".to_string(),
                "每120字至少包含1个可验证事实（位置/物品/关系/状态变化）".to_string(),
                "必须包含环境、动作、心理三类细节中的至少两类".to_string(),
                "出现新人物/地点/功法/物品时，命名要稳定且可追踪".to_string(),
                "needs_player_input 为 true 时，必须给出 2-4 个 options".to_string(),
                "chapter_end 仅在章节接近尾声时为 true".to_string(),
            ],
            output_schema_hint: Some(
                "{\"segment_text\":\"string\",\"needs_player_input\":true|false,\"chapter_end\":true|false,\"chapter_title\":\"string\",\"chapter_summary\":\"string\",\"options\":[\"string\"]}".to_string(),
            ),
        };

        // Prefer LLM storyline quality when enabled: allow longer wait and larger output budget.
        let output_max = if llm_priority_mode {
            llm_service.api_config.max_tokens.clamp(1100, 1800)
        } else {
            llm_service.api_config.max_tokens.clamp(900, 1500)
        };
        let prompt_limit = output_max.saturating_mul(if llm_priority_mode { 10 } else { 8 });
        let primary_timeout_secs: u64 = if llm_priority_mode { 28 } else { 20 };
        let retry_timeout_secs: u64 = if llm_priority_mode { 16 } else { 12 };

        let prompt = self.prompt_builder.build_prompt_with_token_limit(
            PromptTemplate::PlotGeneration,
            &context,
            &constraints,
            prompt_limit,
        );

        let response = match tokio::time::timeout(
            Duration::from_secs(primary_timeout_secs),
            llm_service.generate(LLMRequest {
                prompt: prompt.clone(),
                max_tokens: Some(output_max),
                temperature: Some(0.7),
            }),
        )
        .await
        {
            Ok(Ok(resp)) => resp,
            Ok(Err(first_err)) => {
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
                            "每次输出 700-1000 字".to_string(),
                            "语言平实克制，禁止空泛口号与夸张修辞".to_string(),
                            "每120字至少包含1个可验证事实（位置/物品/关系/状态变化）".to_string(),
                            "必须包含环境、动作、心理三类细节中的至少两类".to_string(),
                            "needs_player_input 为 true 时，必须给出 2-4 个 options".to_string(),
                        ],
                        output_schema_hint: Some(
                            "{\"segment_text\":\"string\",\"needs_player_input\":true|false,\"chapter_end\":true|false,\"chapter_title\":\"string\",\"chapter_summary\":\"string\",\"options\":[\"string\"]}".to_string(),
                        ),
                    },
                    output_max.saturating_mul(3),
                );
                match tokio::time::timeout(
                    Duration::from_secs(retry_timeout_secs),
                    llm_service.generate(LLMRequest {
                        prompt: retry_prompt,
                        max_tokens: Some(output_max),
                        temperature: Some(0.7),
                    }),
                )
                .await
                {
                    Ok(Ok(resp)) => resp,
                    Ok(Err(retry_err)) => {
                        return (
                            None,
                            Some(format!(
                                "LLM 结构化剧情生成失败({})；重试失败({})",
                                Self::llm_error_reason(&first_err),
                                Self::llm_error_reason(&retry_err)
                            )),
                        );
                    }
                    Err(_) => {
                        return (
                            None,
                            Some(format!(
                                "LLM 结构化剧情生成失败({})；重试失败(请求超时)",
                                Self::llm_error_reason(&first_err)
                            )),
                        );
                    }
                }
            }
            Err(_) => return (None, Some("LLM 结构化剧情生成超时".to_string())),
        };

        if self
            .response_validator
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
            .is_err()
        {
            return (None, Some("LLM 返回内容校验失败".to_string()));
        }

        if let Ok(segment) = self.parse_chapter_segment_response(&response.text) {
            return (Some(segment), None);
        }
        let parse_err = match self.parse_chapter_segment_response(&response.text) {
            Ok(segment) => return (Some(segment), None),
            Err(err) => err,
        };

        let retry_context = PromptContext {
            scene: Some(format!(
                "{}。上次输出校验失败：{}。上次输出片段：{}",
                context.scene.clone().unwrap_or_default(),
                parse_err,
                self.truncate_chars(&response.text, 320)
            )),
            ..context.clone()
        };
        let mut retry_constraints = constraints.clone();
        retry_constraints
            .world_rules
            .push("修复上次错误并返回完整 JSON，不要解释".to_string());
        retry_constraints
            .world_rules
            .push("segment_text 必须 700-1000 字，且保持事实密度".to_string());
        let repair_prompt = self.prompt_builder.build_prompt_with_token_limit(
            PromptTemplate::PlotGeneration,
            &retry_context,
            &retry_constraints,
            output_max.saturating_mul(3),
        );

        let repair_response = match tokio::time::timeout(
            Duration::from_secs(retry_timeout_secs),
            llm_service.generate(LLMRequest {
                prompt: repair_prompt,
                max_tokens: Some(output_max),
                temperature: Some(0.65),
            }),
        )
        .await
        {
            Ok(Ok(resp)) => resp,
            Ok(Err(err)) => {
                return (
                    None,
                    Some(format!(
                        "LLM 结构化剧情校验失败({})；修复请求失败({})",
                        parse_err,
                        Self::llm_error_reason(&err)
                    )),
                )
            }
            Err(_) => {
                return (
                    None,
                    Some(format!(
                        "LLM 结构化剧情校验失败({})；修复请求超时",
                        parse_err
                    )),
                )
            }
        };
        match self.parse_chapter_segment_response(&repair_response.text) {
            Ok(segment) => (Some(segment), None),
            Err(repair_parse_err) => (
                None,
                Some(format!(
                    "LLM 结构化剧情校验失败({})；修复输出仍不合格({})",
                    parse_err, repair_parse_err
                )),
            ),
        }
    }

    async fn generate_plot_text_with_llm_async(
        &self,
        current_state: &PlotState,
        action_result: &ActionResult,
    ) -> (Option<String>, Option<String>) {
        if cfg!(test) {
            return (None, None);
        }
        let llm_service = match self.resolve_llm_service() {
            Some(service) => service,
            None => return (None, Some("未检测到可用 LLM 配置".to_string())),
        };
        let llm_priority_mode = current_state.settings.llm_priority_mode;
        let output_max = if llm_priority_mode {
            llm_service.api_config.max_tokens.clamp(1000, 1600)
        } else {
            llm_service.api_config.max_tokens.clamp(850, 1300)
        };
        let primary_timeout_secs: u64 = if llm_priority_mode { 22 } else { 16 };
        let retry_timeout_secs: u64 = if llm_priority_mode { 14 } else { 10 };
        let prompt = self.prompt_builder.build_prompt_with_token_limit(
            PromptTemplate::PlotGeneration,
            &PromptContext {
                scene: Some(format!(
                    "承接上一段剧情，并自然写入玩家刚刚选择：{}。上一段内容：{}",
                    action_result.description,
                    current_state.current_scene.description
                )),
                location: Some(current_state.current_scene.location.clone()),
                actor_name: Some("player".to_string()),
                actor_realm: None,
                actor_combat_power: None,
                history_events: action_result.events.clone(),
                world_setting_summary: Some("参考设定文档 78 与补充稿，修仙叙事保持克制、写实、可验证，强调场景、事件与 NPC 反应".to_string()),
            },
            &PromptConstraints {
                numerical_rules: vec!["必须与行动结果保持一致".to_string()],
                world_rules: vec![
                    "仅输出纯文本".to_string(),
                    "使用简洁的小说叙事".to_string(),
                    "必须使用中文".to_string(),
                    "控制在 700-1000 字".to_string(),
                    "语言平实克制，禁止空泛口号与夸张修辞".to_string(),
                    "每120字至少包含1个可验证事实（位置/物品/关系/状态变化）".to_string(),
                    "必须包含环境、动作、心理三类细节中的至少两类".to_string(),
                ],
                output_schema_hint: Some(
                    "仅返回正文纯文本，不要 JSON、不要 Markdown 代码块。".to_string(),
                ),
            },
            output_max.saturating_mul(if llm_priority_mode { 5 } else { 4 }),
        );

        let response = match tokio::time::timeout(
            Duration::from_secs(primary_timeout_secs),
            llm_service.generate(LLMRequest {
                prompt: prompt.clone(),
                max_tokens: Some(output_max),
                temperature: Some(0.7),
            }),
        )
        .await
        {
            Ok(Ok(resp)) => resp,
            Ok(Err(err)) => {
                let retry_prompt = self.prompt_builder.build_prompt_with_token_limit(
                    PromptTemplate::PlotGeneration,
                    &PromptContext {
                        scene: Some(format!(
                            "自然续写并推进剧情。玩家刚刚行动：{}",
                            action_result.description
                        )),
                        location: Some(current_state.current_scene.location.clone()),
                        actor_name: Some("player".to_string()),
                        actor_realm: None,
                        actor_combat_power: None,
                        history_events: action_result.events.clone(),
                        world_setting_summary: Some("参考设定文档 78 与补充稿，修仙叙事保持克制写实，输出连续叙事文本".to_string()),
                    },
                    &PromptConstraints {
                        numerical_rules: vec!["必须与行动结果保持一致".to_string()],
                        world_rules: vec![
                            "仅输出纯文本".to_string(),
                            "必须使用中文".to_string(),
                            "控制在 700-1000 字".to_string(),
                            "语言平实克制，禁止空泛口号与夸张修辞".to_string(),
                            "每120字至少包含1个可验证事实（位置/物品/关系/状态变化）".to_string(),
                            "必须包含环境、动作、心理三类细节中的至少两类".to_string(),
                        ],
                        output_schema_hint: Some(
                            "仅返回正文纯文本，不要 JSON、不要 Markdown 代码块。".to_string(),
                        ),
                    },
                    output_max.saturating_mul(2),
                );
                match tokio::time::timeout(
                    Duration::from_secs(retry_timeout_secs),
                    llm_service.generate(LLMRequest {
                        prompt: retry_prompt,
                        max_tokens: Some(output_max),
                        temperature: Some(0.65),
                    }),
                )
                .await
                {
                    Ok(Ok(resp)) => resp,
                    Ok(Err(retry_err)) => {
                        return (
                            None,
                            Some(format!(
                                "{}；重试失败({})",
                                Self::llm_error_reason(&err),
                                Self::llm_error_reason(&retry_err)
                            )),
                        );
                    }
                    Err(_) => {
                        return (
                            None,
                            Some(format!("{}；重试失败(请求超时)", Self::llm_error_reason(&err))),
                        );
                    }
                }
            }
            Err(_) => return (None, Some("纯文本续写请求超时".to_string())),
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
            .ok();
        if let Ok(text) = self.parse_plain_text_response(&response.text) {
            return (Some(text), None);
        }
        let parse_err = match self.parse_plain_text_response(&response.text) {
            Ok(text) => return (Some(text), None),
            Err(err) => err,
        };
        let repair_prompt = self.prompt_builder.build_prompt_with_token_limit(
            PromptTemplate::PlotGeneration,
            &PromptContext {
                scene: Some(format!(
                    "修复上次输出并自然续写。玩家刚刚行动：{}。上次错误：{}。上次输出片段：{}",
                    action_result.description,
                    parse_err,
                    self.truncate_chars(&response.text, 320)
                )),
                location: Some(current_state.current_scene.location.clone()),
                actor_name: Some("player".to_string()),
                actor_realm: None,
                actor_combat_power: None,
                history_events: action_result.events.clone(),
                world_setting_summary: Some(
                    "参考设定文档 78 与补充稿，修仙叙事保持克制写实，输出连续叙事文本"
                        .to_string(),
                ),
            },
            &PromptConstraints {
                numerical_rules: vec!["必须与行动结果保持一致".to_string()],
                world_rules: vec![
                    "仅输出纯文本".to_string(),
                    "必须使用中文".to_string(),
                    "控制在 700-1000 字".to_string(),
                    "语言平实克制，禁止空泛口号与夸张修辞".to_string(),
                    "每120字至少包含1个可验证事实（位置/物品/关系/状态变化）".to_string(),
                    "必须包含环境、动作、心理三类细节中的至少两类".to_string(),
                    "修复上次错误，不要解释错误原因".to_string(),
                ],
                output_schema_hint: Some(
                    "仅返回正文纯文本，不要 JSON、不要 Markdown 代码块。".to_string(),
                ),
            },
            output_max.saturating_mul(2),
        );
        let repair_response = match tokio::time::timeout(
            Duration::from_secs(retry_timeout_secs),
            llm_service.generate(LLMRequest {
                prompt: repair_prompt,
                max_tokens: Some(output_max),
                temperature: Some(0.62),
            }),
        )
        .await
        {
            Ok(Ok(resp)) => resp,
            Ok(Err(err)) => {
                return (
                    None,
                    Some(format!(
                        "纯文本续写校验失败({})；修复请求失败({})",
                        parse_err,
                        Self::llm_error_reason(&err)
                    )),
                );
            }
            Err(_) => {
                return (
                    None,
                    Some(format!("纯文本续写校验失败({})；修复请求超时", parse_err)),
                );
            }
        };
        match self.parse_plain_text_response(&repair_response.text) {
            Ok(text) => (Some(text), None),
            Err(repair_parse_err) => (
                None,
                Some(format!(
                    "纯文本续写校验失败({})；修复输出仍不合格({})",
                    parse_err, repair_parse_err
                )),
            ),
        }
    }

    fn generate_plot_text_fallback(&self, current_state: &PlotState, action_result: &ActionResult) -> String {
        let action_desc = action_result.description.trim();
        let event_line = if action_result.events.is_empty() {
            String::new()
        } else {
            format!("局势回响随之显现：{}。", action_result.events.join("；"))
        };

        let beats = [
            "气机顺着经脉回落，你迅速复盘眼前局势。",
            "四周细节在你感知里逐一清晰，新的牵连浮出水面。",
            "你压住余波，判断下一步的代价与收益。",
            "这一步落子后，场中人事都出现了细微偏转。",
        ];
        let sensory = [
            "风里夹着细碎的灵压波动，远近动静都变得格外分明。",
            "地脉微颤从脚下传来，你能感觉到周围局势正在重新排布。",
            "暗处的目光与明处的喧声交织，让这片区域不再平静。",
            "你的呼吸与灵力节律逐渐一致，判断也比先前更冷静。",
        ];
        let beat = beats[(current_state.segment_count as usize) % beats.len()];
        let sense = sensory[(current_state.segment_count as usize) % sensory.len()];

        format!(
            "{}，你{}。{}{}{}",
            current_state.current_scene.location,
            if action_desc.is_empty() { "暂缓动作，转而观察变化" } else { action_desc },
            beat,
            sense,
            if event_line.is_empty() {
                String::new()
            } else {
                format!(" {}", event_line)
            }
        )
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
        let output_max = llm_service.api_config.max_tokens.clamp(900, 1500);
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
                    "长度控制在 700 到 1000 字".to_string(),
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
                            "长度控制在 700 到 1000 字".to_string(),
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
                        max_tokens: Some(output_max),
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
        let element_label = |element: &crate::models::Element| -> &'static str {
            match element {
                crate::models::Element::Metal => "金",
                crate::models::Element::Wood => "木",
                crate::models::Element::Water => "水",
                crate::models::Element::Fire => "火",
                crate::models::Element::Earth => "土",
                crate::models::Element::Thunder => "雷",
                crate::models::Element::Wind => "风",
                crate::models::Element::Ice => "冰",
            }
        };

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

        for element in character.spiritual_root.effective_elements() {
            let label = element_label(&element);
            options.push(PlayerOption {
                id: option_id,
                description: format!("参悟{}系功法", label),
                requirements: vec![format!("灵根属性：{}系", label)],
                action: Action::Custom {
                    description: format!("你运转{}系灵力，尝试推演更契合自身灵根的功法。", label),
                },
            });
            option_id += 1;
        }

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

        // Ensure minimum 2 options.
        if options.len() < 2 {
            options.push(PlayerOption {
                id: option_id,
                description: "盘坐冥想，梳理思绪".to_string(),
                requirements: vec![],
                action: Action::Custom {
                    description: "你静心冥想，回顾当前修行方向。".to_string(),
                },
            });
        }

        options
    }

    pub fn generate_player_options_with_llm(
        &self,
        scene: &Scene,
        character: &CharacterStats,
    ) -> Option<Vec<PlayerOption>> {
        if cfg!(test) {
            return None;
        }
        let llm_service = self.resolve_llm_service()?;
        let prompt = self.prompt_builder.build_prompt_with_token_limit(
            PromptTemplate::OptionGeneration,
            &PromptContext {
                scene: Some(scene.description.clone()),
                location: Some(scene.location.clone()),
                actor_name: Some("player".to_string()),
                actor_realm: Some(character.cultivation_realm.name.clone()),
                actor_combat_power: Some(character.combat_power),
                history_events: Vec::new(),
                world_setting_summary: Some("基于当前剧情生成玩家可执行选项".to_string()),
            },
            &PromptConstraints {
                numerical_rules: vec![
                    "选项数量 2-4 条".to_string(),
                    "选项必须可执行，避免空泛描述".to_string(),
                ],
                world_rules: vec![
                    "优先输出严格 JSON".to_string(),
                    "字段为 options 或 action_choices".to_string(),
                    "每条选项不超过 24 字".to_string(),
                    "只输出中文".to_string(),
                ],
                output_schema_hint: Some(
                    "{\"options\":[\"string\",\"string\"]}".to_string(),
                ),
            },
            280,
        );

        let response = self.run_llm_request(
            &llm_service,
            LLMRequest {
                prompt,
                max_tokens: Some(220),
                temperature: Some(0.6),
            },
        )?;

        let mut texts = if let Some(value) = self.extract_json_value(&response.text) {
            value
                .get("options")
                .or_else(|| value.get("action_choices"))
                .and_then(Value::as_array)
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
                        .filter(|s| !s.is_empty())
                        .take(4)
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default()
        } else {
            self.extract_options_field_raw(&response.text)
        };

        if texts.len() < 2 {
            return None;
        }

        texts.truncate(4);
        let options = texts
            .into_iter()
            .enumerate()
            .map(|(idx, text)| PlayerOption {
                id: idx,
                description: text.clone(),
                requirements: vec![],
                action: Action::Custom { description: text },
            })
            .collect::<Vec<PlayerOption>>();
        Some(options)
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
                if self
                    .find_matching_option_by_text(&action.content, available_options)
                    .is_some()
                {
                    return Ok(());
                }
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
                } else if let Some(option) =
                    self.find_matching_option_by_text(&action.content, available_options)
                {
                    option.action.clone()
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
        let interaction_state = if initial_scene.available_options.is_empty() {
            PlotInteractionState::WaitingForFreeText
        } else {
            PlotInteractionState::WaitingForChoice
        };
        Self {
            current_scene: initial_scene,
            plot_history: Vec::new(),
            is_waiting_for_input: true,
            interaction_state,
            last_action_result: None,
            settings: PlotSettings::default(),
            current_chapter: chapter,
            chapters: Vec::new(),
            segment_count: 0,
            last_generation_diagnostics: None,
            last_option_generation_source: None,
            last_consistency_risk_score: None,
        }
    }

    pub fn recalculate_interaction_state(&mut self) {
        self.interaction_state = if !self.is_waiting_for_input {
            PlotInteractionState::AutoAdvance
        } else if self.current_scene.available_options.is_empty() {
            PlotInteractionState::WaitingForFreeText
        } else {
            PlotInteractionState::WaitingForChoice
        };
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

        self.current_chapter.status = ChapterLifecycle::Closed;
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
            elements: Vec::new(),
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
    fn test_plot_state_recalculate_auto_advance() {
        let scene = create_test_scene();
        let mut state = PlotState::new(scene);
        state.is_waiting_for_input = false;
        state.recalculate_interaction_state();
        assert_eq!(state.interaction_state, PlotInteractionState::AutoAdvance);
    }

    #[test]
    fn test_plot_state_recalculate_waiting_for_choice() {
        let scene = create_test_scene();
        let mut state = PlotState::new(scene);
        state.is_waiting_for_input = true;
        state.current_scene.available_options = vec![PlayerOption {
            id: 0,
            description: "继续探索".to_string(),
            requirements: vec![],
            action: Action::Rest,
        }];
        state.recalculate_interaction_state();
        assert_eq!(state.interaction_state, PlotInteractionState::WaitingForChoice);
    }

    #[test]
    fn test_plot_state_recalculate_waiting_for_free_text() {
        let scene = create_test_scene();
        let mut state = PlotState::new(scene);
        state.is_waiting_for_input = true;
        state.current_scene.available_options.clear();
        state.recalculate_interaction_state();
        assert_eq!(state.interaction_state, PlotInteractionState::WaitingForFreeText);
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
        assert!(options.len() >= 2 && options.len() <= 5);
        assert!(options.iter().any(|o| matches!(o.action, Action::Cultivate)));
        assert!(options.iter().any(|o| matches!(o.action, Action::Rest)));
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
                elements: Vec::new(),
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
        #![proptest_config(ProptestConfig::with_cases(100))]

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
        #![proptest_config(ProptestConfig::with_cases(100))]

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
        #![proptest_config(ProptestConfig::with_cases(100))]

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
        #![proptest_config(ProptestConfig::with_cases(100))]

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
                elements: Vec::new(),
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
        #![proptest_config(ProptestConfig::with_cases(100))]

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

