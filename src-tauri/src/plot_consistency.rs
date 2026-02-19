use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::numerical_system::ActionResult;
use crate::plot_engine::{PlotState, PlotUpdate};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueLevel {
    Warning,
    Critical,
}

#[derive(Debug, Clone)]
pub struct ConsistencyIssue {
    pub level: IssueLevel,
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct ConsistencyReport {
    pub issues: Vec<ConsistencyIssue>,
    pub repaired_plot_text: Option<String>,
    pub force_non_waiting: bool,
    pub override_location: Option<String>,
    pub override_chapter_summary: Option<String>,
}

impl ConsistencyReport {
    pub fn risk_score(&self) -> i32 {
        if self.issues.is_empty() {
            return 0;
        }
        let policy = current_policy_snapshot();
        self.issues
            .iter()
            .map(|i| policy.weight_for(i.code, &i.level))
            .sum()
    }

    pub fn to_diagnostics(&self) -> Option<String> {
        if self.issues.is_empty() {
            return None;
        }
        let risk_score = self.risk_score();
        let body = self
            .issues
            .iter()
            .map(|i| format!("[{}:{}] {}", level_tag(&i.level), i.code, i.message))
            .collect::<Vec<_>>()
            .join("；");
        Some(format!("一致性校验器V2(风险分={}): {}", risk_score, body))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyPolicy {
    pub recent_window: usize,
    pub cross_chapter_window: usize,
    pub duplicate_recent_threshold: f32,
    pub duplicate_cross_chapter_threshold: f32,
    pub weight_warning: i32,
    pub weight_critical: i32,
    pub code_weights: HashMap<String, i32>,
}

impl Default for ConsistencyPolicy {
    fn default() -> Self {
        let mut code_weights = HashMap::new();
        code_weights.insert("duplicate_segment".to_string(), 8);
        code_weights.insert("duplicate_cross_chapter".to_string(), 10);
        code_weights.insert("waiting_without_options".to_string(), 12);
        code_weights.insert("realm_power_conflict".to_string(), 9);
        code_weights.insert("title_drift".to_string(), 6);
        code_weights.insert("location_transition_untracked".to_string(), 6);
        code_weights.insert("chapter_goal_weak".to_string(), 5);
        code_weights.insert("chapter_summary_missing".to_string(), 4);
        code_weights.insert("empty_plot_text".to_string(), 15);
        Self {
            recent_window: 3,
            cross_chapter_window: 3,
            duplicate_recent_threshold: 0.92,
            duplicate_cross_chapter_threshold: 0.88,
            weight_warning: 5,
            weight_critical: 12,
            code_weights,
        }
    }
}

impl ConsistencyPolicy {
    fn weight_for(&self, code: &str, level: &IssueLevel) -> i32 {
        self.code_weights
            .get(code)
            .copied()
            .unwrap_or(match level {
                IssueLevel::Warning => self.weight_warning,
                IssueLevel::Critical => self.weight_critical,
            })
    }
}

fn load_policy() -> &'static ConsistencyPolicy {
    static POLICY: OnceLock<ConsistencyPolicy> = OnceLock::new();
    POLICY.get_or_init(load_policy_from_disk)
}

fn runtime_policy() -> &'static Mutex<ConsistencyPolicy> {
    static RUNTIME_POLICY: OnceLock<Mutex<ConsistencyPolicy>> = OnceLock::new();
    RUNTIME_POLICY.get_or_init(|| Mutex::new(load_policy().clone()))
}

fn load_policy_from_disk() -> ConsistencyPolicy {
    let path = std::path::Path::new("config/consistency_rules_v2.json");
    if let Ok(raw) = std::fs::read_to_string(path) {
        if let Ok(cfg) = serde_json::from_str::<ConsistencyPolicy>(&raw) {
            return cfg;
        }
    }
    ConsistencyPolicy::default()
}

fn persist_policy(policy: &ConsistencyPolicy) -> Result<(), String> {
    let path = std::path::Path::new("config/consistency_rules_v2.json");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {}", e))?;
    }
    let data =
        serde_json::to_string_pretty(policy).map_err(|e| format!("序列化策略失败: {}", e))?;
    std::fs::write(path, data).map_err(|e| format!("写入策略文件失败: {}", e))
}

pub fn get_runtime_policy() -> ConsistencyPolicy {
    match runtime_policy().lock() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    }
}

pub fn update_runtime_policy(next: ConsistencyPolicy) -> Result<ConsistencyPolicy, String> {
    let valid_recent = (0.5..=0.999).contains(&next.duplicate_recent_threshold);
    let valid_cross = (0.5..=0.999).contains(&next.duplicate_cross_chapter_threshold);
    if next.recent_window == 0 || next.cross_chapter_window == 0 || !valid_recent || !valid_cross {
        return Err("一致性策略参数非法".to_string());
    }

    {
        let mut guard = match runtime_policy().lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        *guard = next.clone();
    }

    persist_policy(&next)?;
    Ok(next)
}

pub fn reset_runtime_policy() -> Result<ConsistencyPolicy, String> {
    let default = ConsistencyPolicy::default();
    {
        let mut guard = match runtime_policy().lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        *guard = default.clone();
    }
    persist_policy(&default)?;
    Ok(default)
}

fn current_policy_snapshot() -> ConsistencyPolicy {
    get_runtime_policy()
}

fn level_tag(level: &IssueLevel) -> &'static str {
    match level {
        IssueLevel::Warning => "warn",
        IssueLevel::Critical => "critical",
    }
}

fn normalize_text(raw: &str) -> String {
    raw.chars()
        .filter(|ch| !ch.is_whitespace())
        .map(|ch| ch.to_ascii_lowercase())
        .collect::<String>()
}

fn char_bigrams(text: &str) -> Vec<String> {
    let chars = text.chars().collect::<Vec<_>>();
    if chars.len() < 2 {
        return Vec::new();
    }
    chars
        .windows(2)
        .map(|pair| pair.iter().collect::<String>())
        .collect::<Vec<_>>()
}

fn jaccard_similarity(a: &str, b: &str) -> f32 {
    use std::collections::HashSet;
    let set_a = char_bigrams(a).into_iter().collect::<HashSet<_>>();
    let set_b = char_bigrams(b).into_iter().collect::<HashSet<_>>();
    if set_a.is_empty() && set_b.is_empty() {
        return 1.0;
    }
    let inter = set_a.intersection(&set_b).count() as f32;
    let union = set_a.union(&set_b).count() as f32;
    if union == 0.0 { 0.0 } else { inter / union }
}

fn build_repair_text(plot_state: &PlotState, action_result: &ActionResult) -> String {
    let action = action_result.description.trim();
    let action_line = if action.is_empty() {
        "你调整呼吸，重新观察眼前局势。".to_string()
    } else {
        format!("你上一刻的行动结果是：{}。", action)
    };
    format!(
        "{}\n\n{}的气氛悄然变化，新的线索浮现，局势不再停留在原地。",
        action_line, plot_state.current_scene.location
    )
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| text.contains(n))
}

fn detect_realm_power_conflict(
    text: &str,
    player_realm_level: u32,
    player_combat_power: u64,
) -> bool {
    let low_tier_targets = ["练气", "qi condensation", "low-tier"];
    let hostility_words = ["妖兽", "monster", "beast"];
    let fear_words = ["不敌", "害怕", "畏惧", "落荒而逃", "毫无还手", "flee", "panic"];
    let base_hit = contains_any(text, &low_tier_targets)
        && contains_any(text, &hostility_words)
        && contains_any(text, &fear_words);
    if !base_hit {
        return false;
    }
    player_realm_level >= 3 || player_combat_power >= 3000
}

fn detect_title_drift(text: &str, player_name: &str) -> bool {
    let name = player_name.trim();
    if name.is_empty() {
        return false;
    }
    let has_name = text.contains(name);
    let has_second_person = text.contains("你");
    let has_third_person = contains_any(text, &["他", "她", "此人", "那名修士"]);
    has_name && has_third_person && !has_second_person
}

fn extract_location_transition(text: &str) -> Option<String> {
    let markers = ["来到", "抵达", "前往", "赶到", "进入"];
    let punct = ['，', '。', '！', '？', ',', '.', '!', '?', '\n'];
    for marker in markers {
        let Some(start) = text.find(marker) else {
            continue;
        };
        let rest = text[start + marker.len()..].trim_start();
        if rest.is_empty() {
            continue;
        }
        let end = rest
            .char_indices()
            .find_map(|(idx, ch)| punct.contains(&ch).then_some(idx))
            .unwrap_or(rest.len());
        let candidate = rest[..end].trim().trim_matches('\"').trim_matches('“').trim_matches('”');
        if candidate.chars().count() >= 2 && candidate.chars().count() <= 12 {
            return Some(candidate.to_string());
        }
    }
    None
}

fn needs_chapter_goal_hint(text: &str, interaction_count: u8, chapter_end: bool) -> bool {
    if chapter_end || interaction_count < 1 {
        return false;
    }
    let goal_words = ["目标", "线索", "计划", "决定", "下一步", "调查", "突破", "冲突", "抉择"];
    !contains_any(text, &goal_words)
}

fn fallback_summary(text: &str) -> Option<String> {
    let cleaned = text.trim();
    if cleaned.is_empty() {
        return None;
    }
    let mut out = String::new();
    for ch in cleaned.chars().take(48) {
        out.push(ch);
    }
    if cleaned.chars().count() > 48 {
        out.push('…');
    }
    Some(format!("本章小结：{}", out))
}

pub fn validate_and_repair_plot_update(
    plot_state: &PlotState,
    plot_update: &PlotUpdate,
    action_result: &ActionResult,
    player_realm_level: u32,
    player_combat_power: u64,
    player_name: &str,
) -> ConsistencyReport {
    let policy = current_policy_snapshot();
    let mut report = ConsistencyReport::default();
    let current_text = plot_update.plot_text.trim();

    if current_text.is_empty() {
        report.issues.push(ConsistencyIssue {
            level: IssueLevel::Critical,
            code: "empty_plot_text",
            message: "生成段落为空，已回退为规则续写".to_string(),
        });
        report.repaired_plot_text = Some(build_repair_text(plot_state, action_result));
    }

    let normalized_current = normalize_text(current_text);
    if !normalized_current.is_empty() {
        let duplicate = plot_state
            .current_chapter
            .content
            .iter()
            .rev()
            .take(policy.recent_window)
            .any(|old| {
                let normalized_old = normalize_text(old);
                if normalized_old.is_empty() {
                    return false;
                }
                normalized_old == normalized_current
                    || jaccard_similarity(&normalized_old, &normalized_current)
                        >= policy.duplicate_recent_threshold
            });
        if duplicate {
            report.issues.push(ConsistencyIssue {
                level: IssueLevel::Warning,
                code: "duplicate_segment",
                message: "检测到与近期剧情高度重复，已替换为去重段落".to_string(),
            });
            report.repaired_plot_text = Some(build_repair_text(plot_state, action_result));
        }

        let cross_dup = plot_state
            .chapters
            .iter()
            .rev()
            .take(policy.cross_chapter_window)
            .flat_map(|chapter| chapter.content.iter())
            .any(|old| {
                let normalized_old = normalize_text(old);
                if normalized_old.is_empty() {
                    return false;
                }
                jaccard_similarity(&normalized_old, &normalized_current)
                    >= policy.duplicate_cross_chapter_threshold
            });
        if cross_dup {
            report.issues.push(ConsistencyIssue {
                level: IssueLevel::Warning,
                code: "duplicate_cross_chapter",
                message: "检测到与前序章节重复，已注入差异化推进锚点".to_string(),
            });
            let bridge = "你意识到此事与过往经历相似，但仍有关键差异：新的动机与代价正在浮现。";
            let merged = if let Some(existing) = report.repaired_plot_text.clone() {
                format!("{}\n\n{}", existing.trim(), bridge)
            } else {
                format!("{}\n\n{}", current_text, bridge)
            };
            report.repaired_plot_text = Some(merged);
        }
    }

    if plot_update.is_waiting_for_input
        && !plot_update.chapter_end
        && plot_update.available_options.is_empty()
    {
        report.issues.push(ConsistencyIssue {
            level: IssueLevel::Warning,
            code: "waiting_without_options",
            message: "等待玩家输入但没有选项，已切换为自动推进".to_string(),
        });
        report.force_non_waiting = true;
    }

    if detect_realm_power_conflict(current_text, player_realm_level, player_combat_power) {
        report.issues.push(ConsistencyIssue {
            level: IssueLevel::Warning,
            code: "realm_power_conflict",
            message: "高境界角色对低阶威胁表现失衡，已注入合理化修正".to_string(),
        });
        let fix_line = "你稳住心神，以当前境界与功法判断，此等低阶威胁不足以令你失措，战术重心转为试探与压制。";
        let merged = if let Some(existing) = report.repaired_plot_text.clone() {
            format!("{}\n\n{}", existing.trim(), fix_line)
        } else if current_text.is_empty() {
            fix_line.to_string()
        } else {
            format!("{}\n\n{}", current_text, fix_line)
        };
        report.repaired_plot_text = Some(merged);
    }

    if detect_title_drift(current_text, player_name) {
        report.issues.push(ConsistencyIssue {
            level: IssueLevel::Warning,
            code: "title_drift",
            message: "叙事称谓从玩家视角漂移，已注入称谓修正".to_string(),
        });
        let fix_line = format!("叙事视角重新锚定为“你（{}）”，你的判断与行动将继续主导局势。", player_name.trim());
        let merged = if let Some(existing) = report.repaired_plot_text.clone() {
            format!("{}\n\n{}", existing.trim(), fix_line)
        } else if current_text.is_empty() {
            fix_line
        } else {
            format!("{}\n\n{}", current_text, fix_line)
        };
        report.repaired_plot_text = Some(merged);
    }

    if let Some(next_location) = extract_location_transition(current_text) {
        let now = plot_state.current_scene.location.trim();
        if !now.is_empty() && next_location != now {
            report.issues.push(ConsistencyIssue {
                level: IssueLevel::Warning,
                code: "location_transition_untracked",
                message: format!("检测到地点切换（{} -> {}），已同步场景位置", now, next_location),
            });
            report.override_location = Some(next_location.clone());
            let bridge = format!("你已从{}转入{}，场景状态随之更新。", now, next_location);
            let merged = if let Some(existing) = report.repaired_plot_text.clone() {
                format!("{}\n\n{}", existing.trim(), bridge)
            } else if current_text.is_empty() {
                bridge
            } else {
                format!("{}\n\n{}", current_text, bridge)
            };
            report.repaired_plot_text = Some(merged);
        }
    }

    if needs_chapter_goal_hint(
        current_text,
        plot_state.current_chapter.interaction_count,
        plot_update.chapter_end,
    ) {
        report.issues.push(ConsistencyIssue {
            level: IssueLevel::Warning,
            code: "chapter_goal_weak",
            message: "章节目标不够清晰，已注入目标锚点".to_string(),
        });
        let hint = "本章目标已明确：围绕当前冲突提取关键线索，完成一次可验证的推进。";
        let merged = if let Some(existing) = report.repaired_plot_text.clone() {
            format!("{}\n\n{}", existing.trim(), hint)
        } else if current_text.is_empty() {
            hint.to_string()
        } else {
            format!("{}\n\n{}", current_text, hint)
        };
        report.repaired_plot_text = Some(merged);
    }

    if plot_update.chapter_end
        && plot_update
            .chapter_summary
            .as_ref()
            .map(|s| s.trim().is_empty())
            .unwrap_or(true)
    {
        if let Some(summary) = fallback_summary(
            report
                .repaired_plot_text
                .as_deref()
                .unwrap_or(current_text),
        ) {
            report.issues.push(ConsistencyIssue {
                level: IssueLevel::Warning,
                code: "chapter_summary_missing",
                message: "章节结束但缺少摘要，已生成回退摘要".to_string(),
            });
            report.override_chapter_summary = Some(summary);
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::numerical_system::ActionResult;
    use crate::plot_engine::{ChapterState, PlotInteractionState, PlotSettings, Scene};

    fn test_plot_state() -> PlotState {
        PlotState {
            current_scene: Scene {
                id: "s1".to_string(),
                name: "scene".to_string(),
                description: "desc".to_string(),
                location: "青云山".to_string(),
                available_options: vec![],
            },
            plot_history: vec![],
            is_waiting_for_input: true,
            interaction_state: PlotInteractionState::WaitingForChoice,
            last_action_result: None,
            settings: PlotSettings::default(),
            current_chapter: ChapterState {
                index: 1,
                title: "第一章".to_string(),
                content: vec!["你推开山门，风声呼啸。".to_string()],
                summary: String::new(),
                interaction_count: 0,
                status: Default::default(),
            },
            chapters: vec![],
            segment_count: 1,
            last_generation_diagnostics: None,
            last_option_generation_source: None,
            last_consistency_risk_score: None,
        }
    }

    fn action_result() -> ActionResult {
        ActionResult {
            success: true,
            description: "你向前侦查".to_string(),
            stat_changes: vec![],
            events: vec![],
        }
    }

    #[test]
    fn repairs_empty_plot_text() {
        let state = test_plot_state();
        let update = PlotUpdate {
            new_scene: None,
            plot_text: "  ".to_string(),
            triggered_events: vec![],
            state_changes: vec![],
            is_waiting_for_input: false,
            available_options: vec![],
            chapter_title: None,
            chapter_summary: None,
            chapter_end: false,
            generation_diagnostics: None,
        };
        let report = validate_and_repair_plot_update(&state, &update, &action_result(), 1, 100, "无名弟子");
        assert!(report.repaired_plot_text.is_some());
        assert!(!report.issues.is_empty());
    }

    #[test]
    fn detects_duplicate_segment() {
        let state = test_plot_state();
        let update = PlotUpdate {
            new_scene: None,
            plot_text: "你推开山门，风声呼啸。".to_string(),
            triggered_events: vec![],
            state_changes: vec![],
            is_waiting_for_input: false,
            available_options: vec![],
            chapter_title: None,
            chapter_summary: None,
            chapter_end: false,
            generation_diagnostics: None,
        };
        let report = validate_and_repair_plot_update(&state, &update, &action_result(), 1, 100, "无名弟子");
        assert!(report.repaired_plot_text.is_some());
        assert!(report.issues.iter().any(|i| i.code == "duplicate_segment"));
    }

    #[test]
    fn flags_waiting_without_options() {
        let state = test_plot_state();
        let update = PlotUpdate {
            new_scene: None,
            plot_text: "局势推进".to_string(),
            triggered_events: vec![],
            state_changes: vec![],
            is_waiting_for_input: true,
            available_options: vec![],
            chapter_title: None,
            chapter_summary: None,
            chapter_end: false,
            generation_diagnostics: None,
        };
        let report = validate_and_repair_plot_update(&state, &update, &action_result(), 1, 100, "无名弟子");
        assert!(report.force_non_waiting);
        assert!(report.issues.iter().any(|i| i.code == "waiting_without_options"));
    }

    #[test]
    fn detects_realm_power_conflict_for_high_realm_player() {
        let state = test_plot_state();
        let update = PlotUpdate {
            new_scene: None,
            plot_text: "你面对练气期妖兽竟心生畏惧，几乎落荒而逃。".to_string(),
            triggered_events: vec![],
            state_changes: vec![],
            is_waiting_for_input: false,
            available_options: vec![],
            chapter_title: None,
            chapter_summary: None,
            chapter_end: false,
            generation_diagnostics: None,
        };
        let report = validate_and_repair_plot_update(&state, &update, &action_result(), 4, 6200, "无名弟子");
        assert!(report.issues.iter().any(|i| i.code == "realm_power_conflict"));
        assert!(report.repaired_plot_text.is_some());
    }

    #[test]
    fn detects_title_drift() {
        let state = test_plot_state();
        let update = PlotUpdate {
            new_scene: None,
            plot_text: "无名弟子看向远处，他突然心神不宁。".to_string(),
            triggered_events: vec![],
            state_changes: vec![],
            is_waiting_for_input: false,
            available_options: vec![],
            chapter_title: None,
            chapter_summary: None,
            chapter_end: false,
            generation_diagnostics: None,
        };
        let report = validate_and_repair_plot_update(&state, &update, &action_result(), 2, 500, "无名弟子");
        assert!(report.issues.iter().any(|i| i.code == "title_drift"));
    }

    #[test]
    fn detects_location_transition() {
        let state = test_plot_state();
        let update = PlotUpdate {
            new_scene: None,
            plot_text: "你一路前行，来到黑风谷，四周杀气弥漫。".to_string(),
            triggered_events: vec![],
            state_changes: vec![],
            is_waiting_for_input: false,
            available_options: vec![],
            chapter_title: None,
            chapter_summary: None,
            chapter_end: false,
            generation_diagnostics: None,
        };
        let report = validate_and_repair_plot_update(&state, &update, &action_result(), 2, 500, "无名弟子");
        assert_eq!(report.override_location.as_deref(), Some("黑风谷"));
    }

    #[test]
    fn adds_chapter_goal_hint_when_missing() {
        let mut state = test_plot_state();
        state.current_chapter.interaction_count = 2;
        let update = PlotUpdate {
            new_scene: None,
            plot_text: "你踏入石阶，四周寂静无声。".to_string(),
            triggered_events: vec![],
            state_changes: vec![],
            is_waiting_for_input: false,
            available_options: vec![],
            chapter_title: None,
            chapter_summary: None,
            chapter_end: false,
            generation_diagnostics: None,
        };
        let report = validate_and_repair_plot_update(&state, &update, &action_result(), 2, 500, "无名弟子");
        assert!(report.issues.iter().any(|i| i.code == "chapter_goal_weak"));
    }

    #[test]
    fn detects_cross_chapter_duplicate() {
        let mut state = test_plot_state();
        state.chapters.push(ChapterState {
            index: 0,
            title: "序章".to_string(),
            content: vec!["你推开山门，风声呼啸。".to_string()],
            summary: String::new(),
            interaction_count: 1,
            status: Default::default(),
        });
        let update = PlotUpdate {
            new_scene: None,
            plot_text: "你推开山门，风声呼啸。".to_string(),
            triggered_events: vec![],
            state_changes: vec![],
            is_waiting_for_input: false,
            available_options: vec![],
            chapter_title: None,
            chapter_summary: None,
            chapter_end: false,
            generation_diagnostics: None,
        };
        let report = validate_and_repair_plot_update(&state, &update, &action_result(), 2, 500, "无名弟子");
        assert!(report
            .issues
            .iter()
            .any(|i| i.code == "duplicate_cross_chapter"));
    }
}
