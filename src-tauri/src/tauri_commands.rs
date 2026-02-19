use crate::game_engine::GameEngine;
use crate::game_state::GameState;
use crate::event_log::EventImportance;
use crate::context_builder::{build_context_bundle, ContextBuildInput, ContextBundle};
use crate::entity_store::{EntityQuery, EntityStore};
use crate::entity_types::{
    EntityCandidateRequest, EntityType, ResolvedEntity, StoredEntity, ValidationStatus,
};
use crate::entity_validator::resolve_candidate;
use crate::llm_runtime_config::{
    clear_runtime_llm_config, get_llm_config_status as runtime_llm_config_status,
    resolve_llm_config, set_runtime_llm_config, LLMConfigStatus,
};
use crate::llm_service::{LLMConfig, LLMRequest, LLMService};
use crate::memory_layers::{ChapterSummary, MemoryEntry, WorldFact};
use crate::novel_generator::{Novel, NovelGenerator};
use crate::numerical_system::{Action, Context, StatChange};
use crate::plot_consistency::{
    get_runtime_policy, reset_runtime_policy, update_runtime_policy, validate_and_repair_plot_update,
    ConsistencyPolicy, ConsistencyReport,
};
use crate::plot_engine::{
    PlayerAction, PlayerOption, PlotEngine, PlotInteractionState, PlotSettings, PlotState,
};
use crate::save_load::{MigrationBatchReport, SaveInfo};
use crate::script::Script;
use crate::app_error::AppError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;
use tauri::State;

static ENTITY_STORE: OnceLock<Mutex<EntityStore>> = OnceLock::new();
static MEMORY_LAYERS: OnceLock<Mutex<crate::memory_layers::MemoryLayers>> = OnceLock::new();

fn entity_store() -> &'static Mutex<EntityStore> {
    ENTITY_STORE.get_or_init(|| Mutex::new(EntityStore::new()))
}

fn memory_layers() -> &'static Mutex<crate::memory_layers::MemoryLayers> {
    MEMORY_LAYERS.get_or_init(|| Mutex::new(crate::memory_layers::MemoryLayers::new()))
}

fn build_plot_context_for_generation(
    game_state: &GameState,
    plot_state: &PlotState,
    action: &PlayerAction,
) -> Option<ContextBundle> {
    let mut recent_context_lines = plot_state
        .current_chapter
        .content
        .iter()
        .rev()
        .take(6)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>();
    recent_context_lines.push(format!(
        "战斗后状态: injury_level={}, reputation={}, enmity={}",
        game_state.player.combat_status.injury_level,
        game_state.player.combat_status.reputation,
        game_state.player.combat_status.enmity
    ));

    let input = ContextBuildInput {
        world_id: game_state.script.id.clone(),
        run_id: "active-run".to_string(),
        scene_id: plot_state.current_scene.id.clone(),
        character_ids: vec![game_state.player.id.clone()],
        map_node_id: Some(game_state.player.location.clone()),
        player_intent: if action.content.trim().is_empty() {
            None
        } else {
            Some(action.content.clone())
        },
        recent_context_lines,
        token_budget: 260,
    };

    let store = entity_store().lock().ok()?;
    let memory = memory_layers().lock().ok()?;
    Some(build_context_bundle(&store, &memory, &input))
}

fn render_generation_context(bundle: &ContextBundle) -> String {
    let mut lines = Vec::new();
    for line in bundle.hard_facts.iter().take(4) {
        lines.push(format!("事实: {}", line));
    }
    for line in bundle.recent_context.iter().take(3) {
        lines.push(format!("近文: {}", line));
    }
    for line in bundle.chapter_summaries.iter().take(2) {
        lines.push(format!("章节: {}", line));
    }
    for line in bundle.recent_events.iter().take(2) {
        lines.push(format!("事件: {}", line));
    }
    if lines.is_empty() {
        return String::new();
    }
    format!("\n\n[外置记忆+上下文窗口]\n{}", lines.join("\n"))
}

fn has_consistency_issue(report: &ConsistencyReport, code: &str) -> bool {
    report.issues.iter().any(|issue| issue.code == code)
}

fn chapter_goal_regeneration_hint(interaction_count: u8) -> String {
    let goal = match interaction_count % 5 {
        0 => "冲突升级",
        1 => "角色成长",
        2 => "资源变化",
        3 => "关系变化",
        _ => "伏笔建立",
    };
    format!(
        "\n\n[章节目标重生成约束]\n本段必须明确命中：{}；禁止原地复述，必须出现可验证的新变化。",
        goal
    )
}

fn hollow_expression_regeneration_hint() -> &'static str {
    "\n\n[叙事厚度重生成约束]\n禁止空洞套话与重复句式；至少补足环境、动作、心理三类中的两类，并给出可验证的因果变化。"
}

fn is_hollow_expression(text: &str) -> bool {
    let t = text.trim();
    if t.is_empty() {
        return true;
    }
    let filler_patterns = [
        "不由得",
        "一时间",
        "片刻之后",
        "气氛凝重",
        "心中一凛",
        "强大的气息",
        "隐隐作痛",
        "不禁",
        "仿佛",
        "似乎",
    ];
    let filler_hits = filler_patterns
        .iter()
        .filter(|p| t.contains(**p))
        .count();

    let clauses = t
        .split(['。', '！', '？', ';', '；', '\n'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    let repeated_clause = if clauses.len() >= 3 {
        let unique = clauses.iter().copied().collect::<std::collections::BTreeSet<_>>();
        unique.len() * 10 <= clauses.len() * 7
    } else {
        false
    };

    let sensory_words = ["风", "雨", "雾", "声", "光", "影", "痛", "冷", "热", "血"];
    let action_words = ["挥", "斩", "踏", "退", "冲", "挡", "运转", "催动"];
    let mental_words = ["犹豫", "执念", "惊", "怒", "惧", "定神", "思索", "决意"];
    let mut covered = 0usize;
    if sensory_words.iter().any(|w| t.contains(w)) {
        covered += 1;
    }
    if action_words.iter().any(|w| t.contains(w)) {
        covered += 1;
    }
    if mental_words.iter().any(|w| t.contains(w)) {
        covered += 1;
    }

    (filler_hits >= 3 && t.chars().count() >= 30) || repeated_clause || covered <= 1
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct CombatExplanation {
    dominant_factors: Vec<String>,
    reversal_factors: Vec<String>,
    summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CombatStrategy {
    Cautious,
    Aggressive,
    Survival,
}

fn strategy_label(strategy: CombatStrategy) -> &'static str {
    match strategy {
        CombatStrategy::Cautious => "谨慎周旋",
        CombatStrategy::Aggressive => "强攻压制",
        CombatStrategy::Survival => "保命脱战",
    }
}

fn choose_combat_strategy(
    status: &crate::game_state::CombatAftermathStatus,
    option_hint: &str,
) -> (CombatStrategy, &'static str) {
    let hint = option_hint.to_lowercase();
    if status.injury_level >= 6 || status.qi_deviation >= 6 {
        return (CombatStrategy::Survival, "伤势/气机压力过高");
    }
    if hint.contains("保命") || hint.contains("撤") || hint.contains("脱战") {
        return (CombatStrategy::Survival, "玩家指令倾向保命");
    }
    if hint.contains("强攻")
        || hint.contains("硬拼")
        || hint.contains("冲锋")
        || hint.contains("搏命")
        || status.enmity >= 4
    {
        return (CombatStrategy::Aggressive, "高压进攻意图");
    }
    if hint.contains("试探")
        || hint.contains("防守")
        || hint.contains("周旋")
        || status.injury_level >= 3
    {
        return (CombatStrategy::Cautious, "以稳为主降低风险");
    }
    (CombatStrategy::Aggressive, "默认进攻策略")
}

fn strategy_power_modifier_pct(
    strategy: CombatStrategy,
    status: &crate::game_state::CombatAftermathStatus,
) -> i32 {
    match strategy {
        CombatStrategy::Aggressive => {
            if status.injury_level <= 2 && status.qi_deviation <= 3 {
                8
            } else {
                -6
            }
        }
        CombatStrategy::Cautious => {
            if status.injury_level >= 3 || status.qi_deviation >= 3 {
                5
            } else {
                -2
            }
        }
        CombatStrategy::Survival => -8,
    }
}

fn evaluate_environment_combat_modifier(game_state: &GameState) -> (i32, String) {
    let energy = location_spiritual_energy(game_state).unwrap_or(0.5);
    let status = &game_state.player.combat_status;
    let mut delta = 0i32;
    let mut reasons = Vec::new();

    if energy >= 0.8 {
        delta += 6;
        reasons.push(format!("高灵气场域增幅({:.2})", energy));
    } else if energy <= 0.25 {
        delta -= 6;
        reasons.push(format!("低灵气环境压制({:.2})", energy));
    } else if energy >= 0.55 {
        delta += 3;
        reasons.push(format!("中高灵气稳定输出({:.2})", energy));
    }

    if status.qi_deviation >= 6 {
        delta -= 5;
        reasons.push("气机紊乱削弱环境适应".to_string());
    }
    if status.injury_level >= 6 {
        delta -= 4;
        reasons.push("重伤状态难以利用地利".to_string());
    }

    let reason = if reasons.is_empty() {
        "环境影响中性".to_string()
    } else {
        reasons.join(" / ")
    };
    (delta.clamp(-20, 20), reason)
}

fn detect_player_styles(stats: &crate::models::CharacterStats) -> Vec<&'static str> {
    let mut styles = std::collections::BTreeSet::new();
    for tech in &stats.techniques {
        let t = tech.to_lowercase();
        if t.contains("剑") || t.contains("sword") {
            styles.insert("sword");
        }
        if t.contains("刀") || t.contains("blade") {
            styles.insert("blade");
        }
        if t.contains("拳") || t.contains("体") || t.contains("body") {
            styles.insert("body");
        }
        if t.contains("符") || t.contains("阵") || t.contains("talisman") || t.contains("array") {
            styles.insert("talisman");
        }
    }
    styles.into_iter().collect::<Vec<_>>()
}

fn extract_styles_from_text(hint: &str) -> Vec<&'static str> {
    let h = hint.to_lowercase();
    let mut styles = Vec::new();
    if h.contains("剑") || h.contains("sword") {
        styles.push("sword");
    }
    if h.contains("刀") || h.contains("blade") {
        styles.push("blade");
    }
    if h.contains("拳") || h.contains("体修") || h.contains("body") {
        styles.push("body");
    }
    if h.contains("符") || h.contains("阵") || h.contains("talisman") || h.contains("array") {
        styles.push("talisman");
    }
    styles.sort();
    styles.dedup();
    styles
}

fn detect_enemy_style(hint: &str) -> Option<&'static str> {
    let h = hint.to_lowercase();
    if h.contains("剑") || h.contains("sword") {
        return Some("sword");
    }
    if h.contains("刀") || h.contains("blade") {
        return Some("blade");
    }
    if h.contains("拳") || h.contains("体修") || h.contains("body") {
        return Some("body");
    }
    if h.contains("符") || h.contains("阵") || h.contains("talisman") || h.contains("array") {
        return Some("talisman");
    }
    None
}

fn style_counter_delta(player_style: &str, enemy_style: &str) -> i32 {
    crate::combat_style_rules::counter_delta(player_style, enemy_style)
}

fn evaluate_style_counter_modifier(
    stats: &crate::models::CharacterStats,
    combat_hint: &str,
) -> (i32, String) {
    let enemy_style = detect_enemy_style(combat_hint);
    let mut player_styles = detect_player_styles(stats);
    if player_styles.is_empty() {
        let hint_styles = extract_styles_from_text(combat_hint);
        if let Some(enemy) = enemy_style {
            if let Some(style) = hint_styles.iter().find(|s| **s != enemy) {
                player_styles.push(*style);
            }
        }
        if player_styles.is_empty() {
            if let Some(style) = hint_styles.first() {
                player_styles.push(*style);
            }
        }
    }
    let Some(enemy_style) = enemy_style else {
        return (0, "未识别敌方流派".to_string());
    };
    if player_styles.is_empty() {
        return (0, "我方流派未成型".to_string());
    }
    let mut best = 0;
    let mut best_style = "";
    for style in &player_styles {
        let d = style_counter_delta(style, enemy_style);
        if d > best {
            best = d;
            best_style = style;
        }
    }
    if best == 0 {
        return (
            0,
            format!("未形成明确克制（我方={:?}, 敌方={}）", player_styles, enemy_style),
        );
    }
    (
        best,
        format!("流派克制：{} 克制 {}", best_style, enemy_style),
    )
}

fn build_combat_explanation(
    action_result: &crate::numerical_system::ActionResult,
    player_realm_level: u32,
    player_combat_power: u64,
    numeric_guard_reason: Option<&str>,
    strategy_note: Option<&str>,
) -> CombatExplanation {
    let mut dominant_factors = vec![
        format!("境界层级: {}", player_realm_level),
        format!("战力基线: {}", player_combat_power),
    ];
    if action_result.success {
        dominant_factors.push("行动结果: 成功".to_string());
    } else {
        dominant_factors.push("行动结果: 受阻".to_string());
    }
    if let Some(note) = strategy_note {
        dominant_factors.push(format!("策略匹配: {}", note));
    }

    let mut reversal_factors = Vec::new();
    if let Some(reason) = numeric_guard_reason {
        reversal_factors.push(format!("数值守门裁决: {}", reason));
    }
    if !action_result.success {
        reversal_factors.push("执行环节出现失败分支".to_string());
    }
    if reversal_factors.is_empty() {
        reversal_factors.push("未触发显著反转因子".to_string());
    }

    let summary = format!(
        "主导因素={}；反转因素={}",
        dominant_factors.join(" / "),
        reversal_factors.join(" / ")
    );

    CombatExplanation {
        dominant_factors,
        reversal_factors,
        summary,
    }
}

fn apply_combat_aftermath(
    game_state: &mut GameState,
    combat_success: bool,
    strategy: Option<CombatStrategy>,
) -> String {
    let status = &mut game_state.player.combat_status;
    let strategy = strategy.unwrap_or(CombatStrategy::Aggressive);
    if combat_success {
        status.reputation = status.reputation.saturating_add(2);
        status.enmity = status
            .enmity
            .saturating_add(if strategy == CombatStrategy::Aggressive { 2 } else { 1 });
        if status.injury_level > 0 && strategy != CombatStrategy::Aggressive {
            status.injury_level = status.injury_level.saturating_sub(1);
        }
        game_state.player.social_profile.favor = game_state
            .player
            .social_profile
            .favor
            .saturating_add(if strategy == CombatStrategy::Aggressive {
                2
            } else {
                1
            });
        if strategy == CombatStrategy::Cautious {
            game_state.player.social_profile.mentor_bond =
                game_state.player.social_profile.mentor_bond.saturating_add(1);
        }
        game_state.player.social_profile.vendetta = game_state
            .player
            .social_profile
            .vendetta
            .saturating_sub(1)
            .max(0);
    } else {
        status.reputation = status.reputation.saturating_sub(1);
        status.enmity = status
            .enmity
            .saturating_add(if strategy == CombatStrategy::Survival { 1 } else { 2 });
        let injury_gain = if strategy == CombatStrategy::Survival {
            1
        } else {
            2
        };
        status.injury_level = status.injury_level.saturating_add(injury_gain).min(10);
        game_state.player.social_profile.vendetta = game_state
            .player
            .social_profile
            .vendetta
            .saturating_add(if strategy == CombatStrategy::Survival {
                1
            } else {
                2
            });
        if strategy == CombatStrategy::Aggressive {
            game_state.player.social_profile.sect_affinity =
                game_state.player.social_profile.sect_affinity.saturating_sub(1);
        }
    }
    normalize_social_profile(&mut game_state.player.social_profile);

    format!(
        "战后状态更新：伤势={}, 威望={}, 仇恨={}",
        status.injury_level, status.reputation, status.enmity
    )
}

fn location_spiritual_energy(game_state: &GameState) -> Option<f32> {
    game_state
        .world_state
        .locations
        .get(&game_state.player.location)
        .map(|loc| loc.spiritual_energy)
}

fn cultivation_gain_multiplier_from_location(spiritual_energy: Option<f32>) -> f32 {
    let e = spiritual_energy.unwrap_or(0.5);
    (0.8 + e * 0.6).clamp(0.6, 1.6)
}

fn select_encounter_text(spiritual_energy: f32, seed: u64) -> &'static str {
    const LOW_RISK: [&str; 3] = [
        "途中遇到散修试探，你稳住阵脚后化解冲突。",
        "林间突发灵兽惊扰，短暂交锋后你安全脱身。",
        "遭遇路匪埋伏，但对方见势不妙迅速退去。",
    ];
    const MID_RISK: [&str; 3] = [
        "前路突现阵法余波，你强行破阵，气息略有紊乱。",
        "遭遇敌对门派巡逻，数轮试探后双方各自撤离。",
        "秘径中爆发灵压乱流，你顶住冲击后继续赶路。",
    ];
    const HIGH_RISK: [&str; 3] = [
        "深处魔息翻涌，突遭强敌伏击，激战后方得脱身。",
        "禁地边缘出现失控妖潮，你强行突围，代价不小。",
        "高阶修士威压横扫而过，你硬抗冲击后重整气机。",
    ];
    let bucket = if spiritual_energy >= 0.85 {
        &HIGH_RISK
    } else if spiritual_energy >= 0.5 {
        &MID_RISK
    } else {
        &LOW_RISK
    };
    let idx = (seed as usize) % bucket.len();
    bucket[idx]
}

fn push_growth_log(game_state: &mut GameState, entry: impl Into<String>) {
    let log = &mut game_state.player.growth_log;
    log.push(entry.into());
    const MAX_GROWTH_LOG: usize = 240;
    if log.len() > MAX_GROWTH_LOG {
        let overflow = log.len() - MAX_GROWTH_LOG;
        log.drain(0..overflow);
    }
}

fn normalize_social_profile(profile: &mut crate::game_state::SocialProfile) {
    profile.sect_affinity = profile.sect_affinity.clamp(-50, 50);
    profile.mentor_bond = profile.mentor_bond.clamp(-50, 50);
    profile.favor = profile.favor.clamp(-50, 50);
    profile.vendetta = profile.vendetta.clamp(0, 100);

    let stance_score =
        profile.sect_affinity + profile.mentor_bond + profile.favor - profile.vendetta;
    profile.camp_stance = if stance_score >= 12 {
        "righteous".to_string()
    } else if stance_score <= -8 {
        "demonic".to_string()
    } else {
        "neutral".to_string()
    };
}

fn apply_travel_and_encounter(
    game_state: &mut GameState,
    plot_state: &mut PlotState,
    target_location: &str,
) -> Result<(String, bool), String> {
    let target_node = game_state
        .world_state
        .locations
        .get(target_location)
        .cloned()
        .ok_or_else(|| format!("目标地点不存在: {}", target_location))?;
    let reachable_ids = compute_reachable_location_ids(game_state);
    if !reachable_ids.iter().any(|id| id == target_location) {
        return Err(format!(
            "当前状态下无法前往该地点: {}（请先降低伤势或分段行进）",
            target_location
        ));
    }
    let from_location = game_state.player.location.clone();
    if from_location == target_location {
        return Ok(("你已在当前地点，无需移动。".to_string(), false));
    }

    let from_name = game_state
        .world_state
        .locations
        .get(&from_location)
        .map(|loc| loc.name.clone())
        .unwrap_or_else(|| from_location.clone());
    let target_name = target_node.name.clone();

    game_state.player.location = target_location.to_string();
    plot_state.current_scene.location = target_location.to_string();
    game_state.game_time.advance_days(1);

    let total_days = game_state.game_time.total_days;
    let energy = target_node.spiritual_energy;
    let status = &game_state.player.combat_status;
    let cfg = crate::travel_rules::rules();
    let weighted_prob = (cfg.encounter_base_prob
        + (energy as f64 * cfg.encounter_energy_weight)
        + (status.enmity.max(0) as f64 * cfg.encounter_enmity_weight)
        + (status.qi_deviation as f64 * cfg.encounter_qi_weight))
        .clamp(cfg.encounter_prob_min, cfg.encounter_prob_max);
    let mut hash_acc = total_days as u64;
    for b in target_location.as_bytes() {
        hash_acc = hash_acc.wrapping_mul(131).wrapping_add(u64::from(*b));
    }
    let roll = (hash_acc % 100) as f64 / 100.0;
    let encounter_triggered = roll < weighted_prob;
    let mut message = format!("你从{}前往{}，行程耗时1日。", from_name, target_name);

    if encounter_triggered {
        let status = &mut game_state.player.combat_status;
        let encounter_text = select_encounter_text(energy, hash_acc);
        status.enmity = status.enmity.saturating_add(1);
        game_state.player.social_profile.vendetta =
            game_state.player.social_profile.vendetta.saturating_add(1);
        if energy >= 0.8 {
            status.injury_level = status.injury_level.saturating_add(1).min(10);
            message.push(' ');
            message.push_str(encounter_text);
            message.push_str(" 伤势+1。");
        } else {
            message.push(' ');
            message.push_str(encounter_text);
            message.push_str(" 仇恨+1。");
        }
    } else {
        message.push_str(" 途中未遭遇显著冲突。");
        game_state.player.social_profile.favor =
            game_state.player.social_profile.favor.saturating_add(1);
    }
    if target_location.contains("sect") || target_name.contains("宗") {
        game_state.player.social_profile.sect_affinity =
            game_state.player.social_profile.sect_affinity.saturating_add(1);
    }
    normalize_social_profile(&mut game_state.player.social_profile);

    push_growth_log(
        game_state,
        format!(
            "行程变更：{} -> {}{}",
            from_name,
            target_name,
            if encounter_triggered { "（触发遭遇）" } else { "" }
        ),
    );
    plot_state.current_chapter.content.push(message.clone());
    Ok((message, encounter_triggered))
}

fn compute_reachable_location_ids(game_state: &GameState) -> Vec<String> {
    let cfg = crate::travel_rules::rules();
    let current_id = game_state.player.location.clone();
    let Some(current_node) = game_state.world_state.locations.get(&current_id) else {
        let mut all = game_state
            .world_state
            .locations
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        all.sort();
        return all;
    };
    let realm = &game_state.player.stats.cultivation_realm;
    let status = &game_state.player.combat_status;
    let (mobility, max_energy) = compute_travel_capabilities(realm, status);

    let mut by_gap = game_state
        .world_state
        .locations
        .values()
        .map(|loc| {
            let gap = (loc.spiritual_energy - current_node.spiritual_energy).abs();
            (loc.id.clone(), loc.spiritual_energy, gap)
        })
        .collect::<Vec<_>>();
    by_gap.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
    let nearest_two = if cfg.nearby_fallback_enabled && by_gap.len() >= cfg.nearby_fallback_min_location_count {
        by_gap
            .iter()
            .take(cfg.nearby_fallback_count)
            .map(|(id, _, _)| id.clone())
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let mut reachable = by_gap
        .into_iter()
        .filter(|(_, energy, gap)| *gap <= mobility && *energy <= max_energy + 0.05)
        .map(|(id, _, _)| id)
        .collect::<Vec<_>>();
    reachable.push(current_id);
    reachable.extend(nearest_two);
    reachable.sort();
    reachable.dedup();
    reachable
}

fn compute_travel_capabilities(
    realm: &crate::models::CultivationRealm,
    status: &crate::game_state::CombatAftermathStatus,
) -> (f32, f32) {
    let cfg = crate::travel_rules::rules();
    let mobility = (cfg.mobility_base
        + (realm.level as f32 * cfg.mobility_per_realm)
        + ((realm.sub_level.min(3)) as f32 * cfg.mobility_per_sub_level)
        - (status.injury_level as f32 * cfg.mobility_injury_penalty)
        - (status.qi_deviation as f32 * cfg.mobility_qi_penalty))
        .clamp(cfg.mobility_min, cfg.mobility_max);
    let max_energy = (cfg.max_energy_base
        + (realm.level as f32 * cfg.max_energy_per_realm)
        - (status.injury_level as f32 * cfg.max_energy_injury_penalty))
        .clamp(cfg.max_energy_min, cfg.max_energy_max);
    (mobility, max_energy)
}

fn build_energy_path(
    game_state: &GameState,
    target_id: &str,
    mobility: f32,
    max_energy: f32,
) -> Option<Vec<String>> {
    let current_id = game_state.player.location.clone();
    if current_id == target_id {
        return Some(vec![current_id]);
    }
    let current_energy = game_state.world_state.locations.get(&current_id)?.spiritual_energy;
    let target_energy = game_state.world_state.locations.get(target_id)?.spiritual_energy;
    if (current_energy - target_energy).abs() <= mobility && target_energy <= max_energy + 0.05 {
        return Some(vec![current_id, target_id.to_string()]);
    }

    let mut path = vec![current_id.clone()];
    let mut cursor_energy = current_energy;
    let mut visited = std::collections::BTreeSet::new();
    visited.insert(current_id.clone());

    for _ in 0..6 {
        let mut candidates = game_state
            .world_state
            .locations
            .values()
            .filter(|loc| !visited.contains(&loc.id))
            .filter(|loc| (loc.spiritual_energy - cursor_energy).abs() <= mobility)
            .filter(|loc| loc.spiritual_energy <= max_energy + 0.05)
            .collect::<Vec<_>>();
        candidates.sort_by(|a, b| {
            let da = (a.spiritual_energy - target_energy).abs();
            let db = (b.spiritual_energy - target_energy).abs();
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        });
        let next = candidates.first()?;
        path.push(next.id.clone());
        if next.id == target_id {
            return Some(path);
        }
        visited.insert(next.id.clone());
        cursor_energy = next.spiritual_energy;
    }
    None
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MapLocationOverview {
    pub location_id: String,
    pub name: String,
    pub spiritual_energy: f32,
    pub energy_gap: f32,
    pub reachable: bool,
    pub risk_tier: String,
    pub estimated_steps: u32,
    pub suggested_path: Vec<String>,
}

fn compute_map_overview(game_state: &GameState) -> Vec<MapLocationOverview> {
    let current_id = game_state.player.location.clone();
    let (mobility, max_energy) = compute_travel_capabilities(
        &game_state.player.stats.cultivation_realm,
        &game_state.player.combat_status,
    );
    let reachable = compute_reachable_location_ids(game_state)
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    let current_energy = game_state
        .world_state
        .locations
        .get(&current_id)
        .map(|loc| loc.spiritual_energy)
        .unwrap_or(0.0);

    let mut nodes = game_state
        .world_state
        .locations
        .values()
        .map(|loc| {
            let risk_tier = if loc.spiritual_energy >= 0.85 {
                "high"
            } else if loc.spiritual_energy >= 0.5 {
                "medium"
            } else {
                "low"
            };
            let suggested_path =
                build_energy_path(game_state, &loc.id, mobility, max_energy).unwrap_or_default();
            let estimated_steps = suggested_path.len().saturating_sub(1) as u32;
            MapLocationOverview {
                location_id: loc.id.clone(),
                name: loc.name.clone(),
                spiritual_energy: loc.spiritual_energy,
                energy_gap: (loc.spiritual_energy - current_energy).abs(),
                reachable: reachable.contains(&loc.id),
                risk_tier: risk_tier.to_string(),
                estimated_steps,
                suggested_path,
            }
        })
        .collect::<Vec<_>>();
    nodes.sort_by(|a, b| a.energy_gap.partial_cmp(&b.energy_gap).unwrap_or(std::cmp::Ordering::Equal));
    nodes
}

fn is_high_risk_technique_name(name: &str) -> bool {
    let t = name.to_lowercase();
    t.contains("禁")
        || t.contains("爆")
        || t.contains("噬")
        || t.contains("逆")
        || t.contains("魔")
        || t.contains("forbidden")
        || t.contains("berserk")
}

fn apply_breakthrough_failure_consequences(
    game_state: &mut GameState,
    action_result: &mut crate::numerical_system::ActionResult,
    high_risk_technique: bool,
) {
    let old_deviation = game_state.player.combat_status.qi_deviation;
    let old_injury = game_state.player.combat_status.injury_level;
    let deviation_gain = if high_risk_technique { 2 } else { 1 };
    let (new_deviation, new_injury) = {
        let status = &mut game_state.player.combat_status;
        status.qi_deviation = status.qi_deviation.saturating_add(deviation_gain).min(10);
        if high_risk_technique {
            status.injury_level = status.injury_level.saturating_add(1).min(10);
        }
        (status.qi_deviation, status.injury_level)
    };
    action_result.stat_changes.push(StatChange {
        stat_name: "qi_deviation".to_string(),
        old_value: old_deviation.to_string(),
        new_value: new_deviation.to_string(),
    });
    action_result.events.push(format!(
        "突破失败后果：气机紊乱+{}（当前 {}）",
        deviation_gain, new_deviation
    ));

    if high_risk_technique {
        action_result.stat_changes.push(StatChange {
            stat_name: "injury_level".to_string(),
            old_value: old_injury.to_string(),
            new_value: new_injury.to_string(),
        });
        action_result
            .events
            .push("高风险功法反噬：伤势+1，出现走火入魔征兆".to_string());
    }

    if new_deviation >= 7 {
        action_result
            .events
            .push("气机紊乱接近临界，需休整或降风险修炼".to_string());
    }

    action_result.description = format!(
        "{}（突破失败后果：气机紊乱+{}）",
        action_result.description, deviation_gain
    );
    push_growth_log(
        game_state,
        format!(
            "突破受挫：气机紊乱 {} -> {}{}",
            old_deviation,
            new_deviation,
            if high_risk_technique {
                "；触发高风险反噬"
            } else {
                ""
            }
        ),
    );
}

fn breakthrough_blocked_by_qi_deviation(qi_deviation: u8) -> bool {
    qi_deviation >= 8
}

fn evaluate_technique_semantic_modifier(
    stats: &crate::models::CharacterStats,
) -> (i32, Vec<String>, bool) {
    let mut percent_delta = 0i32;
    let mut reasons = Vec::new();
    let roots = stats
        .spiritual_root
        .effective_elements()
        .iter()
        .map(|e| format!("{:?}", e).to_lowercase())
        .collect::<Vec<_>>();
    let mut has_high_risk = false;

    for tech in &stats.techniques {
        let t = tech.to_lowercase();
        if (t.contains("fire") || t.contains("炎") || t.contains("焰") || t.contains("火"))
            && roots.iter().any(|r| r == "fire")
        {
            percent_delta += 8;
            reasons.push(format!("功法 `{}` 与火灵根适配", tech));
        }
        if (t.contains("ice") || t.contains("寒") || t.contains("冰"))
            && (roots.iter().any(|r| r == "water") || roots.iter().any(|r| r == "ice"))
        {
            percent_delta += 6;
            reasons.push(format!("功法 `{}` 与水/冰灵根适配", tech));
        }
        if t.contains("元婴") && stats.cultivation_realm.level < 4 {
            percent_delta -= 10;
            reasons.push(format!("功法 `{}` 境界门槛偏高，发挥受限", tech));
        }
        if is_high_risk_technique_name(tech) {
            has_high_risk = true;
            percent_delta += 5;
            reasons.push(format!("功法 `{}` 高风险强行驱动，短时增益", tech));
        }
    }

    (percent_delta.clamp(-30, 30), reasons, has_high_risk)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GenerationTimingSample {
    total_ms: u64,
    plot_gen_ms: u64,
    option_gen_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GenerationTimingSummary {
    pub sample_count: usize,
    pub total_p50_ms: u64,
    pub total_p95_ms: u64,
    pub total_p99_ms: u64,
    pub plot_gen_p95_ms: u64,
    pub option_gen_p95_ms: u64,
}

fn percentile_u64(samples: &[u64], p: f64) -> u64 {
    if samples.is_empty() {
        return 0;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn parse_generation_timing_sample(diag: &str) -> Option<GenerationTimingSample> {
    let marker = "耗时(ms)：";
    let payload = diag.split(marker).last()?.trim();
    let mut total_ms = None;
    let mut plot_gen_ms = None;
    let mut option_gen_ms = None;

    for part in payload.split([',', '，', ';', '；']) {
        let mut iter = part.trim().splitn(2, '=');
        let key = iter.next()?.trim();
        let value = iter.next()?.trim().parse::<u64>().ok()?;
        match key {
            "total" => total_ms = Some(value),
            "plot_gen" => plot_gen_ms = Some(value),
            "option_gen" => option_gen_ms = Some(value),
            _ => {}
        }
    }

    Some(GenerationTimingSample {
        total_ms: total_ms?,
        plot_gen_ms: plot_gen_ms?,
        option_gen_ms: option_gen_ms?,
    })
}

fn summarize_generation_timing_diagnostics(
    diagnostics: &[String],
) -> Option<GenerationTimingSummary> {
    let samples = diagnostics
        .iter()
        .filter_map(|diag| parse_generation_timing_sample(diag))
        .collect::<Vec<_>>();
    if samples.is_empty() {
        return None;
    }

    let total = samples.iter().map(|s| s.total_ms).collect::<Vec<_>>();
    let plot = samples.iter().map(|s| s.plot_gen_ms).collect::<Vec<_>>();
    let option = samples.iter().map(|s| s.option_gen_ms).collect::<Vec<_>>();

    Some(GenerationTimingSummary {
        sample_count: samples.len(),
        total_p50_ms: percentile_u64(&total, 0.50),
        total_p95_ms: percentile_u64(&total, 0.95),
        total_p99_ms: percentile_u64(&total, 0.99),
        plot_gen_p95_ms: percentile_u64(&plot, 0.95),
        option_gen_p95_ms: percentile_u64(&option, 0.95),
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl From<anyhow::Error> for ErrorResponse {
    fn from(err: anyhow::Error) -> Self {
        ErrorResponse {
            error: err.to_string(),
        }
    }
}

fn map_error(context: &str, err: impl Into<AppError>) -> String {
    err.into().with_context(context).to_string()
}

fn validate_slot_id(slot_id: u32) -> Result<(), AppError> {
    if (1..=99).contains(&slot_id) {
        Ok(())
    } else {
        Err(AppError::new(
            crate::app_error::AppErrorKind::InvalidInput,
            format!("存档槽位必须在 1-99 之间，当前为 {}", slot_id),
        ))
    }
}

fn validate_file_path(path: &str, allowed_exts: &[&str]) -> Result<(), AppError> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(AppError::new(
            crate::app_error::AppErrorKind::NotFound,
            format!("文件不存在: {}", path),
        ));
    }
    if !p.is_file() {
        return Err(AppError::new(
            crate::app_error::AppErrorKind::InvalidInput,
            format!("路径不是文件: {}", path),
        ));
    }
    if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
        if allowed_exts.iter().any(|allowed| ext.eq_ignore_ascii_case(allowed)) {
            return Ok(());
        }
    }
    Err(AppError::new(
        crate::app_error::AppErrorKind::InvalidInput,
        format!("文件格式不支持: {}", path),
    ))
}

fn validate_endpoint(endpoint: &str) -> Result<(), AppError> {
    let trimmed = endpoint.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        Ok(())
    } else {
        Err(AppError::new(
            crate::app_error::AppErrorKind::InvalidInput,
            "LLM endpoint 必须以 http:// 或 https:// 开头",
        ))
    }
}

fn validate_non_empty(value: &str, label: &str) -> Result<(), AppError> {
    if value.trim().is_empty() {
        Err(AppError::new(
            crate::app_error::AppErrorKind::InvalidInput,
            format!("{}不能为空", label),
        ))
    } else {
        Ok(())
    }
}

fn validate_output_path(path: &str, allowed_exts: &[&str]) -> Result<(), AppError> {
    let p = Path::new(path);
    if let Some(parent) = p.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            return Err(AppError::new(
                crate::app_error::AppErrorKind::NotFound,
                format!("输出目录不存在: {}", parent.display()),
            ));
        }
    }
    if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
        if allowed_exts.iter().any(|allowed| ext.eq_ignore_ascii_case(allowed)) {
            return Ok(());
        }
    }
    Err(AppError::new(
        crate::app_error::AppErrorKind::InvalidInput,
        format!("输出文件格式不支持: {}", path),
    ))
}

fn validate_llm_config_input(input: &LLMConfigInput) -> Result<(), AppError> {
    validate_endpoint(&input.endpoint)?;
    validate_non_empty(&input.model, "模型名称")?;
    if input.max_tokens == 0 || input.max_tokens > 8192 {
        return Err(AppError::new(
            crate::app_error::AppErrorKind::InvalidInput,
            "max_tokens 必须在 1-8192 之间",
        ));
    }
    if !(0.0..=2.0).contains(&input.temperature) {
        return Err(AppError::new(
            crate::app_error::AppErrorKind::InvalidInput,
            "temperature 必须在 0-2 之间",
        ));
    }
    if input.api_key.trim().is_empty() {
        let endpoint = input.endpoint.to_lowercase();
        let local = endpoint.contains("localhost") || endpoint.contains("127.0.0.1");
        if !local {
            return Err(AppError::new(
                crate::app_error::AppErrorKind::InvalidInput,
                "API Key 不能为空",
            ));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LLMConfigInput {
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[tauri::command]
pub async fn set_llm_config(input: LLMConfigInput) -> Result<String, String> {
    validate_llm_config_input(&input).map_err(|e| map_error("LLM 配置校验失败", e))?;
    let config = LLMConfig {
        endpoint: input.endpoint,
        api_key: input.api_key,
        model: input.model,
        max_tokens: input.max_tokens,
        temperature: input.temperature,
    };
    LLMService::new(config.clone()).map_err(|e| map_error("LLM 配置校验失败", e))?;
    set_runtime_llm_config(config);
    Ok("LLM 配置已更新".to_string())
}

#[tauri::command]
pub async fn clear_llm_config() -> Result<String, String> {
    clear_runtime_llm_config();
    Ok("已清除运行时 LLM 配置".to_string())
}

#[tauri::command]
pub async fn get_llm_config_status() -> Result<LLMConfigStatus, String> {
    Ok(runtime_llm_config_status())
}

#[tauri::command]
pub async fn test_llm_connection() -> Result<String, String> {
    let cfg = resolve_llm_config().ok_or_else(|| "未检测到 LLM 配置".to_string())?;
    let service = LLMService::new(cfg).map_err(|e| e.to_string())?;
    let response = service
        .generate(LLMRequest {
            prompt: "请回复：连接成功".to_string(),
            max_tokens: Some(32),
            temperature: Some(0.1),
        })
        .await
        .map_err(|e| e.to_string())?;
    Ok(response.text)
}

#[tauri::command]
pub async fn initialize_game(
    script: Script,
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<GameState, String> {
    let mut engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    engine.initialize_game(script).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn execute_player_action(
    action: PlayerAction,
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<String, String> {
    let total_started = Instant::now();
    let (mut game_state, mut plot_state) = {
        let engine = match engine.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let game_state = engine.get_current_state().map_err(|e| e.to_string())?;
        let plot_state = engine.get_plot_state().map_err(|e| e.to_string())?;
        (game_state, plot_state)
    };
    plot_state.interaction_state = PlotInteractionState::Resolving;

    let plot_engine = PlotEngine::new();
    let context = Context {
        location: game_state.player.location.clone(),
        time_of_day: "day".to_string(),
        weather: None,
    };

    let mut action_result = plot_engine
        .process_player_action(
            &action,
            &game_state.player.stats,
            &plot_state.current_scene.available_options,
            &context,
        )
        .map_err(|e| e.to_string())?;

    let mut combat_guard_reason: Option<String> = None;
    let mut combat_strategy_note: Option<String> = None;
    let mut should_emit_combat_explanation = false;
    if let Some(selected_option_id) = action.selected_option_id {
        if let Some(selected_option) = plot_state.current_scene.available_options.get(selected_option_id)
        {
            match &selected_option.action {
                Action::Cultivate => {
                    let old_power = game_state.player.stats.combat_power;
                    let energy = location_spiritual_energy(&game_state);
                    let gain_multiplier = cultivation_gain_multiplier_from_location(energy);
                    let gain = ((old_power as f32 * 0.03 * gain_multiplier).round() as u64).max(1);
                    let new_power = old_power.saturating_add(gain);
                    game_state.player.stats.combat_power = new_power;
                    action_result.stat_changes.push(StatChange {
                        stat_name: "combat_power".to_string(),
                        old_value: old_power.to_string(),
                        new_value: new_power.to_string(),
                    });
                    action_result.description = format!(
                        "{} 战力提升了 {}（区域灵气修正 {:.2}x）。",
                        action_result.description, gain, gain_multiplier
                    );
                    push_growth_log(
                        &mut game_state,
                        format!(
                            "修炼成长：战力 {} -> {}（灵气 {:.2}）",
                            old_power,
                            new_power,
                            energy.unwrap_or(0.5)
                        ),
                    );
                }
                Action::Breakthrough => {
                    let high_risk_technique = game_state
                        .player
                        .stats
                        .techniques
                        .iter()
                        .any(|t| is_high_risk_technique_name(t));
                    if breakthrough_blocked_by_qi_deviation(
                        game_state.player.combat_status.qi_deviation,
                    ) {
                        action_result.success = false;
                        action_result.events.push("突破被中断：气机紊乱接近失控".to_string());
                        action_result.description = format!(
                            "{}（气机紊乱过高，强行突破失败）",
                            action_result.description
                        );
                        apply_breakthrough_failure_consequences(
                            &mut game_state,
                            &mut action_result,
                            high_risk_technique,
                        );
                    } else if action_result.success
                        && game_state.player.stats.cultivation_realm.sub_level < 3
                    {
                        let old_sub = game_state.player.stats.cultivation_realm.sub_level;
                        game_state.player.stats.cultivation_realm.sub_level += 1;
                        game_state.player.stats.cultivation_realm.power_multiplier *= 1.2;
                        game_state.player.stats.update_combat_power();
                        action_result.stat_changes.push(StatChange {
                            stat_name: "realm_sub_level".to_string(),
                            old_value: old_sub.to_string(),
                            new_value: game_state
                                .player
                                .stats
                                .cultivation_realm
                                .sub_level
                                .to_string(),
                        });
                        let realm_name = game_state.player.stats.cultivation_realm.name.clone();
                        let new_sub = game_state.player.stats.cultivation_realm.sub_level;
                        let growth_entry =
                            format!("突破成长：{} 小境界 {} -> {}", realm_name, old_sub, new_sub);
                        push_growth_log(&mut game_state, growth_entry);
                    } else if !action_result.success {
                        apply_breakthrough_failure_consequences(
                            &mut game_state,
                            &mut action_result,
                            high_risk_technique,
                        );
                    }
                }
                Action::Combat { .. } => {
                    should_emit_combat_explanation = true;
                    let option_hint = format!("{} {}", selected_option.description, action.content);
                    let (strategy, strategy_reason) =
                        choose_combat_strategy(&game_state.player.combat_status, &option_hint);
                    let strategy_delta_pct =
                        strategy_power_modifier_pct(strategy, &game_state.player.combat_status);
                    if strategy_delta_pct != 0 {
                        let old_power = game_state.player.stats.combat_power;
                        let scaled = (old_power as i128)
                            .saturating_mul((100 + strategy_delta_pct) as i128)
                            / 100i128;
                        let new_power = scaled.max(1) as u64;
                        game_state.player.stats.combat_power = new_power;
                        action_result.stat_changes.push(StatChange {
                            stat_name: "combat_power".to_string(),
                            old_value: old_power.to_string(),
                            new_value: new_power.to_string(),
                        });
                        action_result.description = format!(
                            "{}（行为策略修正: {} {}%）",
                            action_result.description,
                            strategy_label(strategy),
                            strategy_delta_pct
                        );
                        push_growth_log(
                            &mut game_state,
                            format!(
                                "战斗策略影响：{}，战力 {} -> {}",
                                strategy_label(strategy),
                                old_power,
                                new_power
                            ),
                        );
                    }
                    combat_strategy_note = Some(format!(
                        "{}（{}）",
                        strategy_label(strategy),
                        strategy_reason
                    ));
                    let (env_delta_pct, env_reason) =
                        evaluate_environment_combat_modifier(&game_state);
                    if env_delta_pct != 0 {
                        let old_power = game_state.player.stats.combat_power;
                        let scaled = (old_power as i128)
                            .saturating_mul((100 + env_delta_pct) as i128)
                            / 100i128;
                        let new_power = scaled.max(1) as u64;
                        game_state.player.stats.combat_power = new_power;
                        action_result.stat_changes.push(StatChange {
                            stat_name: "combat_power".to_string(),
                            old_value: old_power.to_string(),
                            new_value: new_power.to_string(),
                        });
                        action_result.description = format!(
                            "{}（环境修正: {}%）",
                            action_result.description, env_delta_pct
                        );
                        push_growth_log(
                            &mut game_state,
                            format!("环境影响战斗：战力 {} -> {}（{}）", old_power, new_power, env_reason),
                        );
                    }
                    if !env_reason.is_empty() {
                        action_result.description =
                            format!("{}；环境依据：{}", action_result.description, env_reason);
                    }
                    let (style_delta_pct, style_reason) =
                        evaluate_style_counter_modifier(&game_state.player.stats, &option_hint);
                    if style_delta_pct != 0 {
                        let old_power = game_state.player.stats.combat_power;
                        let scaled = (old_power as i128)
                            .saturating_mul((100 + style_delta_pct) as i128)
                            / 100i128;
                        let new_power = scaled.max(1) as u64;
                        game_state.player.stats.combat_power = new_power;
                        action_result.stat_changes.push(StatChange {
                            stat_name: "combat_power".to_string(),
                            old_value: old_power.to_string(),
                            new_value: new_power.to_string(),
                        });
                        action_result.description = format!(
                            "{}（流派克制修正: {}%）",
                            action_result.description, style_delta_pct
                        );
                        push_growth_log(
                            &mut game_state,
                            format!("流派克制影响：战力 {} -> {}（{}）", old_power, new_power, style_reason),
                        );
                    }
                    action_result.description =
                        format!("{}；克制依据：{}", action_result.description, style_reason);
                    let (semantic_delta_pct, semantic_reasons, high_risk_technique) =
                        evaluate_technique_semantic_modifier(&game_state.player.stats);
                    if semantic_delta_pct != 0 {
                        let old_power = game_state.player.stats.combat_power;
                        let scaled = (old_power as i128)
                            .saturating_mul((100 + semantic_delta_pct) as i128)
                            / 100i128;
                        let new_power = scaled.max(1) as u64;
                        game_state.player.stats.combat_power = new_power;
                        action_result.stat_changes.push(StatChange {
                            stat_name: "combat_power".to_string(),
                            old_value: old_power.to_string(),
                            new_value: new_power.to_string(),
                        });
                        action_result.description = format!(
                            "{}（功法语义修正: {}%）",
                            action_result.description, semantic_delta_pct
                        );
                        push_growth_log(
                            &mut game_state,
                            format!("功法语义影响战斗：战力 {} -> {}", old_power, new_power),
                        );
                    }
                    if !semantic_reasons.is_empty() {
                        action_result.description = format!(
                            "{}；语义依据：{}",
                            action_result.description,
                            semantic_reasons.join(" / ")
                        );
                    }
                    let realm_level = game_state.player.stats.cultivation_realm.level;
                    let check = crate::numeric_guard::validate_character_combat_power(
                        realm_level,
                        game_state.player.stats.combat_power,
                    );
                    if check.accepted && check.normalized {
                        if let Some(v) = check.normalized_value {
                            let old_power = game_state.player.stats.combat_power;
                            let new_power = v.round().max(1.0) as u64;
                            game_state.player.stats.combat_power = new_power;
                            action_result.stat_changes.push(StatChange {
                                stat_name: "combat_power".to_string(),
                                old_value: old_power.to_string(),
                                new_value: new_power.to_string(),
                            });
                            if let Some(reason) = check.reason {
                                combat_guard_reason = Some(reason.clone());
                                action_result.description = format!(
                                    "{}（数值校验：{}）",
                                    action_result.description, reason
                                );
                            }
                        }
                    } else if !check.accepted {
                        action_result.success = false;
                        combat_guard_reason = check.reason.clone();
                        action_result.description = format!(
                            "{}（数值校验未通过：{}）",
                            action_result.description,
                            check.reason.unwrap_or_else(|| "未知原因".to_string())
                        );
                    }
                    let aftermath_summary =
                        apply_combat_aftermath(&mut game_state, action_result.success, Some(strategy));
                    if high_risk_technique {
                        let status = &mut game_state.player.combat_status;
                        status.injury_level = status.injury_level.saturating_add(1).min(10);
                        action_result.events.push("高风险功法反噬：伤势+1".to_string());
                    }
                    action_result.events.push(aftermath_summary.clone());
                    action_result.description =
                        format!("{}；{}", action_result.description, aftermath_summary);
                    push_growth_log(&mut game_state, format!("战斗成长：{}", aftermath_summary));
                }
                Action::Rest | Action::Custom { .. } => {}
            }
        }
    }

    game_state.game_time.advance_days(1);
    let timestamp = u64::from(game_state.game_time.total_days);

    let context_bundle = build_plot_context_for_generation(&game_state, &plot_state, &action);
    let mut plot_state_for_generation = plot_state.clone();
    if let Some(bundle) = &context_bundle {
        let inject = render_generation_context(bundle);
        if !inject.is_empty() {
            plot_state_for_generation.current_scene.description =
                format!("{}{}", plot_state_for_generation.current_scene.description, inject);
        }
    }

    let mut plot_update = plot_engine
        .advance_plot_async(&plot_state_for_generation, &action_result)
        .await;
    let plot_generation_ms = total_started.elapsed().as_millis();

    if let Some(bundle) = context_bundle {
        let ctx_diag = format!(
            "上下文注入：facts={}, recent_ctx={}, chapter_refs={}, recent_events={}, token_used={}",
            bundle.hard_facts.len(),
            bundle.recent_context.len(),
            bundle.chapter_summaries.len(),
            bundle.recent_events.len(),
            bundle.token_budget_used
        );
        match &mut plot_update.generation_diagnostics {
            Some(diag) => {
                diag.push('；');
                diag.push_str(&ctx_diag);
            }
            None => {
                plot_update.generation_diagnostics = Some(ctx_diag);
            }
        }
    }

    let mut consistency_report = validate_and_repair_plot_update(
        &plot_state,
        &plot_update,
        &action_result,
        game_state.player.stats.cultivation_realm.level,
        game_state.player.stats.combat_power,
        &game_state.player.name,
    );

    if has_consistency_issue(&consistency_report, "chapter_goal_weak") {
        let mut regenerated_state = plot_state_for_generation.clone();
        regenerated_state.current_scene.description = format!(
            "{}{}",
            regenerated_state.current_scene.description,
            chapter_goal_regeneration_hint(plot_state.current_chapter.interaction_count)
        );

        let mut regenerated_update = plot_engine
            .advance_plot_async(&regenerated_state, &action_result)
            .await;
        let regenerated_report = validate_and_repair_plot_update(
            &plot_state,
            &regenerated_update,
            &action_result,
            game_state.player.stats.cultivation_realm.level,
            game_state.player.stats.combat_power,
            &game_state.player.name,
        );

        let improved_goal_hit = !has_consistency_issue(&regenerated_report, "chapter_goal_weak");
        let not_worse_risk = regenerated_report.risk_score() <= consistency_report.risk_score();
        if improved_goal_hit || not_worse_risk {
            match &mut regenerated_update.generation_diagnostics {
                Some(diag) => diag.push_str("；章节目标重生成：accepted"),
                None => {
                    regenerated_update.generation_diagnostics =
                        Some("章节目标重生成：accepted".to_string())
                }
            }
            plot_update = regenerated_update;
            consistency_report = regenerated_report;
        } else {
            match &mut plot_update.generation_diagnostics {
                Some(diag) => diag.push_str("；章节目标重生成：rejected"),
                None => {
                    plot_update.generation_diagnostics =
                        Some("章节目标重生成：rejected".to_string())
                }
            }
        }
    }

    if is_hollow_expression(&plot_update.plot_text) {
        let mut regenerated_state = plot_state_for_generation.clone();
        regenerated_state.current_scene.description = format!(
            "{}{}",
            regenerated_state.current_scene.description,
            hollow_expression_regeneration_hint()
        );
        let mut regenerated_update = plot_engine
            .advance_plot_async(&regenerated_state, &action_result)
            .await;
        let regenerated_report = validate_and_repair_plot_update(
            &plot_state,
            &regenerated_update,
            &action_result,
            game_state.player.stats.cultivation_realm.level,
            game_state.player.stats.combat_power,
            &game_state.player.name,
        );

        let improved_density = !is_hollow_expression(&regenerated_update.plot_text);
        let not_worse_risk = regenerated_report.risk_score() <= consistency_report.risk_score() + 2;
        if improved_density || not_worse_risk {
            match &mut regenerated_update.generation_diagnostics {
                Some(diag) => diag.push_str("；叙事厚度重生成：accepted"),
                None => {
                    regenerated_update.generation_diagnostics =
                        Some("叙事厚度重生成：accepted".to_string())
                }
            }
            plot_update = regenerated_update;
            consistency_report = regenerated_report;
        } else {
            match &mut plot_update.generation_diagnostics {
                Some(diag) => diag.push_str("；叙事厚度重生成：rejected"),
                None => {
                    plot_update.generation_diagnostics =
                        Some("叙事厚度重生成：rejected".to_string())
                }
            }
        }
    }
    if let Some(text) = consistency_report.repaired_plot_text.clone() {
        plot_update.plot_text = text;
    }
    if consistency_report.force_non_waiting {
        plot_update.is_waiting_for_input = false;
    }
    if let Some(next_location) = consistency_report.override_location.clone() {
        plot_state.current_scene.location = next_location;
    }
    if let Some(summary) = consistency_report.override_chapter_summary.clone() {
        plot_update.chapter_summary = Some(summary);
    }
    if let Some(diag) = consistency_report.to_diagnostics() {
        match &mut plot_update.generation_diagnostics {
            Some(existing) => {
                existing.push('；');
                existing.push_str(&diag);
            }
            None => {
                plot_update.generation_diagnostics = Some(diag);
            }
        }
    }

    let log_entry = if let Some(selected_option_id) = action.selected_option_id {
        if let Some(selected_option) = plot_state.current_scene.available_options.get(selected_option_id) {
            match &selected_option.action {
                Action::Combat { .. } => Some((
                    "combat",
                    format!("Player engaged in combat: {}", selected_option.description),
                    EventImportance::Important,
                )),
                Action::Breakthrough => Some((
                    "breakthrough_attempt",
                    format!("Player attempted breakthrough: {}", selected_option.description),
                    EventImportance::Important,
                )),
                Action::Custom { .. } | Action::Cultivate | Action::Rest => Some((
                    "player_action",
                    selected_option.description.clone(),
                    EventImportance::Normal,
                )),
            }
        } else {
            None
        }
    } else if matches!(action.action_type, crate::plot_engine::ActionType::FreeText) {
        Some((
            "player_free_text",
            action.content.clone(),
            EventImportance::Normal,
        ))
    } else {
        None
    };

    plot_state.last_action_result = Some(action_result.clone());
    plot_state.append_segment(plot_update.plot_text.clone());

    if let Some(title) = plot_update.chapter_title.clone() {
        if !title.trim().is_empty() {
            plot_state.current_chapter.title = title.trim().to_string();
            plot_state.current_scene.name = plot_state.current_chapter.title.clone();
        }
    }

    if plot_update.is_waiting_for_input {
        plot_state.current_chapter.interaction_count = plot_state
            .current_chapter
            .interaction_count
            .saturating_add(1);
    }

    if plot_update.chapter_end {
        plot_state.finalize_chapter(plot_update.chapter_title, plot_update.chapter_summary);
    }

    plot_state.last_generation_diagnostics = plot_update.generation_diagnostics.clone();
    let risk_score = consistency_report.risk_score();
    plot_state.last_consistency_risk_score = if risk_score > 0 {
        Some(risk_score)
    } else {
        None
    };

    // 用最新段落更新场景描述，避免选项生成长期绑定旧描述导致“选项不变”。
    if !plot_update.plot_text.trim().is_empty() {
        plot_state.current_scene.description = plot_update.plot_text.trim().to_string();
    }

    let previous_options = plot_state.current_scene.available_options.clone();
    let options_started = Instant::now();

    let mut option_source: String = if plot_update.is_waiting_for_input {
        if !plot_update.available_options.is_empty() {
            plot_state.current_scene.available_options = plot_update.available_options;
            "llm_structured".to_string()
        } else {
            let llm_regenerated = plot_engine.generate_player_options_with_llm(
                &plot_state.current_scene,
                &game_state.player.stats,
            );
            let (mut regenerated_options, mut source) = if let Some(options) = llm_regenerated {
                (options, "llm_regenerated".to_string())
            } else {
                (
                    plot_engine
                        .generate_player_options(&plot_state.current_scene, &game_state.player.stats),
                    "rule_fallback".to_string(),
                )
            };

            if regenerated_options.is_empty() {
                regenerated_options = previous_options;
                source = "previous_reused".to_string();
            }

            // 通过时间推进对兜底选项做轻量轮转，确保连续交互时选项呈现有变化。
            if !regenerated_options.is_empty() {
                let rotation =
                    (game_state.game_time.total_days as usize) % regenerated_options.len();
                regenerated_options.rotate_left(rotation);
                for (idx, option) in regenerated_options.iter_mut().enumerate() {
                    option.id = idx;
                }
            }
            plot_state.current_scene.available_options = regenerated_options;
            source
        }
    } else {
        plot_state.current_scene.available_options.clear();
        "not_waiting_for_input".to_string()
    };

    if plot_update.is_waiting_for_input
        && !plot_update.chapter_end
        && plot_state.current_scene.available_options.is_empty()
    {
        plot_update.is_waiting_for_input = false;
        option_source = "consistency_non_waiting_fallback".to_string();
        match &mut plot_state.last_generation_diagnostics {
            Some(diag) => diag.push_str("；一致性兜底：无选项等待态已改为自动推进"),
            None => {
                plot_state.last_generation_diagnostics =
                    Some("一致性兜底：无选项等待态已改为自动推进".to_string())
            }
        }
    }
    let options_generation_ms = options_started.elapsed().as_millis();

    plot_state.is_waiting_for_input = plot_update.is_waiting_for_input;
    plot_state.recalculate_interaction_state();
    if plot_update.chapter_end {
        plot_state.interaction_state = PlotInteractionState::Cooldown;
    }

    plot_state.last_option_generation_source = Some(option_source.clone());
    match &mut plot_state.last_generation_diagnostics {
        Some(diag) => {
            diag.push_str(&format!("；选项来源：{}", option_source));
        }
        None => {
            plot_state.last_generation_diagnostics = Some(format!("选项来源：{}", option_source));
        }
    }
    let total_ms = total_started.elapsed().as_millis();
    match &mut plot_state.last_generation_diagnostics {
        Some(diag) => {
            diag.push_str(&format!(
                "；耗时(ms)：total={},plot_gen={},option_gen={}",
                total_ms, plot_generation_ms, options_generation_ms
            ));
        }
        None => {
            plot_state.last_generation_diagnostics = Some(format!(
                "耗时(ms)：total={},plot_gen={},option_gen={}",
                total_ms, plot_generation_ms, options_generation_ms
            ));
        }
    }

    let mut engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    if let Some((event_type, message, importance)) = log_entry {
        engine.log_event(timestamp, event_type, message, importance);
    }

    if should_emit_combat_explanation {
        let explanation = build_combat_explanation(
            &action_result,
            game_state.player.stats.cultivation_realm.level,
            game_state.player.stats.combat_power,
            combat_guard_reason.as_deref(),
            combat_strategy_note.as_deref(),
        );
        let payload = serde_json::to_string(&explanation).unwrap_or(explanation.summary);
        engine.log_event(
            timestamp,
            "combat_explanation",
            payload,
            EventImportance::Important,
        );
    }

    let _npc_reactions = engine
        .process_npc_reactions_for_events(&plot_update.triggered_events)
        .map_err(|e| e.to_string())?;

    // V2 记忆回写：短期事件 / 中期章节摘要 / 长期事实
    if let Ok(mut memory) = memory_layers().lock() {
        for (idx, event_text) in plot_update.triggered_events.iter().enumerate() {
            memory.push_recent_event(
                MemoryEntry {
                    event_id: format!("evt-{}-{}", timestamp, idx),
                    summary: event_text.clone(),
                    turn: timestamp,
                },
                120,
            );
        }

        if let Some(last_chapter) = plot_state.chapters.last() {
            memory.upsert_chapter_summary(ChapterSummary {
                chapter_id: format!("chapter-{}", last_chapter.index),
                title: last_chapter.title.clone(),
                summary: if last_chapter.summary.trim().is_empty() {
                    last_chapter
                        .content
                        .last()
                        .cloned()
                        .unwrap_or_else(|| "（无摘要）".to_string())
                } else {
                    last_chapter.summary.clone()
                },
            });
        }

        memory.upsert_world_fact(WorldFact {
            fact_id: format!("fact-location-{}", timestamp),
            subject: "player".to_string(),
            predicate: "at".to_string(),
            object: game_state.player.location.clone(),
        });
    }

    engine
        .update_current_state(game_state)
        .map_err(|e| e.to_string())?;
    engine
        .update_plot_state(plot_state)
        .map_err(|e| e.to_string())?;

    Ok(plot_update.plot_text)
}

#[tauri::command]
pub async fn get_game_state(engine: State<'_, Mutex<GameEngine>>) -> Result<GameState, String> {
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    engine.get_current_state().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_game(slot_id: u32, engine: State<'_, Mutex<GameEngine>>) -> Result<(), String> {
    validate_slot_id(slot_id).map_err(|e| map_error("保存存档失败", e))?;
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    engine.save_game(slot_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_game(
    slot_id: u32,
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<GameState, String> {
    validate_slot_id(slot_id).map_err(|e| map_error("加载存档失败", e))?;
    let mut engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    engine.load_game(slot_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_save_slots(engine: State<'_, Mutex<GameEngine>>) -> Result<Vec<SaveInfo>, String> {
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    engine.list_saves().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_reachable_locations(
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<Vec<String>, String> {
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let game_state = engine.get_current_state().map_err(|e| e.to_string())?;
    Ok(compute_reachable_location_ids(&game_state))
}

#[tauri::command]
pub async fn get_map_overview(
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<Vec<MapLocationOverview>, String> {
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let game_state = engine.get_current_state().map_err(|e| e.to_string())?;
    Ok(compute_map_overview(&game_state))
}

#[tauri::command]
pub async fn travel_to_location(
    location_id: String,
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<String, String> {
    validate_non_empty(&location_id, "目标地点")
        .map_err(|e| map_error("移动失败", e))?;
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let mut game_state = engine.get_current_state().map_err(|e| e.to_string())?;
    let mut plot_state = engine.get_plot_state().map_err(|e| e.to_string())?;
    let timestamp_before = u64::from(game_state.game_time.total_days);

    let (message, encounter_triggered) =
        apply_travel_and_encounter(&mut game_state, &mut plot_state, &location_id)?;
    let timestamp = u64::from(game_state.game_time.total_days.max(timestamp_before as u32));
    engine.log_event(
        timestamp,
        "travel",
        message.clone(),
        EventImportance::Normal,
    );
    if encounter_triggered {
        engine.log_event(
            timestamp,
            "encounter",
            format!("移动遭遇：{}", message),
            EventImportance::Important,
        );
    }

    engine
        .update_current_state(game_state)
        .map_err(|e| e.to_string())?;
    engine
        .update_plot_state(plot_state)
        .map_err(|e| e.to_string())?;
    Ok(message)
}

#[tauri::command]
pub async fn migrate_all_saves(
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<MigrationBatchReport, String> {
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    engine.migrate_all_saves().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_script(
    script_path: String,
    _engine: State<'_, Mutex<GameEngine>>,
) -> Result<Script, String> {
    use crate::script_manager::ScriptManager;

    validate_file_path(&script_path, &["json"]).map_err(|e| map_error("加载剧本失败", e))?;
    let manager = ScriptManager::new();
    manager
        .load_custom_script(&script_path)
        .map_err(|e| map_error("加载剧本失败", e))
}

#[tauri::command]
pub async fn generate_random_script() -> Result<Script, String> {
    use crate::script_manager::ScriptManager;

    let manager = ScriptManager::new();
    manager
        .generate_random_script()
        .await
        .map_err(|e| map_error("随机剧本生成失败", e))
}

#[tauri::command]
pub async fn parse_novel_characters(novel_path: String) -> Result<Vec<String>, String> {
    use crate::script_manager::ScriptManager;

    validate_file_path(&novel_path, &["txt", "md"]).map_err(|e| map_error("解析小说角色失败", e))?;
    let manager = ScriptManager::new();
    manager
        .extract_novel_characters(&novel_path)
        .map_err(|e| map_error("解析小说角色失败", e))
}

#[tauri::command]
pub async fn load_existing_novel(
    novel_path: String,
    selected_character: String,
) -> Result<Script, String> {
    use crate::script_manager::ScriptManager;

    validate_file_path(&novel_path, &["txt", "md"]).map_err(|e| map_error("导入现有小说失败", e))?;
    if selected_character.trim().is_empty() {
        return Err(map_error(
            "导入现有小说失败",
            AppError::new(crate::app_error::AppErrorKind::InvalidInput, "请选择有效角色"),
        ));
    }
    let manager = ScriptManager::new();
    manager
        .load_existing_novel(&novel_path, &selected_character)
        .map_err(|e| map_error("导入现有小说失败", e))
}

#[tauri::command]
pub async fn get_player_options(
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<Vec<PlayerOption>, String> {
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let plot_state = engine.get_plot_state().map_err(|e| e.to_string())?;
    Ok(plot_state.current_scene.available_options)
}

#[tauri::command]
pub async fn initialize_plot(
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<PlotState, String> {
    let (player_name, realm_name, spiritual_root, location) = {
        let engine = match engine.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let state = engine.get_current_state().map_err(|e| e.to_string())?;
        (
            state.player.name,
            state.player.stats.cultivation_realm.name,
            state.player.stats.spiritual_root.display_elements(),
            state.player.location,
        )
    };

    let plot_engine = PlotEngine::new();
    let opening = plot_engine
        .generate_opening_plot_async(&player_name, &realm_name, &spiritual_root, &location)
        .await;

    let opening_options = if opening.options.is_empty() {
        None
    } else {
        Some(
            opening
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
                .collect(),
        )
    };

    let mut engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    engine
        .initialize_plot_with_opening(opening.text, opening_options)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_plot_state(
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<PlotState, String> {
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    engine.get_plot_state().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_plot_settings(
    settings: PlotSettings,
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<PlotState, String> {
    if settings.min_interactions_per_chapter == 0
        || settings.max_interactions_per_chapter == 0
        || settings.min_interactions_per_chapter > settings.max_interactions_per_chapter
    {
        return Err(map_error(
            "更新剧情设置失败",
            AppError::new(
                crate::app_error::AppErrorKind::InvalidInput,
                "每章互动次数范围不合法",
            ),
        ));
    }
    if settings.target_chapter_words_min == 0
        || settings.target_chapter_words_max == 0
        || settings.target_chapter_words_min > settings.target_chapter_words_max
    {
        return Err(map_error(
            "更新剧情设置失败",
            AppError::new(
                crate::app_error::AppErrorKind::InvalidInput,
                "章节字数范围不合法",
            ),
        ));
    }
    if settings.novel_style.trim().is_empty() {
        return Err(map_error(
            "更新剧情设置失败",
            AppError::new(crate::app_error::AppErrorKind::InvalidInput, "小说风格不能为空"),
        ));
    }
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    engine
        .update_plot_settings(settings)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_consistency_policy() -> Result<ConsistencyPolicy, String> {
    Ok(get_runtime_policy())
}

#[tauri::command]
pub async fn update_consistency_policy(
    policy: ConsistencyPolicy,
) -> Result<ConsistencyPolicy, String> {
    update_runtime_policy(policy).map_err(|e| map_error("更新一致性策略失败", AppError::new(
        crate::app_error::AppErrorKind::InvalidInput,
        e,
    )))
}

#[tauri::command]
pub async fn reset_consistency_policy() -> Result<ConsistencyPolicy, String> {
    reset_runtime_policy().map_err(|e| {
        map_error(
            "重置一致性策略失败",
            AppError::new(crate::app_error::AppErrorKind::InvalidInput, e),
        )
    })
}

#[tauri::command]
pub async fn generate_novel(
    title: String,
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<Novel, String> {
    validate_non_empty(&title, "小说标题").map_err(|e| map_error("生成小说失败", e))?;
    let (events, plot_state) = {
        let engine = engine.lock().map_err(|e| e.to_string())?;
        let state = engine.get_current_state().map_err(|e| e.to_string())?;
        let plot_state = engine.get_plot_state().ok();
        (state.event_history, plot_state)
    };
    if let Some(plot_state) = plot_state {
        if !plot_state.chapters.is_empty() || !plot_state.current_chapter.content.is_empty() {
            return Ok(generate_novel_from_plot_state(&title, &plot_state));
        }
    }
    generate_novel_from_events(&title, &events).await
}

#[tauri::command]
pub async fn export_novel(novel: Novel, output_path: String) -> Result<(), String> {
    validate_output_path(&output_path, &["txt"]).map_err(|e| map_error("导出小说失败", e))?;
    export_novel_to_path(&novel, &output_path)
}

#[tauri::command]
pub async fn summarize_generation_diagnostics(
    diagnostics: Vec<String>,
) -> Result<GenerationTimingSummary, String> {
    summarize_generation_timing_diagnostics(&diagnostics)
        .ok_or_else(|| "未找到可解析的耗时诊断数据".to_string())
}

async fn generate_novel_from_events(title: &str, events: &[crate::event_log::GameEvent]) -> Result<Novel, String> {
    let generator = NovelGenerator::new();
    generator.generate_novel(title.to_string(), events).await
}

fn generate_novel_from_plot_state(title: &str, plot_state: &PlotState) -> Novel {
    let generator = NovelGenerator::new();
    generator.generate_chronicle_from_plot(title.to_string(), plot_state)
}

fn export_novel_to_path(novel: &Novel, output_path: &str) -> Result<(), String> {
    let generator = NovelGenerator::new();
    generator.export_to_file(novel, output_path)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitEntitiesInput {
    pub world_id: String,
    pub run_id: String,
    pub candidates: Vec<EntityCandidateRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitEntitiesResponse {
    pub committed: Vec<StoredEntity>,
    pub rejected: Vec<ResolvedEntity>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateCandidatesInput {
    pub entity_type: EntityType,
    pub hint: Option<String>,
    pub count: Option<u8>,
}

#[tauri::command]
pub async fn generate_entity_candidates(
    input: GenerateCandidatesInput,
) -> Result<Vec<EntityCandidateRequest>, String> {
    let count = input.count.unwrap_or(1).clamp(1, 5) as usize;
    let hint = input.hint.unwrap_or_else(|| "default".to_string());
    let candidates = (0..count)
        .map(|idx| {
            let payload = match input.entity_type {
                EntityType::Technique => json!({
                    "techniqueId": format!("tech_{}_{}", hint, idx + 1),
                    "name": format!("{} Technique {}", hint, idx + 1),
                    "tags": ["generated"],
                    "realmRequirement": 1,
                    "rootAffinity": ["Fire"],
                    "basePower": 32.0,
                    "riskTags": [],
                    "description": "Generated candidate technique"
                }),
                EntityType::Character => json!({
                    "characterId": format!("char_{}_{}", hint, idx + 1),
                    "name": format!("{} Character {}", hint, idx + 1),
                    "realm": "Qi Condensation",
                    "personalityTags": ["cautious"],
                    "relationshipEdges": [],
                    "knownTechniques": []
                }),
                EntityType::MapNode => json!({
                    "nodeId": format!("map_{}_{}", hint, idx + 1),
                    "name": format!("{} Region {}", hint, idx + 1),
                    "nodeType": "wilderness",
                    "dangerTier": 3,
                    "auraDensity": 0.8,
                    "factionControl": "neutral",
                    "connectedNodes": []
                }),
                EntityType::Item => json!({
                    "itemId": format!("item_{}_{}", hint, idx + 1),
                    "name": format!("{} Item {}", hint, idx + 1),
                    "itemType": "artifact",
                    "qualityTier": 2,
                    "description": "Generated candidate item"
                }),
            };
            EntityCandidateRequest {
                entity_type: input.entity_type,
                payload,
                source_trace_id: Some(format!("candidate-gen-{}", idx + 1)),
            }
        })
        .collect();
    Ok(candidates)
}

#[tauri::command]
pub async fn commit_entities(input: CommitEntitiesInput) -> Result<CommitEntitiesResponse, String> {
    if input.world_id.trim().is_empty() || input.run_id.trim().is_empty() {
        return Err("world_id and run_id must be non-empty".to_string());
    }

    let mut committed = Vec::new();
    let mut rejected = Vec::new();
    let mut store = entity_store()
        .lock()
        .map_err(|_| "failed to lock entity store".to_string())?;
    let mut memory = memory_layers()
        .lock()
        .map_err(|_| "failed to lock memory layers".to_string())?;

    for candidate in &input.candidates {
        let resolved = resolve_candidate(candidate);
        if matches!(resolved.validation_report.status, ValidationStatus::Rejected) {
            rejected.push(resolved);
            continue;
        }

        let stored = StoredEntity {
            world_id: input.world_id.clone(),
            run_id: input.run_id.clone(),
            entity_id: resolved.entity_id.clone(),
            entity_type: resolved.entity_type,
            payload: resolved.payload.clone(),
            updated_at: 0,
        };
        store.upsert(stored.clone());
        committed.push(stored.clone());

        memory.upsert_world_fact(crate::memory_layers::WorldFact {
            fact_id: format!("fact_{}_{}", input.run_id, resolved.entity_id),
            subject: resolved.entity_id,
            predicate: "defined_as".to_string(),
            object: format!("{:?}", resolved.entity_type),
        });
    }

    Ok(CommitEntitiesResponse { committed, rejected })
}

#[tauri::command]
pub async fn query_entities(query: EntityQuery) -> Result<Vec<StoredEntity>, String> {
    if query.world_id.trim().is_empty() || query.run_id.trim().is_empty() {
        return Err("world_id and run_id must be non-empty".to_string());
    }
    let store = entity_store()
        .lock()
        .map_err(|_| "failed to lock entity store".to_string())?;
    Ok(store.list_by_query(&query))
}

#[tauri::command]
pub async fn build_context_bundle_command(input: ContextBuildInput) -> Result<ContextBundle, String> {
    let store = entity_store()
        .lock()
        .map_err(|_| "failed to lock entity store".to_string())?;
    let memory = memory_layers()
        .lock()
        .map_err(|_| "failed to lock memory layers".to_string())?;
    Ok(build_context_bundle(&store, &memory, &input))
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_log::{EventImportance, GameEvent};
    use crate::models::{CultivationRealm, Element, Grade, SpiritualRoot};
    use crate::script::{InitialState, Location, ScriptType, WorldSetting};
    use tempfile::tempdir;

    fn create_test_script() -> Script {
        let mut world_setting = WorldSetting::new();
        world_setting.cultivation_realms = vec![
            CultivationRealm::new("Qi Condensation".to_string(), 1, 0, 1.0),
            CultivationRealm::new("Foundation Establishment".to_string(), 2, 0, 2.0),
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
                affinity: 0.9,
            elements: Vec::new(),
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
    fn test_command_logic_initialize_game() {
        let mut engine = GameEngine::new();
        let script = create_test_script();

        let result = engine.initialize_game(script);

        assert!(result.is_ok());
        let game_state = result.unwrap();
        assert_eq!(game_state.player.name, "Test Player");
        assert_eq!(game_state.player.location, "sect");
    }

    #[test]
    fn test_command_logic_get_game_state_before_initialization() {
        let engine = GameEngine::new();
        let result = engine.get_current_state();

        assert!(result.is_err());
    }

    #[test]
    fn test_error_response_conversion() {
        let error = anyhow::anyhow!("Test error");
        let response: ErrorResponse = error.into();

        assert_eq!(response.error, "Test error");
    }

    #[test]
    fn test_validate_slot_id_bounds() {
        assert!(validate_slot_id(1).is_ok());
        assert!(validate_slot_id(99).is_ok());
        assert!(validate_slot_id(0).is_err());
        assert!(validate_slot_id(120).is_err());
    }

    #[test]
    fn test_validate_file_path_extension() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("sample.json");
        std::fs::write(&file, "{}").unwrap();
        let ok = validate_file_path(file.to_str().unwrap(), &["json"]);
        assert!(ok.is_ok());
        let bad = validate_file_path(file.to_str().unwrap(), &["txt"]);
        assert!(bad.is_err());
    }

    #[test]
    fn test_validate_endpoint_scheme() {
        assert!(validate_endpoint("https://example.com").is_ok());
        assert!(validate_endpoint("http://localhost").is_ok());
        assert!(validate_endpoint("ftp://example.com").is_err());
        assert!(validate_endpoint("example.com").is_err());
    }

    #[test]
    fn test_validate_llm_config_requires_api_key_for_remote() {
        let input = LLMConfigInput {
            endpoint: "https://api.example.com/v1".to_string(),
            api_key: "".to_string(),
            model: "test".to_string(),
            max_tokens: 128,
            temperature: 0.7,
        };
        assert!(validate_llm_config_input(&input).is_err());
    }

    #[test]
    fn test_validate_llm_config_allows_local_without_key() {
        let input = LLMConfigInput {
            endpoint: "http://localhost:8000/v1".to_string(),
            api_key: "".to_string(),
            model: "test".to_string(),
            max_tokens: 128,
            temperature: 0.7,
        };
        assert!(validate_llm_config_input(&input).is_ok());
    }

    #[test]
    fn test_validate_output_path_extension() {
        let dir = tempdir().unwrap();
        let out = dir.path().join("novel.txt");
        let ok = validate_output_path(out.to_str().unwrap(), &["txt"]);
        assert!(ok.is_ok());
        let bad = dir.path().join("novel.md");
        let err = validate_output_path(bad.to_str().unwrap(), &["txt"]);
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn test_generate_novel_command_logic() {
        let events = vec![
            GameEvent {
                id: 1,
                timestamp: 1,
                event_type: std::sync::Arc::from("cultivation"),
                description: std::sync::Arc::from("Player cultivated"),
                importance: EventImportance::Normal,
            },
            GameEvent {
                id: 2,
                timestamp: 2,
                event_type: std::sync::Arc::from("combat"),
                description: std::sync::Arc::from("Player won duel"),
                importance: EventImportance::Important,
            },
        ];

        let novel = generate_novel_from_events("Test Novel", &events).await.unwrap();
        assert_eq!(novel.title, "Test Novel");
        assert_eq!(novel.total_events, 2);
        assert!(!novel.chapters.is_empty());
    }

    #[test]
    fn test_export_novel_command_logic() {
        let novel = Novel {
            title: "Exported Novel".to_string(),
            chapters: vec![crate::novel_generator::Chapter {
                index: 1,
                title: "Start".to_string(),
                content: "A new journey starts.".to_string(),
                source_event_ids: vec![1],
            }],
            toc: vec![crate::novel_generator::TocEntry {
                index: 1,
                title: "Start".to_string(),
                summary: "A new journey starts.".to_string(),
                source_event_count: 1,
            }],
            total_events: 1,
        };

        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("novel_out.txt");
        let result = export_novel_to_path(&novel, output.to_str().unwrap());
        assert!(result.is_ok());
        assert!(output.exists());
    }

    #[test]
    fn test_parse_generation_timing_sample() {
        let diag = "选项来源：rule_fallback；耗时(ms)：total=12,plot_gen=7,option_gen=3";
        let parsed = parse_generation_timing_sample(diag).unwrap();
        assert_eq!(
            parsed,
            GenerationTimingSample {
                total_ms: 12,
                plot_gen_ms: 7,
                option_gen_ms: 3,
            }
        );
    }

    #[test]
    fn test_summarize_generation_timing_diagnostics() {
        let input = vec![
            "耗时(ms)：total=10,plot_gen=6,option_gen=2".to_string(),
            "上下文注入：facts=2；耗时(ms)：total=14,plot_gen=8,option_gen=4".to_string(),
            "耗时(ms)：total=13,plot_gen=7,option_gen=3".to_string(),
            "invalid-line".to_string(),
        ];

        let summary = summarize_generation_timing_diagnostics(&input).unwrap();
        assert_eq!(summary.sample_count, 3);
        assert_eq!(summary.total_p50_ms, 13);
        assert_eq!(summary.total_p95_ms, 14);
        assert_eq!(summary.plot_gen_p95_ms, 8);
        assert_eq!(summary.option_gen_p95_ms, 4);
    }

    #[test]
    fn test_has_consistency_issue() {
        let mut report = ConsistencyReport::default();
        report.issues.push(crate::plot_consistency::ConsistencyIssue {
            level: crate::plot_consistency::IssueLevel::Warning,
            code: "chapter_goal_weak",
            message: "goal weak".to_string(),
        });
        assert!(has_consistency_issue(&report, "chapter_goal_weak"));
        assert!(!has_consistency_issue(&report, "duplicate_segment"));
    }

    #[test]
    fn test_chapter_goal_regeneration_hint_contains_goal_anchor() {
        let hint = chapter_goal_regeneration_hint(2);
        assert!(hint.contains("章节目标重生成约束"));
        assert!(hint.contains("资源变化"));
    }

    #[test]
    fn test_is_hollow_expression_detects_filler_text() {
        let text = "一时间气氛凝重，你不由得心中一凛，似乎四周都安静了下来。";
        assert!(is_hollow_expression(text));
    }

    #[test]
    fn test_is_hollow_expression_allows_dense_text() {
        let text = "山风掠过石阶，你踏前半步催动灵力，心神却在师门旧训与眼前杀机间迅速权衡。";
        assert!(!is_hollow_expression(text));
    }

    #[test]
    fn test_build_combat_explanation_contains_dominant_and_reversal() {
        let action_result = crate::numerical_system::ActionResult {
            success: false,
            description: "战斗受阻".to_string(),
            stat_changes: vec![],
            events: vec![],
        };
        let explanation = build_combat_explanation(
            &action_result,
            3,
            4200,
            Some("combat_power above maximum for realm 3"),
            Some("强攻压制（高压进攻意图）"),
        );
        assert!(explanation.summary.contains("主导因素="));
        assert!(explanation.summary.contains("反转因素="));
        assert!(explanation
            .reversal_factors
            .iter()
            .any(|item| item.contains("数值守门裁决")));
    }

    #[test]
    fn test_apply_combat_aftermath_updates_persistent_status() {
        let mut state = crate::game_state::GameState {
            script: crate::script::Script::new(
                "id".to_string(),
                "name".to_string(),
                crate::script::ScriptType::Custom,
                crate::script::WorldSetting::new(),
                crate::script::InitialState {
                    player_name: "p".to_string(),
                    player_spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    starting_location: "sect".to_string(),
                    starting_age: 16,
                },
            ),
            player: crate::game_state::Character::new(
                "player".to_string(),
                "Tester".to_string(),
                crate::models::CharacterStats {
                    spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    cultivation_realm: crate::models::CultivationRealm::new(
                        "Qi".to_string(),
                        1,
                        0,
                        1.0,
                    ),
                    techniques: vec![],
                    lifespan: crate::models::Lifespan {
                        current_age: 16,
                        max_age: 100,
                        realm_bonus: 0,
                    },
                    combat_power: 120,
                },
                "sect".to_string(),
            ),
            world_state: crate::game_state::WorldState::new(),
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
        };

        let success_summary =
            apply_combat_aftermath(&mut state, true, Some(CombatStrategy::Aggressive));
        assert!(success_summary.contains("战后状态更新"));
        assert_eq!(state.player.combat_status.reputation, 2);
        assert_eq!(state.player.combat_status.enmity, 2);

        let fail_summary =
            apply_combat_aftermath(&mut state, false, Some(CombatStrategy::Survival));
        assert!(fail_summary.contains("战后状态更新"));
        assert_eq!(state.player.combat_status.reputation, 1);
        assert_eq!(state.player.combat_status.enmity, 3);
        assert_eq!(state.player.combat_status.injury_level, 1);
    }

    #[test]
    fn test_push_growth_log_keeps_recent_entries() {
        let mut state = crate::game_state::GameState {
            script: crate::script::Script::new(
                "id".to_string(),
                "name".to_string(),
                crate::script::ScriptType::Custom,
                crate::script::WorldSetting::new(),
                crate::script::InitialState {
                    player_name: "p".to_string(),
                    player_spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    starting_location: "sect".to_string(),
                    starting_age: 16,
                },
            ),
            player: crate::game_state::Character::new(
                "player".to_string(),
                "Tester".to_string(),
                crate::models::CharacterStats {
                    spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    cultivation_realm: crate::models::CultivationRealm::new(
                        "Qi".to_string(),
                        1,
                        0,
                        1.0,
                    ),
                    techniques: vec![],
                    lifespan: crate::models::Lifespan {
                        current_age: 16,
                        max_age: 100,
                        realm_bonus: 0,
                    },
                    combat_power: 120,
                },
                "sect".to_string(),
            ),
            world_state: crate::game_state::WorldState::new(),
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
        };

        for i in 0..260 {
            push_growth_log(&mut state, format!("entry-{}", i));
        }
        assert_eq!(state.player.growth_log.len(), 240);
        assert_eq!(state.player.growth_log.first().map(String::as_str), Some("entry-20"));
        assert_eq!(state.player.growth_log.last().map(String::as_str), Some("entry-259"));
    }

    #[test]
    fn test_evaluate_technique_semantic_modifier_affinity_and_risk() {
        let stats = crate::models::CharacterStats {
            spiritual_root: crate::models::SpiritualRoot {
                element: crate::models::Element::Fire,
                elements: vec![crate::models::Element::Fire],
                grade: crate::models::Grade::Heavenly,
                affinity: 0.8,
            },
            cultivation_realm: crate::models::CultivationRealm::new("Qi".to_string(), 2, 0, 1.0),
            techniques: vec!["赤炎诀".to_string(), "禁术爆燃".to_string()],
            lifespan: crate::models::Lifespan {
                current_age: 18,
                max_age: 100,
                realm_bonus: 0,
            },
            combat_power: 200,
        };
        let (delta, reasons, risk) = evaluate_technique_semantic_modifier(&stats);
        assert!(delta > 0);
        assert!(!reasons.is_empty());
        assert!(risk);
    }

    #[test]
    fn test_is_high_risk_technique_name_detects_cn_and_en_keywords() {
        assert!(is_high_risk_technique_name("禁术爆燃"));
        assert!(is_high_risk_technique_name("forbidden flame"));
        assert!(!is_high_risk_technique_name("清心诀"));
    }

    #[test]
    fn test_apply_breakthrough_failure_consequences_updates_qi_deviation_and_injury() {
        let mut state = crate::game_state::GameState {
            script: crate::script::Script::new(
                "id".to_string(),
                "name".to_string(),
                crate::script::ScriptType::Custom,
                crate::script::WorldSetting::new(),
                crate::script::InitialState {
                    player_name: "p".to_string(),
                    player_spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    starting_location: "sect".to_string(),
                    starting_age: 16,
                },
            ),
            player: crate::game_state::Character::new(
                "player".to_string(),
                "Tester".to_string(),
                crate::models::CharacterStats {
                    spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    cultivation_realm: crate::models::CultivationRealm::new(
                        "Qi".to_string(),
                        1,
                        2,
                        1.0,
                    ),
                    techniques: vec!["禁术爆燃".to_string()],
                    lifespan: crate::models::Lifespan {
                        current_age: 16,
                        max_age: 100,
                        realm_bonus: 0,
                    },
                    combat_power: 120,
                },
                "sect".to_string(),
            ),
            world_state: crate::game_state::WorldState::new(),
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
        };
        let mut action_result = crate::numerical_system::ActionResult {
            success: false,
            description: "突破失败".to_string(),
            stat_changes: vec![],
            events: vec![],
        };

        apply_breakthrough_failure_consequences(&mut state, &mut action_result, true);

        assert_eq!(state.player.combat_status.qi_deviation, 2);
        assert_eq!(state.player.combat_status.injury_level, 1);
        assert!(action_result
            .events
            .iter()
            .any(|e| e.contains("突破失败后果：气机紊乱+2")));
        assert!(action_result
            .events
            .iter()
            .any(|e| e.contains("走火入魔征兆")));
        assert!(state
            .player
            .growth_log
            .iter()
            .any(|e| e.contains("突破受挫：气机紊乱")));
    }

    #[test]
    fn test_breakthrough_blocked_by_qi_deviation_threshold() {
        assert!(!breakthrough_blocked_by_qi_deviation(7));
        assert!(breakthrough_blocked_by_qi_deviation(8));
        assert!(breakthrough_blocked_by_qi_deviation(10));
    }

    #[test]
    fn test_apply_travel_and_encounter_moves_location_and_advances_time() {
        let mut world_setting = crate::script::WorldSetting::new();
        world_setting.locations = vec![
            crate::script::Location {
                id: "sect".to_string(),
                name: "青云宗".to_string(),
                description: "宗门驻地".to_string(),
                spiritual_energy: 0.3,
            },
            crate::script::Location {
                id: "valley".to_string(),
                name: "幽风谷".to_string(),
                description: "灵压紊乱".to_string(),
                spiritual_energy: 0.3,
            },
        ];
        let script = crate::script::Script::new(
            "id".to_string(),
            "name".to_string(),
            crate::script::ScriptType::Custom,
            world_setting.clone(),
            crate::script::InitialState {
                player_name: "p".to_string(),
                player_spiritual_root: crate::models::SpiritualRoot {
                    element: crate::models::Element::Fire,
                    elements: vec![crate::models::Element::Fire],
                    grade: crate::models::Grade::Heavenly,
                    affinity: 0.8,
                },
                starting_location: "sect".to_string(),
                starting_age: 16,
            },
        );
        let mut state = crate::game_state::GameState {
            player: crate::game_state::Character::new(
                "player".to_string(),
                "Tester".to_string(),
                crate::models::CharacterStats {
                    spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    cultivation_realm: crate::models::CultivationRealm::new(
                        "Qi".to_string(),
                        1,
                        0,
                        1.0,
                    ),
                    techniques: vec![],
                    lifespan: crate::models::Lifespan {
                        current_age: 16,
                        max_age: 100,
                        realm_bonus: 0,
                    },
                    combat_power: 120,
                },
                "sect".to_string(),
            ),
            world_state: crate::game_state::WorldState::from_script(&script),
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
            script,
        };
        let mut plot_state = crate::plot_engine::PlotState::new(crate::plot_engine::Scene::new(
            "s1".to_string(),
            "scene".to_string(),
            "desc".to_string(),
            "sect".to_string(),
        ));

        let result = apply_travel_and_encounter(&mut state, &mut plot_state, "valley").unwrap();

        assert_eq!(state.player.location, "valley");
        assert_eq!(plot_state.current_scene.location, "valley");
        assert_eq!(state.game_time.total_days, 2);
        assert!(result.0.contains("前往"));
        assert!(state
            .player
            .growth_log
            .iter()
            .any(|entry| entry.contains("行程变更")));
    }

    #[test]
    fn test_apply_travel_and_encounter_rejects_unknown_location() {
        let script = create_test_script();
        let mut state = crate::game_state::GameState {
            player: crate::game_state::Character::new(
                "player".to_string(),
                "Tester".to_string(),
                crate::models::CharacterStats {
                    spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    cultivation_realm: crate::models::CultivationRealm::new(
                        "Qi".to_string(),
                        1,
                        0,
                        1.0,
                    ),
                    techniques: vec![],
                    lifespan: crate::models::Lifespan {
                        current_age: 16,
                        max_age: 100,
                        realm_bonus: 0,
                    },
                    combat_power: 120,
                },
                "sect".to_string(),
            ),
            world_state: crate::game_state::WorldState::from_script(&script),
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
            script,
        };
        let mut plot_state = crate::plot_engine::PlotState::new(crate::plot_engine::Scene::new(
            "s1".to_string(),
            "scene".to_string(),
            "desc".to_string(),
            "sect".to_string(),
        ));

        let err = apply_travel_and_encounter(&mut state, &mut plot_state, "unknown")
            .expect_err("unknown location should be rejected");
        assert!(err.contains("目标地点不存在"));
    }

    #[test]
    fn test_apply_travel_and_encounter_rejects_unreachable_location() {
        let mut world_setting = crate::script::WorldSetting::new();
        world_setting.locations = vec![
            crate::script::Location {
                id: "sect".to_string(),
                name: "青云宗".to_string(),
                description: "宗门驻地".to_string(),
                spiritual_energy: 0.2,
            },
            crate::script::Location {
                id: "abyss".to_string(),
                name: "魔渊".to_string(),
                description: "极高灵压".to_string(),
                spiritual_energy: 1.5,
            },
        ];
        let script = crate::script::Script::new(
            "id".to_string(),
            "name".to_string(),
            crate::script::ScriptType::Custom,
            world_setting.clone(),
            crate::script::InitialState {
                player_name: "p".to_string(),
                player_spiritual_root: crate::models::SpiritualRoot {
                    element: crate::models::Element::Fire,
                    elements: vec![crate::models::Element::Fire],
                    grade: crate::models::Grade::Triple,
                    affinity: 0.3,
                },
                starting_location: "sect".to_string(),
                starting_age: 16,
            },
        );

        let mut state = crate::game_state::GameState {
            player: crate::game_state::Character::new(
                "player".to_string(),
                "Tester".to_string(),
                crate::models::CharacterStats {
                    spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Triple,
                        affinity: 0.3,
                    },
                    cultivation_realm: crate::models::CultivationRealm::new(
                        "Qi".to_string(),
                        1,
                        0,
                        1.0,
                    ),
                    techniques: vec![],
                    lifespan: crate::models::Lifespan {
                        current_age: 16,
                        max_age: 100,
                        realm_bonus: 0,
                    },
                    combat_power: 80,
                },
                "sect".to_string(),
            ),
            world_state: crate::game_state::WorldState::from_script(&script),
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
            script,
        };
        state.player.combat_status.injury_level = 8;
        state.player.combat_status.qi_deviation = 8;
        let mut plot_state = crate::plot_engine::PlotState::new(crate::plot_engine::Scene::new(
            "s1".to_string(),
            "scene".to_string(),
            "desc".to_string(),
            "sect".to_string(),
        ));

        let err = apply_travel_and_encounter(&mut state, &mut plot_state, "abyss")
            .expect_err("unreachable location should be rejected");
        assert!(err.contains("无法前往"));
    }

    #[test]
    fn test_cultivation_gain_multiplier_from_location_is_monotonic() {
        let low = cultivation_gain_multiplier_from_location(Some(0.1));
        let mid = cultivation_gain_multiplier_from_location(Some(0.5));
        let high = cultivation_gain_multiplier_from_location(Some(1.0));
        assert!(low <= mid);
        assert!(mid <= high);
        assert!(low >= 0.6 && high <= 1.6);
    }

    #[test]
    fn test_location_spiritual_energy_reads_current_node() {
        let mut world_setting = crate::script::WorldSetting::new();
        world_setting.locations = vec![crate::script::Location {
            id: "cave".to_string(),
            name: "灵息洞".to_string(),
            description: "灵气汇聚".to_string(),
            spiritual_energy: 0.95,
        }];
        let script = crate::script::Script::new(
            "id".to_string(),
            "name".to_string(),
            crate::script::ScriptType::Custom,
            world_setting.clone(),
            crate::script::InitialState {
                player_name: "p".to_string(),
                player_spiritual_root: crate::models::SpiritualRoot {
                    element: crate::models::Element::Fire,
                    elements: vec![crate::models::Element::Fire],
                    grade: crate::models::Grade::Heavenly,
                    affinity: 0.8,
                },
                starting_location: "cave".to_string(),
                starting_age: 16,
            },
        );
        let state = crate::game_state::GameState {
            player: crate::game_state::Character::new(
                "player".to_string(),
                "Tester".to_string(),
                crate::models::CharacterStats {
                    spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    cultivation_realm: crate::models::CultivationRealm::new(
                        "Qi".to_string(),
                        1,
                        0,
                        1.0,
                    ),
                    techniques: vec![],
                    lifespan: crate::models::Lifespan {
                        current_age: 16,
                        max_age: 100,
                        realm_bonus: 0,
                    },
                    combat_power: 120,
                },
                "cave".to_string(),
            ),
            world_state: crate::game_state::WorldState::from_script(&script),
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
            script,
        };
        assert_eq!(location_spiritual_energy(&state), Some(0.95));
    }

    #[test]
    fn test_select_encounter_text_uses_risk_buckets() {
        let low = select_encounter_text(0.2, 1);
        let mid = select_encounter_text(0.6, 1);
        let high = select_encounter_text(0.95, 1);
        assert!(low.contains("散修") || low.contains("灵兽") || low.contains("路匪"));
        assert!(mid.contains("阵法") || mid.contains("巡逻") || mid.contains("乱流"));
        assert!(high.contains("魔息") || high.contains("妖潮") || high.contains("威压"));
    }

    #[test]
    fn test_choose_combat_strategy_prefers_survival_when_status_is_bad() {
        let status = crate::game_state::CombatAftermathStatus {
            injury_level: 7,
            reputation: 0,
            enmity: 0,
            qi_deviation: 6,
        };
        let (strategy, reason) = choose_combat_strategy(&status, "强攻压制");
        assert_eq!(strategy, CombatStrategy::Survival);
        assert!(reason.contains("伤势/气机"));
    }

    #[test]
    fn test_strategy_power_modifier_pct_matches_expected_bias() {
        let status = crate::game_state::CombatAftermathStatus {
            injury_level: 1,
            reputation: 0,
            enmity: 2,
            qi_deviation: 1,
        };
        let aggressive = strategy_power_modifier_pct(CombatStrategy::Aggressive, &status);
        let survival = strategy_power_modifier_pct(CombatStrategy::Survival, &status);
        assert!(aggressive > 0);
        assert!(survival < 0);
    }

    #[test]
    fn test_apply_combat_aftermath_updates_social_profile() {
        let script = create_test_script();
        let mut state = crate::game_state::GameState {
            player: crate::game_state::Character::new(
                "player".to_string(),
                "Tester".to_string(),
                crate::models::CharacterStats {
                    spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    cultivation_realm: crate::models::CultivationRealm::new(
                        "Qi".to_string(),
                        1,
                        0,
                        1.0,
                    ),
                    techniques: vec![],
                    lifespan: crate::models::Lifespan {
                        current_age: 16,
                        max_age: 100,
                        realm_bonus: 0,
                    },
                    combat_power: 120,
                },
                "sect".to_string(),
            ),
            world_state: crate::game_state::WorldState::from_script(&script),
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
            script,
        };
        state.player.social_profile.sect_affinity = 10;
        state.player.social_profile.vendetta = 4;

        let _ = apply_combat_aftermath(&mut state, true, Some(CombatStrategy::Cautious));
        assert!(state.player.social_profile.favor >= 1);
        assert!(state.player.social_profile.mentor_bond >= 1);
        assert!(state.player.social_profile.vendetta <= 4);
        assert!(
            state.player.social_profile.camp_stance == "righteous"
                || state.player.social_profile.camp_stance == "neutral"
        );
    }

    #[test]
    fn test_evaluate_environment_combat_modifier_high_energy_bonus() {
        let mut world_setting = crate::script::WorldSetting::new();
        world_setting.locations = vec![crate::script::Location {
            id: "peak".to_string(),
            name: "灵峰".to_string(),
            description: "灵气充沛".to_string(),
            spiritual_energy: 0.9,
        }];
        let script = crate::script::Script::new(
            "id".to_string(),
            "name".to_string(),
            crate::script::ScriptType::Custom,
            world_setting.clone(),
            crate::script::InitialState {
                player_name: "p".to_string(),
                player_spiritual_root: crate::models::SpiritualRoot {
                    element: crate::models::Element::Fire,
                    elements: vec![crate::models::Element::Fire],
                    grade: crate::models::Grade::Heavenly,
                    affinity: 0.8,
                },
                starting_location: "peak".to_string(),
                starting_age: 16,
            },
        );
        let state = crate::game_state::GameState {
            player: crate::game_state::Character::new(
                "player".to_string(),
                "Tester".to_string(),
                crate::models::CharacterStats {
                    spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    cultivation_realm: crate::models::CultivationRealm::new(
                        "Qi".to_string(),
                        1,
                        0,
                        1.0,
                    ),
                    techniques: vec![],
                    lifespan: crate::models::Lifespan {
                        current_age: 16,
                        max_age: 100,
                        realm_bonus: 0,
                    },
                    combat_power: 120,
                },
                "peak".to_string(),
            ),
            world_state: crate::game_state::WorldState::from_script(&script),
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
            script,
        };
        let (delta, reason) = evaluate_environment_combat_modifier(&state);
        assert!(delta > 0);
        assert!(reason.contains("高灵气场域增幅"));
    }

    #[test]
    fn test_compute_map_overview_contains_reachability() {
        let mut world_setting = crate::script::WorldSetting::new();
        world_setting.locations = vec![
            crate::script::Location {
                id: "sect".to_string(),
                name: "青云宗".to_string(),
                description: "宗门驻地".to_string(),
                spiritual_energy: 0.3,
            },
            crate::script::Location {
                id: "market".to_string(),
                name: "坊市".to_string(),
                description: "交易区域".to_string(),
                spiritual_energy: 0.35,
            },
        ];
        let script = crate::script::Script::new(
            "id".to_string(),
            "name".to_string(),
            crate::script::ScriptType::Custom,
            world_setting.clone(),
            crate::script::InitialState {
                player_name: "p".to_string(),
                player_spiritual_root: crate::models::SpiritualRoot {
                    element: crate::models::Element::Fire,
                    elements: vec![crate::models::Element::Fire],
                    grade: crate::models::Grade::Heavenly,
                    affinity: 0.8,
                },
                starting_location: "sect".to_string(),
                starting_age: 16,
            },
        );
        let state = crate::game_state::GameState {
            player: crate::game_state::Character::new(
                "player".to_string(),
                "Tester".to_string(),
                crate::models::CharacterStats {
                    spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    cultivation_realm: crate::models::CultivationRealm::new(
                        "Qi".to_string(),
                        1,
                        0,
                        1.0,
                    ),
                    techniques: vec![],
                    lifespan: crate::models::Lifespan {
                        current_age: 16,
                        max_age: 100,
                        realm_bonus: 0,
                    },
                    combat_power: 120,
                },
                "sect".to_string(),
            ),
            world_state: crate::game_state::WorldState::from_script(&script),
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
            script,
        };
        let overview = compute_map_overview(&state);
        assert_eq!(overview.len(), 2);
        assert!(overview.iter().any(|node| node.location_id == "sect" && node.reachable));
    }

    #[test]
    fn test_evaluate_style_counter_modifier_detects_counter_bonus() {
        let stats = crate::models::CharacterStats {
            spiritual_root: crate::models::SpiritualRoot {
                element: crate::models::Element::Fire,
                elements: vec![crate::models::Element::Fire],
                grade: crate::models::Grade::Heavenly,
                affinity: 0.8,
            },
            cultivation_realm: crate::models::CultivationRealm::new("Qi".to_string(), 2, 0, 1.0),
            techniques: vec!["青霜剑诀".to_string()],
            lifespan: crate::models::Lifespan {
                current_age: 18,
                max_age: 100,
                realm_bonus: 0,
            },
            combat_power: 220,
        };
        let (delta, reason) = evaluate_style_counter_modifier(&stats, "敌方体修强攻");
        assert!(delta > 0);
        assert!(reason.contains("克制"));
    }

    #[test]
    fn test_evaluate_style_counter_modifier_handles_unknown_enemy_style() {
        let stats = crate::models::CharacterStats {
            spiritual_root: crate::models::SpiritualRoot {
                element: crate::models::Element::Fire,
                elements: vec![crate::models::Element::Fire],
                grade: crate::models::Grade::Heavenly,
                affinity: 0.8,
            },
            cultivation_realm: crate::models::CultivationRealm::new("Qi".to_string(), 2, 0, 1.0),
            techniques: vec!["青霜剑诀".to_string()],
            lifespan: crate::models::Lifespan {
                current_age: 18,
                max_age: 100,
                realm_bonus: 0,
            },
            combat_power: 220,
        };
        let (delta, reason) = evaluate_style_counter_modifier(&stats, "敌方神秘修士");
        assert_eq!(delta, 0);
        assert!(reason.contains("未识别"));
    }

    #[test]
    fn test_extract_styles_from_text_extracts_styles() {
        assert!(extract_styles_from_text("我以剑诀起手").contains(&"sword"));
        assert!(extract_styles_from_text("贴身体修对轰").contains(&"body"));
        assert!(extract_styles_from_text("普通描述").is_empty());
    }

    #[test]
    fn test_evaluate_style_counter_modifier_uses_hint_when_technique_empty() {
        let stats = crate::models::CharacterStats {
            spiritual_root: crate::models::SpiritualRoot {
                element: crate::models::Element::Fire,
                elements: vec![crate::models::Element::Fire],
                grade: crate::models::Grade::Heavenly,
                affinity: 0.8,
            },
            cultivation_realm: crate::models::CultivationRealm::new("Qi".to_string(), 2, 0, 1.0),
            techniques: vec![],
            lifespan: crate::models::Lifespan {
                current_age: 18,
                max_age: 100,
                realm_bonus: 0,
            },
            combat_power: 220,
        };
        let (delta, reason) = evaluate_style_counter_modifier(&stats, "我以剑诀迎战体修强攻");
        assert!(delta > 0);
        assert!(reason.contains("克制"));
    }
}


