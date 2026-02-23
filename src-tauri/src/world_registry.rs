use crate::game_state::{GameState, Item, ItemType};
use crate::llm_bootstrap::{
    build_bootstrap_prompt, build_bootstrap_repair_prompt, validate_bootstrap_payload,
};
use crate::llm_runtime_config::resolve_llm_config;
use crate::llm_service::{LLMRequest, LLMService};
use crate::plot_engine::PlotState;
use crate::state_patch_validator::{default_key_field, validate_patch_row};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::time::Duration;
use std::collections::BTreeSet;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorldRegistry {
    pub session_id: String,
    pub seed: u64,
    pub created_at: u64,
    pub llm_model: Option<String>,
    pub source: String,
    pub tables: RegistryTables,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RegistryTables {
    pub characters: Vec<Value>,
    pub map_nodes: Vec<Value>,
    pub map_edges: Vec<Value>,
    pub techniques: Vec<Value>,
    pub inventory_items: Vec<Value>,
    pub factions: Vec<Value>,
    pub story_state: Vec<Value>,
    pub world_facts: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TurnUpdateResult {
    pub narrative_segment: String,
    pub choices: Vec<String>,
    pub state_patch: Value,
}

const BOOTSTRAP_LLM_TIMEOUT_SECS: u64 = 3000;
const TURN_UPDATE_LLM_TIMEOUT_SECS: u64 = 3000;
const BOOTSTRAP_LLM_MAX_ATTEMPTS: u8 = 2;
const TURN_UPDATE_LLM_MAX_ATTEMPTS: u8 = 2;

impl WorldRegistry {
    pub async fn bootstrap_with_llm(
        game_state: &GameState,
        reference_78: &str,
        supplement: &str,
    ) -> Option<Self> {
        let cfg = resolve_llm_config()?;
        let service = LLMService::new(cfg.clone()).ok()?;
        let mut prompt = build_bootstrap_prompt(game_state, reference_78, supplement);
        let mut last_error: Option<String> = None;

        for _attempt in 0..BOOTSTRAP_LLM_MAX_ATTEMPTS {
            let response = tokio::time::timeout(
                Duration::from_secs(BOOTSTRAP_LLM_TIMEOUT_SECS),
                service.generate(LLMRequest {
                    prompt: prompt.clone(),
                    max_tokens: Some(2200),
                    temperature: Some(0.35),
                }),
            )
            .await;
            let response = match response {
                Ok(v) => v,
                Err(_) => {
                    last_error = Some("bootstrap request timeout".to_string());
                    continue;
                }
            };
            let response = match response {
                Ok(v) => v,
                Err(err) => {
                    last_error = Some(format!("bootstrap request failed: {}", err));
                    continue;
                }
            };
            let parsed = match extract_json_value(&response.text) {
                Some(v) => v,
                None => {
                    last_error = Some("bootstrap output is not valid JSON object".to_string());
                    prompt = build_bootstrap_repair_prompt(
                        game_state,
                        reference_78,
                        supplement,
                        last_error.as_deref().unwrap_or("unknown parse error"),
                        &response.text,
                    );
                    continue;
                }
            };
            if let Err(err) = validate_bootstrap_payload(&parsed, game_state) {
                last_error = Some(err.clone());
                prompt = build_bootstrap_repair_prompt(
                    game_state,
                    reference_78,
                    supplement,
                    &err,
                    &response.text,
                );
                continue;
            }

            let mut registry =
                from_bootstrap_payload(game_state, parsed, "llm_bootstrap".to_string());
            registry.llm_model = Some(cfg.model);
            return Some(registry);
        }

        let _ = last_error;
        None
    }

    pub fn fallback_from_game_state(game_state: &GameState, source: &str) -> Self {
        let seed = current_unix_ts();
        let player = &game_state.player;
        let mut tables = RegistryTables::default();
        tables.characters.push(json!({
            "character_id": player.id,
            "name": player.name,
            "role": "player",
            "realm_stage": player.stats.cultivation_realm.name,
            "realm_substage": player.stats.cultivation_realm.sub_level,
            "location_id": player.location,
            "combat_power": player.stats.combat_power
        }));
        for loc in &game_state.script.world_setting.locations {
            tables.map_nodes.push(json!({
                "location_id": loc.id,
                "name": loc.name,
                "description": loc.description,
                "spiritual_density": loc.spiritual_energy
            }));
        }
        for fac in &game_state.script.world_setting.factions {
            tables.factions.push(json!({
                "faction_id": fac.id,
                "name": fac.name,
                "description": fac.description,
                "power_rank": fac.power_level
            }));
        }
        for tech in &game_state.script.world_setting.techniques {
            tables.techniques.push(json!({
                "technique_id": tech.id,
                "name": tech.name,
                "description": tech.description,
                "required_realm_level": tech.required_realm_level,
                "owner_character_id": "player"
            }));
        }
        tables.story_state.push(json!({
            "chapter_index": 1,
            "chapter_goal": "Establish concrete conflict and short-term objective",
            "current_arc": "opening",
            "pending_conflicts": ["resource_shortage", "weak_foundation"]
        }));
        tables.world_facts.push(json!({
            "fact_id": format!("fact-player-at-{}", seed),
            "subject": "player",
            "predicate": "at",
            "object": player.location,
            "confidence": 0.9
        }));

        Self {
            session_id: build_session_id(&game_state.script.id),
            seed,
            created_at: current_unix_ts(),
            llm_model: None,
            source: source.to_string(),
            tables: normalize_tables(tables, game_state),
        }
    }

    pub async fn generate_turn_update_with_llm(
        &self,
        game_state: &GameState,
        plot_state: &PlotState,
        player_action: &str,
        reference_78: &str,
        supplement: &str,
    ) -> Option<TurnUpdateResult> {
        let (result, _) = self
            .generate_turn_update_with_llm_diagnostic(
                game_state,
                plot_state,
                player_action,
                reference_78,
                supplement,
            )
            .await;
        result
    }

    pub async fn generate_turn_update_with_llm_diagnostic(
        &self,
        game_state: &GameState,
        plot_state: &PlotState,
        player_action: &str,
        reference_78: &str,
        supplement: &str,
    ) -> (Option<TurnUpdateResult>, Option<String>) {
        let cfg = match resolve_llm_config() {
            Some(v) => v,
            None => return (None, Some("turn update missing llm config".to_string())),
        };
        let service = match LLMService::new(cfg) {
            Ok(v) => v,
            Err(err) => return (None, Some(format!("turn update llm service init failed: {}", err))),
        };
        let mut prompt = build_turn_prompt(
            self,
            game_state,
            plot_state,
            player_action,
            reference_78,
            supplement,
        );
        let mut last_error: Option<String> = None;

        for _attempt in 0..TURN_UPDATE_LLM_MAX_ATTEMPTS {
            let response = tokio::time::timeout(
                Duration::from_secs(TURN_UPDATE_LLM_TIMEOUT_SECS),
                service.generate(LLMRequest {
                    prompt: prompt.clone(),
                    max_tokens: Some(2200),
                    temperature: Some(0.4),
                }),
            )
            .await;
            let response = match response {
                Ok(v) => v,
                Err(_) => {
                    last_error = Some("turn update request timeout".to_string());
                    continue;
                }
            };
            let response = match response {
                Ok(v) => v,
                Err(err) => {
                    last_error = Some(format!("turn update request failed: {}", err));
                    continue;
                }
            };
            let parsed = match extract_json_value(&response.text) {
                Some(v) => v,
                None => {
                    last_error = Some("turn update output is not valid JSON object".to_string());
                    prompt = build_turn_repair_prompt(
                        self,
                        game_state,
                        plot_state,
                        player_action,
                        reference_78,
                        supplement,
                        last_error.as_deref().unwrap_or("unknown parse error"),
                        &response.text,
                    );
                    continue;
                }
            };

            match parse_turn_update_payload(&parsed) {
                Ok(result) => return (Some(result), None),
                Err(err) => {
                    last_error = Some(err.clone());
                    prompt = build_turn_repair_prompt(
                        self,
                        game_state,
                        plot_state,
                        player_action,
                        reference_78,
                        supplement,
                        &err,
                        &response.text,
                    );
                }
            }
        }

        (
            None,
            Some(last_error.unwrap_or_else(|| "turn update failed for unknown reason".to_string())),
        )
    }

    pub fn apply_state_patch_transactional(&mut self, patch: &Value) -> Result<Vec<String>, String> {
        let Some(obj) = patch.as_object() else {
            return Err("state_patch must be a JSON object".to_string());
        };
        if obj.is_empty() {
            return Ok(Vec::new());
        }

        let mut working = self.tables.clone();
        let mut notes = Vec::new();
        for (table, operations) in obj {
            let Some(target) = table_mut(&mut working, table) else {
                return Err(format!("unknown table in state_patch: {}", table));
            };
            let Some(ops) = operations.as_array() else {
                return Err(format!("table {} patch must be an array", table));
            };

            for op in ops {
                let action = op
                    .get("__op")
                    .and_then(Value::as_str)
                    .unwrap_or("upsert");
                match action {
                    "upsert_by_key" => {
                        let key_field = op
                            .get("__key_field")
                            .and_then(Value::as_str)
                            .filter(|s| !s.trim().is_empty())
                            .unwrap_or_else(|| default_key_field(table));
                        let row = op.get("row").unwrap_or(op);
                        let Some(row_obj) = row.as_object() else {
                            return Err(format!("table {} upsert_by_key row must be object", table));
                        };
                        validate_patch_row(table, row_obj)?;
                        let key_value = row_obj
                            .get(key_field)
                            .and_then(Value::as_str)
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                            .ok_or_else(|| {
                                format!(
                                    "table {} upsert_by_key requires non-empty key {}",
                                    table, key_field
                                )
                            })?;
                        if let Some((idx, _)) = target.iter().enumerate().find(|(_, existing)| {
                            existing
                                .get(key_field)
                                .and_then(Value::as_str)
                                .map(str::trim)
                                .map(|v| v == key_value)
                                .unwrap_or(false)
                        }) {
                            target[idx] = Value::Object(row_obj.clone());
                            notes.push(format!(
                                "patch:upsert_by_key {} {}={}",
                                table, key_field, key_value
                            ));
                        } else {
                            target.push(Value::Object(row_obj.clone()));
                            notes.push(format!(
                                "patch:insert_by_key {} {}={}",
                                table, key_field, key_value
                            ));
                        }
                    }
                    "delete" => {
                        let idx = op
                            .get("__index")
                            .and_then(Value::as_u64)
                            .ok_or_else(|| format!("table {} delete requires __index", table))?
                            as usize;
                        if idx >= target.len() {
                            return Err(format!(
                                "table {} delete index {} out of bounds({})",
                                table,
                                idx,
                                target.len()
                            ));
                        }
                        target.remove(idx);
                        notes.push(format!("patch:delete {} @{}", table, idx));
                    }
                    "replace" => {
                        let idx = op
                            .get("__index")
                            .and_then(Value::as_u64)
                            .ok_or_else(|| format!("table {} replace requires __index", table))?
                            as usize;
                        if idx >= target.len() {
                            return Err(format!(
                                "table {} replace index {} out of bounds({})",
                                table,
                                idx,
                                target.len()
                            ));
                        }
                        let row = op.get("row").unwrap_or(op);
                        let Some(row_obj) = row.as_object() else {
                            return Err(format!("table {} replace row must be object", table));
                        };
                        validate_patch_row(table, row_obj)?;
                        target[idx] = Value::Object(row_obj.clone());
                        notes.push(format!("patch:replace {} @{}", table, idx));
                    }
                    _ => {
                        let row = op.get("row").unwrap_or(op);
                        let Some(row_obj) = row.as_object() else {
                            return Err(format!("table {} row must be object", table));
                        };
                        validate_patch_row(table, row_obj)?;
                        target.push(Value::Object(row_obj.clone()));
                        notes.push(format!("patch:upsert {} +1", table));
                    }
                }
            }
        }

        self.tables = working;
        Ok(notes)
    }

    pub fn validate_narrative_entity_references(&self, narrative: &str) -> Result<(), Vec<String>> {
        let mut candidates = extract_marked_entities(narrative);
        candidates.extend(extract_unmarked_named_entities(narrative));
        candidates.sort();
        candidates.dedup();
        if candidates.is_empty() {
            return Ok(());
        }
        let known = collect_known_names(&self.tables);
        let unknown = candidates
            .into_iter()
            .filter(|name| !known.contains(name))
            .collect::<Vec<_>>();
        if unknown.is_empty() {
            Ok(())
        } else {
            Err(unknown)
        }
    }

    pub fn validate_turn_narrative_contract(&self, narrative: &str) -> Result<(), String> {
        let chars = narrative.chars().count();
        if !(500..=1200).contains(&chars) {
            return Err(format!("narrative_segment length out of range: {} (expected 500-1200)", chars));
        }

        let sentences = narrative
            .split(['。', '！', '？', '\n'])
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        let factual_hits = sentences
            .iter()
            .filter(|s| sentence_has_fact_anchor(s))
            .count();
        let required_hits = (chars / 180).max(1);
        if factual_hits < required_hits {
            return Err(format!(
                "narrative factual density too low: {} < {} (per 120 chars)",
                factual_hits, required_hits
            ));
        }

        let adjective_ratio = estimate_adjective_ratio(narrative);
        if adjective_ratio > 0.15 {
            return Err(format!(
                "narrative adjective ratio too high: {:.2} > 0.15",
                adjective_ratio
            ));
        }
        Ok(())
    }
}

fn build_session_id(prefix: &str) -> String {
    format!("{}-{}", prefix, current_unix_ts())
}

fn current_unix_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn build_turn_prompt(
    registry: &WorldRegistry,
    game_state: &GameState,
    plot_state: &PlotState,
    player_action: &str,
    reference_78: &str,
    supplement: &str,
) -> String {
    let story_state_json = serde_json::to_string(&registry.tables.story_state).unwrap_or_default();
    let facts_json = serde_json::to_string(&registry.tables.world_facts).unwrap_or_default();
    format!(
        "You are a state-first xianxia narrative engine.\n\
Follow docs 78 + supplement. Style must be plain and concrete. No bombastic prose.\n\
[ref_78]\n{}\n\
[supplement]\n{}\n\
[player]\nname={} realm={} location={} age={}\n\
[action]\n{}\n\
[current_chapter]\n{}\n\
[current_scene]\n{}\n\
[story_state]\n{}\n\
[world_facts]\n{}\n\
Return strict JSON only:\n\
{{\
  \"state_patch\": {{\
    \"characters\": [],\
    \"map_nodes\": [],\
    \"map_edges\": [],\
    \"techniques\": [],\
    \"inventory_items\": [],\
    \"factions\": [],\
    \"story_state\": [],\
    \"world_facts\": []\
  }},\
  \"narrative_segment\": \"500-1200 chars Chinese narrative\",\
  \"choices\": [\"...\",\"...\",\"...\"]\
}}\n\
Extra constraints:\n\
- narrative_segment must be 500-1200 Chinese characters.\n\
- narrative_segment must use second-person pronoun \"你\" consistently for protagonist.\n\
- Write concrete actions, resources, locations, relations, injuries or time cost. Avoid empty rhetoric.\n\
- Any newly introduced proper noun must be marked as 《name》 in narrative_segment.\n\
- Every marked entity 《name》 must exist in state_patch or pre-existing tables.\n",
        clip(reference_78, 1400),
        clip(supplement, 1200),
        game_state.player.name,
        game_state.player.stats.cultivation_realm.name,
        game_state.player.location,
        game_state.player.stats.lifespan.current_age,
        player_action,
        plot_state.current_chapter.title,
        clip(&plot_state.current_scene.description, 600),
        clip(&story_state_json, 800),
        clip(&facts_json, 800),
    )
}

fn build_turn_repair_prompt(
    registry: &WorldRegistry,
    game_state: &GameState,
    plot_state: &PlotState,
    player_action: &str,
    reference_78: &str,
    supplement: &str,
    error: &str,
    previous_output: &str,
) -> String {
    format!(
        "{}\n\
[RETRY]\n\
Your previous turn JSON failed validation.\n\
Validation error: {}\n\
Previous output snippet:\n{}\n\
Return a full corrected JSON object only. Keep narrative_segment in 500-1200 chars.",
        build_turn_prompt(
            registry,
            game_state,
            plot_state,
            player_action,
            reference_78,
            supplement
        ),
        error,
        clip(previous_output, 1200),
    )
}

fn parse_turn_update_payload(value: &Value) -> Result<TurnUpdateResult, String> {
    let Some(obj) = value.as_object() else {
        return Err("turn update payload must be object".to_string());
    };

    let narrative = obj
        .get("narrative_segment")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string();
    if narrative.is_empty() {
        return Err("turn update narrative_segment is empty".to_string());
    }
    let chars = narrative.chars().count();
    if !(500..=1200).contains(&chars) {
        return Err(format!(
            "turn update narrative_segment length out of range: {} (expected 500-1200)",
            chars
        ));
    }

    let choices = obj
        .get("choices")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(ToString::to_string))
        .filter(|s| !s.trim().is_empty())
        .take(4)
        .collect::<Vec<_>>();
    if choices.is_empty() {
        return Err("turn update choices cannot be empty".to_string());
    }

    let state_patch = obj.get("state_patch").cloned().unwrap_or_else(|| json!({}));
    if !state_patch.is_object() {
        return Err("turn update state_patch must be object".to_string());
    }

    Ok(TurnUpdateResult {
        narrative_segment: narrative,
        choices,
        state_patch,
    })
}

fn collect_known_names(tables: &RegistryTables) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    collect_name_field(&tables.characters, "name", &mut out);
    collect_name_field(&tables.map_nodes, "name", &mut out);
    collect_name_field(&tables.techniques, "name", &mut out);
    collect_name_field(&tables.inventory_items, "name", &mut out);
    collect_name_field(&tables.factions, "name", &mut out);
    out
}

fn collect_name_field(rows: &[Value], key: &str, out: &mut BTreeSet<String>) {
    for row in rows {
        let Some(name) = row.get(key).and_then(Value::as_str) else {
            continue;
        };
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            out.insert(trimmed.to_string());
        }
    }
}

fn extract_marked_entities(text: &str) -> Vec<String> {
    let mut out = BTreeSet::new();
    extract_between(text, '《', '》', &mut out);
    extract_between(text, '「', '」', &mut out);
    extract_between(text, '【', '】', &mut out);
    out.into_iter().collect::<Vec<_>>()
}

fn sentence_has_fact_anchor(sentence: &str) -> bool {
    let anchors = [
        "获得", "失去", "消耗", "到达", "离开", "位于", "灵石", "功法", "背包", "境界", "关系",
        "伤势", "地图", "地点", "目标", "资源", "线索",
    ];
    if sentence.chars().any(|c| c.is_ascii_digit()) {
        return true;
    }
    anchors.iter().any(|a| sentence.contains(a))
}

fn estimate_adjective_ratio(text: &str) -> f32 {
    let adjectives = [
        "宏大", "浩瀚", "无尽", "滔天", "璀璨", "古老", "神秘", "强大", "惊人", "绝世",
        "巍峨", "磅礴", "辉煌", "冷冽", "炽烈", "耀眼", "深邃", "玄妙", "飘渺", "庄严",
        "沉重", "凌厉", "温润", "阴森", "苍茫", "巨大", "微弱", "稀薄", "混乱", "平静",
    ];
    let total = text.chars().count().max(1) as f32;
    let mut hits = 0usize;
    for adj in adjectives {
        hits += text.match_indices(adj).count();
    }
    // Add lightweight proxy for decorative wording density.
    let de_count = text.match_indices('的').count();
    ((hits as f32 * 2.0) + (de_count as f32 * 0.15)) / total
}

fn extract_unmarked_named_entities(text: &str) -> Vec<String> {
    let mut out = BTreeSet::new();
    let suffixes = ['宗', '门', '城', '谷', '宫', '派', '诀', '术', '功', '丹', '剑'];
    let stop_words = [
        "玩家", "主角", "自己", "对方", "众人", "弟子", "长老", "敌人", "灵石", "资源", "关系",
        "境界", "功法", "地图", "地点", "线索", "目标", "背包", "人物", "山门", "宗门", "城门", "大门",
    ];
    let chars = text.chars().collect::<Vec<_>>();
    for i in 0..chars.len() {
        let ch = chars[i];
        if !suffixes.contains(&ch) {
            continue;
        }
        let start = i.saturating_sub(3);
        let end = i;
        let mut candidate = chars[start..=end].iter().collect::<String>().trim().to_string();
        loop {
            let Some(first) = candidate.chars().next() else {
                break;
            };
            let is_prefix = matches!(
                first,
                '在' | '于' | '从' | '向' | '到' | '自' | '入' | '出' | '对' | '将' | '把' | '与' | '和'
            );
            if is_prefix && candidate.chars().count() > 2 {
                candidate = candidate.chars().skip(1).collect::<String>();
            } else {
                break;
            }
        }
        let len = candidate.chars().count();
        if len < 2 || len > 5 {
            continue;
        }
        if candidate.chars().any(|c| c.is_ascii_whitespace() || c.is_ascii_punctuation()) {
            continue;
        }
        if stop_words.iter().any(|s| candidate.contains(s)) {
            continue;
        }
        if candidate.ends_with('门') && candidate.chars().count() <= 2 {
            continue;
        }
        out.insert(candidate);
    }
    out.into_iter().collect::<Vec<_>>()
}

fn extract_between(text: &str, left: char, right: char, out: &mut BTreeSet<String>) {
    let mut in_span = false;
    let mut buf = String::new();
    for ch in text.chars() {
        if ch == left {
            in_span = true;
            buf.clear();
            continue;
        }
        if in_span && ch == right {
            let trimmed = buf.trim();
            if !trimmed.is_empty() && trimmed.chars().count() <= 24 {
                out.insert(trimmed.to_string());
            }
            in_span = false;
            buf.clear();
            continue;
        }
        if in_span {
            buf.push(ch);
        }
    }
}

fn clip(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }
    input.chars().take(max_chars).collect()
}

fn extract_json_value(raw: &str) -> Option<Value> {
    if let Ok(value) = serde_json::from_str::<Value>(raw) {
        return Some(value);
    }
    let start = raw.find('{')?;
    let end = raw.rfind('}')?;
    serde_json::from_str::<Value>(&raw[start..=end]).ok()
}

fn value_array(value: &Value, key: &str) -> Vec<Value> {
    value
        .get(key)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn from_bootstrap_payload(game_state: &GameState, value: Value, source: String) -> WorldRegistry {
    let mut tables = RegistryTables {
        characters: value_array(&value, "characters"),
        map_nodes: value_array(&value, "map_nodes"),
        map_edges: value_array(&value, "map_edges"),
        techniques: value_array(&value, "techniques"),
        inventory_items: value_array(&value, "inventory_items"),
        factions: value_array(&value, "factions"),
        story_state: value_array(&value, "story_state"),
        world_facts: value_array(&value, "world_facts"),
    };
    if tables.story_state.is_empty() {
        tables.story_state.push(json!({
            "chapter_index": 1,
            "chapter_goal": "establish conflict",
            "current_arc": "opening",
            "pending_conflicts": ["resource_shortage"]
        }));
    }

    WorldRegistry {
        session_id: build_session_id(&game_state.script.id),
        seed: current_unix_ts(),
        created_at: current_unix_ts(),
        llm_model: None,
        source,
        tables: normalize_tables(tables, game_state),
    }
}

fn ensure_object(mut value: Value) -> Map<String, Value> {
    if let Value::Object(map) = value {
        return map;
    }
    let mut map = Map::new();
    map.insert("raw".to_string(), std::mem::take(&mut value));
    map
}

fn normalize_tables(mut tables: RegistryTables, game_state: &GameState) -> RegistryTables {
    let player_id = game_state.player.id.clone();
    let player_name = game_state.player.name.clone();
    let player_location = game_state.player.location.clone();
    let player_realm = game_state.player.stats.cultivation_realm.name.clone();
    let player_substage = game_state.player.stats.cultivation_realm.sub_level;

    let mut has_player = false;
    let mut normalized_characters = Vec::new();
    for row in tables.characters {
        let mut obj = ensure_object(row);
        let role = obj.get("role").and_then(Value::as_str).unwrap_or("npc");
        let is_player_id = obj
            .get("character_id")
            .and_then(Value::as_str)
            .map(|id| id == player_id)
            .unwrap_or(false);
        if role == "player" || is_player_id {
            obj.insert("character_id".to_string(), Value::String(player_id.clone()));
            obj.insert("name".to_string(), Value::String(player_name.clone()));
            obj.insert("role".to_string(), Value::String("player".to_string()));
            obj.entry("location_id".to_string())
                .or_insert(Value::String(player_location.clone()));
            obj.entry("realm_stage".to_string())
                .or_insert(Value::String(player_realm.clone()));
            obj.entry("realm_substage".to_string())
                .or_insert(Value::from(player_substage));
            has_player = true;
        }
        normalized_characters.push(Value::Object(obj));
    }
    if !has_player {
        normalized_characters.insert(
            0,
            json!({
                "character_id": player_id,
                "name": player_name,
                "role": "player",
                "realm_stage": player_realm,
                "realm_substage": player_substage,
                "location_id": player_location
            }),
        );
    }
    tables.characters = normalized_characters;

    if tables.map_nodes.is_empty() {
        for loc in &game_state.script.world_setting.locations {
            tables.map_nodes.push(json!({
                "location_id": loc.id,
                "name": loc.name,
                "description": loc.description,
                "spiritual_density": loc.spiritual_energy
            }));
        }
    }
    if tables.factions.is_empty() {
        for fac in &game_state.script.world_setting.factions {
            tables.factions.push(json!({
                "faction_id": fac.id,
                "name": fac.name,
                "description": fac.description,
                "power_rank": fac.power_level
            }));
        }
    }

    tables
}

fn table_mut<'a>(tables: &'a mut RegistryTables, table: &str) -> Option<&'a mut Vec<Value>> {
    match table {
        "characters" => Some(&mut tables.characters),
        "map_nodes" => Some(&mut tables.map_nodes),
        "map_edges" => Some(&mut tables.map_edges),
        "techniques" => Some(&mut tables.techniques),
        "inventory_items" => Some(&mut tables.inventory_items),
        "factions" => Some(&mut tables.factions),
        "story_state" => Some(&mut tables.story_state),
        "world_facts" => Some(&mut tables.world_facts),
        _ => None,
    }
}

pub fn apply_registry_to_game_state(game_state: &mut GameState, registry: &WorldRegistry) {
    if game_state.player.stats.techniques.is_empty() {
        let learned = registry
            .tables
            .techniques
            .iter()
            .filter_map(|v| v.as_object())
            .filter(|obj| {
                obj.get("owner_character_id")
                    .and_then(Value::as_str)
                    .map(|owner| owner == game_state.player.id || owner == "player")
                    .unwrap_or(false)
            })
            .filter_map(|obj| obj.get("name").and_then(Value::as_str).map(ToString::to_string))
            .take(2)
            .collect::<Vec<_>>();
        if !learned.is_empty() {
            game_state.player.stats.techniques = learned;
            game_state.player.refresh_profile_views();
        }
    }

    if game_state.player.inventory.is_empty() {
        let mut items = Vec::new();
        for row in &registry.tables.inventory_items {
            let Some(obj) = row.as_object() else {
                continue;
            };
            let owner = obj
                .get("owner_character_id")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if owner != game_state.player.id && owner != "player" {
                continue;
            }
            let name = obj
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("Unknown Item")
                .to_string();
            let id = obj
                .get("item_id")
                .and_then(Value::as_str)
                .unwrap_or("item_auto")
                .to_string();
            let description = obj
                .get("effect_desc")
                .and_then(Value::as_str)
                .unwrap_or("No description")
                .to_string();
            let item_type = obj
                .get("item_type")
                .and_then(Value::as_str)
                .map(normalize_item_type)
                .unwrap_or(ItemType::Material);
            items.push(Item {
                id,
                name,
                description,
                item_type,
            });
            if items.len() >= 4 {
                break;
            }
        }
        if !items.is_empty() {
            game_state.player.inventory = items;
        }
    }
}

fn normalize_item_type(raw: &str) -> ItemType {
    let lower = raw.to_lowercase();
    if lower.contains("technique") || lower.contains("gongfa") {
        ItemType::Technique
    } else if lower.contains("artifact") || lower.contains("weapon") || lower.contains("equip") {
        ItemType::Artifact
    } else if lower.contains("medicine") || lower.contains("pill") {
        ItemType::Medicine
    } else {
        ItemType::Material
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CharacterStats, CultivationRealm, Element, Grade, Lifespan, SpiritualRoot};
    use crate::script::{InitialState, Script, ScriptType, WorldSetting};

    fn sample_state() -> GameState {
        let script = Script {
            id: "test".to_string(),
            name: "test".to_string(),
            script_type: ScriptType::Custom,
            world_setting: WorldSetting::new(),
            initial_state: InitialState {
                player_name: "LinMo".to_string(),
                player_spiritual_root: SpiritualRoot {
                    element: Element::Fire,
                    elements: vec![Element::Fire],
                    grade: Grade::Double,
                    affinity: 0.7,
                },
                starting_location: "sect".to_string(),
                starting_age: 16,
            },
        };
        GameState {
            script,
            player: crate::game_state::Character::new(
                "player".to_string(),
                "LinMo".to_string(),
                CharacterStats {
                    spiritual_root: SpiritualRoot {
                        element: Element::Fire,
                        elements: vec![Element::Fire],
                        grade: Grade::Double,
                        affinity: 0.7,
                    },
                    cultivation_realm: CultivationRealm::new("Qi".to_string(), 1, 0, 1.0),
                    techniques: vec![],
                    lifespan: Lifespan {
                        current_age: 16,
                        max_age: 120,
                        realm_bonus: 0,
                    },
                    combat_power: 100,
                },
                "sect".to_string(),
            ),
            world_state: crate::game_state::WorldState {
                locations: std::collections::HashMap::new(),
                global_events: vec![],
            },
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
        }
    }

    #[test]
    fn normalize_tables_keeps_player_name() {
        let state = sample_state();
        let registry = from_bootstrap_payload(
            &state,
            json!({
                "characters": [{"character_id":"player","name":"Wrong","role":"player"}]
            }),
            "test".to_string(),
        );
        let player = registry
            .tables
            .characters
            .iter()
            .find(|row| row.get("role").and_then(Value::as_str) == Some("player"))
            .unwrap();
        assert_eq!(player.get("name").and_then(Value::as_str), Some("LinMo"));
    }

    #[test]
    fn transactional_patch_rejects_unknown_table() {
        let state = sample_state();
        let mut registry = WorldRegistry::fallback_from_game_state(&state, "test");
        let before = registry.tables.clone();
        let result = registry.apply_state_patch_transactional(&json!({
            "unknown_table": [{"id":"x"}]
        }));
        assert!(result.is_err());
        assert_eq!(registry.tables, before);
    }

    #[test]
    fn transactional_patch_rejects_invalid_row_and_keeps_atomicity() {
        let state = sample_state();
        let mut registry = WorldRegistry::fallback_from_game_state(&state, "test");
        let before = registry.tables.clone();
        let result = registry.apply_state_patch_transactional(&json!({
            "world_facts": [
                {"fact_id":"ok_1","subject":"player","predicate":"at","object":"sect"},
                {"fact_id":"bad_2","subject":"player","predicate":"at"}
            ]
        }));
        assert!(result.is_err());
        assert_eq!(registry.tables, before);
    }

    #[test]
    fn transactional_patch_supports_replace_and_delete() {
        let state = sample_state();
        let mut registry = WorldRegistry::fallback_from_game_state(&state, "test");
        registry.tables.world_facts.push(json!({
            "fact_id":"f2","subject":"player","predicate":"at","object":"market"
        }));
        let result = registry.apply_state_patch_transactional(&json!({
            "world_facts": [
                {
                    "__op":"replace",
                    "__index":0,
                    "row":{"fact_id":"f1x","subject":"player","predicate":"at","object":"sect","confidence":0.9}
                },
                {
                    "__op":"delete",
                    "__index":1
                }
            ]
        }));
        assert!(result.is_ok());
        assert_eq!(registry.tables.world_facts.len(), 1);
        assert_eq!(
            registry.tables.world_facts[0].get("fact_id").and_then(Value::as_str),
            Some("f1x")
        );
    }

    #[test]
    fn transactional_patch_supports_upsert_by_key() {
        let state = sample_state();
        let mut registry = WorldRegistry::fallback_from_game_state(&state, "test");
        let result = registry.apply_state_patch_transactional(&json!({
            "world_facts": [
                {
                    "__op":"upsert_by_key",
                    "__key_field":"fact_id",
                    "row":{"fact_id":"fact-player-at-0","subject":"player","predicate":"at","object":"sect","confidence":0.9}
                },
                {
                    "__op":"upsert_by_key",
                    "__key_field":"fact_id",
                    "row":{"fact_id":"f_new","subject":"player","predicate":"goal","object":"market","confidence":0.9}
                }
            ]
        }));
        assert!(result.is_ok());
        assert!(registry.tables.world_facts.iter().any(|r| {
            r.get("fact_id").and_then(Value::as_str) == Some("f_new")
        }));
    }

    #[test]
    fn bootstrap_validation_rejects_missing_required_tables() {
        let state = sample_state();
        let bad = json!({
            "characters": []
        });
        let err = validate_bootstrap_payload(&bad, &state).unwrap_err();
        assert!(err.contains("missing top-level table"));
    }

    #[test]
    fn bootstrap_validation_accepts_minimal_valid_shape() {
        let state = sample_state();
        let good = json!({
            "characters": [{
                "character_id":"player",
                "name":"LinMo",
                "role":"player"
            }],
            "map_nodes": [{
                "location_id":"sect",
                "name":"Sect",
                "description":"base"
            }],
            "map_edges": [],
            "techniques": [],
            "inventory_items": [],
            "factions": [],
            "story_state": [{
                "chapter_index": 1,
                "chapter_goal": "g",
                "current_arc": "opening"
            }],
            "world_facts": []
        });
        assert!(validate_bootstrap_payload(&good, &state).is_ok());
    }

    #[test]
    fn narrative_entity_reference_validation_accepts_known_marked_entities() {
        let state = sample_state();
        let mut registry = WorldRegistry::fallback_from_game_state(&state, "test");
        registry
            .tables
            .techniques
            .push(json!({"technique_id":"t1","name":"FlameStep","description":"x"}));
        let ok = registry.validate_narrative_entity_references("你催动《FlameStep》，身形前掠。");
        assert!(ok.is_ok());
    }

    #[test]
    fn narrative_entity_reference_validation_rejects_unknown_marked_entities() {
        let state = sample_state();
        let registry = WorldRegistry::fallback_from_game_state(&state, "test");
        let err = registry.validate_narrative_entity_references("你得到《UnknownRelic》。");
        assert!(err.is_err());
    }

    #[test]
    fn narrative_contract_rejects_short_text() {
        let state = sample_state();
        let registry = WorldRegistry::fallback_from_game_state(&state, "test");
        let err = registry
            .validate_turn_narrative_contract("你到达山门，获得一块灵石。")
            .unwrap_err();
        assert!(err.contains("length out of range"));
    }

    #[test]
    fn narrative_entity_reference_detects_unmarked_named_entity_suffix() {
        let state = sample_state();
        let mut registry = WorldRegistry::fallback_from_game_state(&state, "test");
        registry
            .tables
            .factions
            .push(json!({"faction_id":"f1","name":"青云宗"}));
        let ok = registry.validate_narrative_entity_references("你在青云宗山门外停步。");
        assert!(ok.is_ok());
    }
}


