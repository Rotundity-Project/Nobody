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
}

impl ConsistencyReport {
    pub fn to_diagnostics(&self) -> Option<String> {
        if self.issues.is_empty() {
            return None;
        }
        let body = self
            .issues
            .iter()
            .map(|i| format!("[{}:{}] {}", level_tag(&i.level), i.code, i.message))
            .collect::<Vec<_>>()
            .join("；");
        Some(format!("一致性校验器V2：{}", body))
    }
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

pub fn validate_and_repair_plot_update(
    plot_state: &PlotState,
    plot_update: &PlotUpdate,
    action_result: &ActionResult,
) -> ConsistencyReport {
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
            .take(3)
            .any(|old| {
                let normalized_old = normalize_text(old);
                if normalized_old.is_empty() {
                    return false;
                }
                normalized_old == normalized_current
                    || jaccard_similarity(&normalized_old, &normalized_current) >= 0.92
            });
        if duplicate {
            report.issues.push(ConsistencyIssue {
                level: IssueLevel::Warning,
                code: "duplicate_segment",
                message: "检测到与近期剧情高度重复，已替换为去重段落".to_string(),
            });
            report.repaired_plot_text = Some(build_repair_text(plot_state, action_result));
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

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::numerical_system::ActionResult;
    use crate::plot_engine::{ChapterState, PlotSettings, Scene};

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
        let report = validate_and_repair_plot_update(&state, &update, &action_result());
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
        let report = validate_and_repair_plot_update(&state, &update, &action_result());
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
        let report = validate_and_repair_plot_update(&state, &update, &action_result());
        assert!(report.force_non_waiting);
        assert!(report.issues.iter().any(|i| i.code == "waiting_without_options"));
    }
}

