use crate::entity_types::{EntityCandidateRequest, ValidationStatus};
use crate::entity_validator::resolve_candidate;
use crate::noname_roles::NoNameDirectorObservation;
use crate::noname_trace::NoNameGuardrailTraceResult;
use crate::noname_types::{
    NoNameApplyScope, NoNameMode, NoNameProposal, NoNameProposalStatus, NoNameTargetSegment,
};
use crate::numeric_guard::{validate_character_combat_power, validate_map_numbers};
use crate::numerical_system::ActionResult;
use crate::plot_consistency::{validate_and_repair_plot_update, IssueLevel};
use crate::plot_engine::{PlotState, PlotUpdate};
use crate::state_patch_validator::validate_patch_row;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoNameGuardrailOutcome {
    Accept,
    Repair,
    Reject,
}

impl NoNameGuardrailOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Accept => "accept",
            Self::Repair => "repair",
            Self::Reject => "reject",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameGuardrailResult {
    pub outcome: NoNameGuardrailOutcome,
    pub reason: Option<String>,
    #[serde(default)]
    pub details: Vec<String>,
    pub repaired_focus: Option<String>,
    pub repaired_rationale: Option<String>,
}

impl NoNameGuardrailResult {
    pub fn accept() -> Self {
        Self {
            outcome: NoNameGuardrailOutcome::Accept,
            reason: None,
            details: Vec::new(),
            repaired_focus: None,
            repaired_rationale: None,
        }
    }

    pub fn reject(reason: impl Into<String>) -> Self {
        Self {
            outcome: NoNameGuardrailOutcome::Reject,
            reason: Some(reason.into()),
            details: Vec::new(),
            repaired_focus: None,
            repaired_rationale: None,
        }
    }

    pub fn repair(
        reason: impl Into<String>,
        details: Vec<String>,
        repaired_focus: Option<String>,
        repaired_rationale: Option<String>,
    ) -> Self {
        Self {
            outcome: NoNameGuardrailOutcome::Repair,
            reason: Some(reason.into()),
            details,
            repaired_focus,
            repaired_rationale,
        }
    }

    pub fn is_rejected(&self) -> bool {
        self.outcome == NoNameGuardrailOutcome::Reject
    }

    pub fn to_trace_result(&self) -> NoNameGuardrailTraceResult {
        NoNameGuardrailTraceResult {
            outcome: self.outcome.as_str().to_string(),
            reason: self.reason.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NoNameDirectorGuardrailInput {
    pub plot_state: PlotState,
    pub action_result: ActionResult,
    pub player_name: String,
    pub player_realm_level: u32,
    pub player_combat_power: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoNameApplyGuardrailOutcome {
    Ready,
    ModeForbidden,
    ProposalBlocked,
    StateRisk,
    PlotRisk,
    FallbackRequired,
}

impl NoNameApplyGuardrailOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::ModeForbidden => "mode_forbidden",
            Self::ProposalBlocked => "proposal_blocked",
            Self::StateRisk => "state_risk",
            Self::PlotRisk => "plot_risk",
            Self::FallbackRequired => "fallback_required",
        }
    }

    pub fn allows_apply(self) -> bool {
        matches!(self, Self::Ready)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameApplyGuardrailResult {
    pub outcome: NoNameApplyGuardrailOutcome,
    pub reason: Option<String>,
    #[serde(default)]
    pub details: Vec<String>,
}

impl NoNameApplyGuardrailResult {
    pub fn ready() -> Self {
        Self {
            outcome: NoNameApplyGuardrailOutcome::Ready,
            reason: None,
            details: Vec::new(),
        }
    }

    pub fn blocked(
        outcome: NoNameApplyGuardrailOutcome,
        reason: impl Into<String>,
        details: Vec<String>,
    ) -> Self {
        Self {
            outcome,
            reason: Some(reason.into()),
            details,
        }
    }
}

pub fn validate_director_observation(
    observation: &NoNameDirectorObservation,
    input: &NoNameDirectorGuardrailInput,
) -> NoNameGuardrailResult {
    let focus = observation.focus.trim();
    let rationale = observation.rationale.trim();
    if focus.is_empty() {
        return NoNameGuardrailResult::reject("Director proposal focus 不能为空");
    }
    if rationale.is_empty() {
        return NoNameGuardrailResult::reject("Director proposal rationale 不能为空");
    }

    let mut repaired_focus = None;
    let mut repaired_rationale = None;
    let mut details = Vec::new();
    let mut candidate_focus = focus.to_string();
    let mut candidate_rationale = rationale.to_string();

    if candidate_focus.chars().count() > 48 {
        candidate_focus = candidate_focus.chars().take(48).collect();
        repaired_focus = Some(candidate_focus.clone());
        details.push("focus 超过 48 字，已裁剪为调试安全长度".to_string());
    }

    if candidate_rationale.chars().count() > 160 {
        candidate_rationale = candidate_rationale.chars().take(160).collect();
        repaired_rationale = Some(candidate_rationale.clone());
        details.push("rationale 超过 160 字，已裁剪为调试安全长度".to_string());
    }

    let numeric =
        validate_character_combat_power(input.player_realm_level, input.player_combat_power);
    if !numeric.accepted {
        return NoNameGuardrailResult::reject(
            numeric
                .reason
                .unwrap_or_else(|| "玩家战力基线未通过数值护栏".to_string()),
        );
    }
    if numeric.normalized {
        details.push(
            numeric
                .reason
                .unwrap_or_else(|| "玩家战力基线已被数值护栏归一化".to_string()),
        );
    }

    let synthetic_update = PlotUpdate {
        new_scene: None,
        plot_text: format!(
            "NoName Director Focus: {}\n\nNoName Director Rationale: {}",
            candidate_focus, candidate_rationale
        ),
        triggered_events: Vec::new(),
        state_changes: Vec::new(),
        is_waiting_for_input: true,
        available_options: input.plot_state.current_scene.available_options.clone(),
        chapter_title: None,
        chapter_summary: Some(format!("NoName 观察焦点：{}", candidate_focus)),
        chapter_end: false,
        generation_diagnostics: Some("NoName guardrail preflight".to_string()),
    };

    let report = validate_and_repair_plot_update(
        &input.plot_state,
        &synthetic_update,
        &input.action_result,
        input.player_realm_level,
        input.player_combat_power,
        &input.player_name,
    );

    if !report.issues.is_empty() {
        let critical = report
            .issues
            .iter()
            .any(|issue| issue.level == IssueLevel::Critical);
        if let Some(diag) = report.to_diagnostics() {
            details.push(diag);
        }

        if critical && report.repaired_plot_text.is_none() {
            return NoNameGuardrailResult::reject(
                report
                    .issues
                    .first()
                    .map(|issue| issue.message.clone())
                    .unwrap_or_else(|| "Director proposal 未通过剧情一致性护栏".to_string()),
            );
        }

        return NoNameGuardrailResult::repair(
            "Director proposal 已通过护栏修复",
            details,
            repaired_focus,
            repaired_rationale,
        );
    }

    if repaired_focus.is_some() || repaired_rationale.is_some() || !details.is_empty() {
        return NoNameGuardrailResult::repair(
            "Director proposal 已通过轻量修复",
            details,
            repaired_focus,
            repaired_rationale,
        );
    }

    NoNameGuardrailResult::accept()
}

pub fn validate_director_proposal_for_apply(
    mode: NoNameMode,
    proposal: &NoNameProposal,
    input: Option<&NoNameDirectorGuardrailInput>,
) -> NoNameApplyGuardrailResult {
    if !mode.allows_apply() {
        return NoNameApplyGuardrailResult::blocked(
            NoNameApplyGuardrailOutcome::ModeForbidden,
            "当前模式不允许进入 assisted apply",
            Vec::new(),
        );
    }

    if proposal.status == NoNameProposalStatus::Blocked {
        return NoNameApplyGuardrailResult::blocked(
            NoNameApplyGuardrailOutcome::ProposalBlocked,
            "proposal 已在上游护栏阶段被阻塞",
            Vec::new(),
        );
    }

    if proposal.status != NoNameProposalStatus::Ready {
        return NoNameApplyGuardrailResult::blocked(
            NoNameApplyGuardrailOutcome::FallbackRequired,
            format!(
                "proposal 当前状态为 {}，不能进入 apply",
                proposal.status.as_str()
            ),
            Vec::new(),
        );
    }

    let Some(input) = input else {
        return NoNameApplyGuardrailResult::blocked(
            NoNameApplyGuardrailOutcome::FallbackRequired,
            "缺少 apply guardrail 输入，回退经典链路",
            Vec::new(),
        );
    };

    if !input.plot_state.is_waiting_for_input {
        return NoNameApplyGuardrailResult::blocked(
            NoNameApplyGuardrailOutcome::PlotRisk,
            "当前剧情不处于等待输入状态，暂不进入 apply",
            vec!["plot_state.is_waiting_for_input=false".to_string()],
        );
    }

    let numeric =
        validate_character_combat_power(input.player_realm_level, input.player_combat_power);
    if !numeric.accepted {
        return NoNameApplyGuardrailResult::blocked(
            NoNameApplyGuardrailOutcome::StateRisk,
            numeric
                .reason
                .unwrap_or_else(|| "玩家状态未通过 apply 护栏".to_string()),
            Vec::new(),
        );
    }

    if proposal.focus.trim().is_empty() {
        return NoNameApplyGuardrailResult::blocked(
            NoNameApplyGuardrailOutcome::FallbackRequired,
            "proposal 缺少 focus，回退经典链路",
            Vec::new(),
        );
    }

    let intended_effect = proposal.intended_effect.trim();
    if intended_effect.is_empty() {
        return NoNameApplyGuardrailResult::blocked(
            NoNameApplyGuardrailOutcome::FallbackRequired,
            "proposal 缺少 intended_effect，回退经典链路",
            Vec::new(),
        );
    }

    let intended_effect_len = intended_effect.chars().count();
    if intended_effect_len > 80 {
        return NoNameApplyGuardrailResult::blocked(
            NoNameApplyGuardrailOutcome::FallbackRequired,
            "proposal intended_effect 过长，回退经典链路",
            vec![format!("intended_effect_len={}", intended_effect_len)],
        );
    }

    if matches!(
        proposal.target_segment,
        NoNameTargetSegment::ChapterSummaryHead | NoNameTargetSegment::ChapterSummaryTail
    ) && proposal
        .apply_scopes
        .contains(&NoNameApplyScope::PlotTextHint)
    {
        return NoNameApplyGuardrailResult::blocked(
            NoNameApplyGuardrailOutcome::FallbackRequired,
            "proposal target_segment 与 apply_scope 不匹配，回退经典链路",
            vec![
                format!("target_segment={}", proposal.target_segment.as_str()),
                "apply_scope=plot_text_hint".to_string(),
            ],
        );
    }

    NoNameApplyGuardrailResult::ready()
}

pub fn validate_state_patch_rows(
    table: &str,
    rows: &[Map<String, Value>],
) -> NoNameGuardrailResult {
    for row in rows {
        if let Err(err) = validate_patch_row(table, row) {
            return NoNameGuardrailResult::reject(err);
        }
    }
    NoNameGuardrailResult::accept()
}

pub fn validate_entity_candidate_proposal(
    candidate: &EntityCandidateRequest,
) -> NoNameGuardrailResult {
    let resolved = resolve_candidate(candidate);
    match resolved.validation_report.status {
        ValidationStatus::Accepted => NoNameGuardrailResult::accept(),
        ValidationStatus::Normalized => NoNameGuardrailResult::repair(
            "实体提案已规范化",
            resolved.validation_report.reasons,
            None,
            None,
        ),
        ValidationStatus::Rejected => NoNameGuardrailResult::reject(
            resolved
                .validation_report
                .reasons
                .join("；")
                .trim()
                .to_string(),
        ),
    }
}

pub fn validate_map_hint(danger_tier: u8, aura_density: f64) -> NoNameGuardrailResult {
    let check = validate_map_numbers(danger_tier, aura_density);
    if !check.accepted {
        return NoNameGuardrailResult::reject(
            check
                .reason
                .unwrap_or_else(|| "地图数值提示未通过护栏".to_string()),
        );
    }
    if check.normalized {
        return NoNameGuardrailResult::repair(
            check
                .reason
                .unwrap_or_else(|| "地图数值提示已被护栏归一化".to_string()),
            Vec::new(),
            None,
            None,
        );
    }
    NoNameGuardrailResult::accept()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity_types::EntityType;
    use crate::numerical_system::Action;
    use crate::plot_engine::{
        ChapterState, PlayerOption, PlotInteractionState, PlotSettings, Scene,
    };
    use serde_json::json;

    fn make_plot_state() -> PlotState {
        PlotState {
            current_scene: Scene {
                id: "scene-1".to_string(),
                name: "山门".to_string(),
                description: "山门风声渐紧".to_string(),
                location: "山门".to_string(),
                available_options: vec![PlayerOption {
                    id: 0,
                    description: "返回山门广场".to_string(),
                    requirements: Vec::new(),
                    action: Action::Rest,
                }],
            },
            plot_history: vec!["上一段剧情".to_string()],
            is_waiting_for_input: true,
            interaction_state: PlotInteractionState::WaitingForChoice,
            last_action_result: None,
            settings: PlotSettings::default(),
            current_chapter: ChapterState::new(1, "第一章".to_string()),
            chapters: Vec::new(),
            segment_count: 0,
            last_generation_diagnostics: None,
            last_option_generation_source: None,
            last_consistency_risk_score: None,
        }
    }

    fn make_guardrail_input() -> NoNameDirectorGuardrailInput {
        NoNameDirectorGuardrailInput {
            plot_state: make_plot_state(),
            action_result: ActionResult {
                success: true,
                description: "玩家返回山门".to_string(),
                stat_changes: Vec::new(),
                events: Vec::new(),
            },
            player_name: "无名弟子".to_string(),
            player_realm_level: 1,
            player_combat_power: 150,
        }
    }

    fn make_director_proposal(focus: &str, rationale: &str) -> crate::noname_types::NoNameProposal {
        crate::noname_types::NoNameProposal {
            proposal_id: "proposal-1".to_string(),
            kind: crate::noname_types::NoNameProposalKind::PlotCandidate,
            producer_role: crate::noname_types::NoNameRole::Director,
            title: format!("Director提案：{}", focus.trim()),
            summary: format!("围绕“{}”继续推进观察。", focus.trim()),
            focus: focus.to_string(),
            target_segment: crate::noname_types::NoNameTargetSegment::CurrentTurnTail,
            intended_effect: "维持当前章节冲突的低风险延续".to_string(),
            rationale: rationale.to_string(),
            suggested_action: Some("保持 observe-only".to_string()),
            labels: vec!["director".to_string()],
            apply_scopes: vec![
                crate::noname_types::NoNameApplyScope::Diagnostics,
                crate::noname_types::NoNameApplyScope::ChapterSummaryHint,
            ],
            status: crate::noname_types::NoNameProposalStatus::Observed,
            applyable: false,
        }
    }

    #[test]
    fn director_guardrail_rejects_blank_focus() {
        let observation = NoNameDirectorObservation {
            role: crate::noname_types::NoNameRole::Director,
            action_summary: "返回山门".to_string(),
            focus: "   ".to_string(),
            rationale: "保持山门线索推进".to_string(),
            prompt_preview: "prompt".to_string(),
            proposal: make_director_proposal("   ", "保持山门线索推进"),
        };

        let result = validate_director_observation(&observation, &make_guardrail_input());
        assert_eq!(result.outcome, NoNameGuardrailOutcome::Reject);
    }

    #[test]
    fn director_guardrail_repairs_overlong_focus() {
        let observation = NoNameDirectorObservation {
            role: crate::noname_types::NoNameRole::Director,
            action_summary: "返回山门".to_string(),
            focus: "山门危机".repeat(13),
            rationale: "优先观察山门内部的冲突升级与人物反应".to_string(),
            prompt_preview: "prompt".to_string(),
            proposal: make_director_proposal(
                &"山门危机".repeat(13),
                "优先观察山门内部的冲突升级与人物反应",
            ),
        };

        let result = validate_director_observation(&observation, &make_guardrail_input());
        assert_eq!(result.outcome, NoNameGuardrailOutcome::Repair);
        assert!(result.repaired_focus.is_some());
    }

    #[test]
    fn apply_guardrail_blocks_when_mode_is_not_assisted() {
        let result = validate_director_proposal_for_apply(
            NoNameMode::ObserveOnly,
            &make_director_proposal("山门危机", "保持观察"),
            Some(&make_guardrail_input()),
        );
        assert_eq!(result.outcome, NoNameApplyGuardrailOutcome::ModeForbidden);
    }

    #[test]
    fn apply_guardrail_requires_ready_proposal() {
        let proposal = make_director_proposal("山门危机", "保持观察");
        let result = validate_director_proposal_for_apply(
            NoNameMode::Assisted,
            &proposal,
            Some(&make_guardrail_input()),
        );
        assert_eq!(
            result.outcome,
            NoNameApplyGuardrailOutcome::FallbackRequired
        );
    }

    #[test]
    fn apply_guardrail_rejects_non_waiting_plot_state() {
        let mut proposal = make_director_proposal("山门危机", "保持观察");
        proposal.status = crate::noname_types::NoNameProposalStatus::Ready;
        proposal.applyable = true;
        let mut input = make_guardrail_input();
        input.plot_state.is_waiting_for_input = false;

        let result =
            validate_director_proposal_for_apply(NoNameMode::Assisted, &proposal, Some(&input));
        assert_eq!(result.outcome, NoNameApplyGuardrailOutcome::PlotRisk);
    }

    #[test]
    fn apply_guardrail_accepts_ready_proposal_with_valid_input() {
        let mut proposal = make_director_proposal("山门危机", "保持观察");
        proposal.status = crate::noname_types::NoNameProposalStatus::Ready;
        proposal.applyable = true;

        let result = validate_director_proposal_for_apply(
            NoNameMode::Assisted,
            &proposal,
            Some(&make_guardrail_input()),
        );
        assert_eq!(result.outcome, NoNameApplyGuardrailOutcome::Ready);
        assert!(result.outcome.allows_apply());
    }

    #[test]
    fn apply_guardrail_requires_intended_effect() {
        let mut proposal = make_director_proposal("山门危机", "保持观察");
        proposal.status = crate::noname_types::NoNameProposalStatus::Ready;
        proposal.applyable = true;
        proposal.intended_effect = "   ".to_string();

        let result = validate_director_proposal_for_apply(
            NoNameMode::Assisted,
            &proposal,
            Some(&make_guardrail_input()),
        );
        assert_eq!(
            result.outcome,
            NoNameApplyGuardrailOutcome::FallbackRequired
        );
        assert_eq!(
            result.reason.as_deref(),
            Some("proposal 缺少 intended_effect，回退经典链路")
        );
    }

    #[test]
    fn apply_guardrail_rejects_summary_target_with_plot_text_scope() {
        let mut proposal = make_director_proposal("山门危机", "保持观察");
        proposal.status = crate::noname_types::NoNameProposalStatus::Ready;
        proposal.applyable = true;
        proposal.target_segment = crate::noname_types::NoNameTargetSegment::ChapterSummaryHead;
        proposal
            .apply_scopes
            .push(crate::noname_types::NoNameApplyScope::PlotTextHint);

        let result = validate_director_proposal_for_apply(
            NoNameMode::Assisted,
            &proposal,
            Some(&make_guardrail_input()),
        );
        assert_eq!(
            result.outcome,
            NoNameApplyGuardrailOutcome::FallbackRequired
        );
        assert!(result
            .details
            .iter()
            .any(|item| item == "apply_scope=plot_text_hint"));
    }

    #[test]
    fn state_patch_guardrail_rejects_missing_fields() {
        let rows = vec![Map::from_iter([(
            "name".to_string(),
            Value::String("青云宗".to_string()),
        )])];

        let result = validate_state_patch_rows("factions", &rows);
        assert_eq!(result.outcome, NoNameGuardrailOutcome::Reject);
    }

    #[test]
    fn entity_guardrail_rejects_invalid_technique_candidate() {
        let candidate = EntityCandidateRequest {
            entity_type: EntityType::Technique,
            payload: json!({
                "techniqueId": "",
                "name": "",
                "realmRequirement": 99,
                "basePower": 99999.0,
                "description": ""
            }),
            source_trace_id: Some("trace-1".to_string()),
        };

        let result = validate_entity_candidate_proposal(&candidate);
        assert_eq!(result.outcome, NoNameGuardrailOutcome::Reject);
    }

    #[test]
    fn map_guardrail_rejects_out_of_range_hint() {
        let result = validate_map_hint(99, 0.5);
        assert_eq!(result.outcome, NoNameGuardrailOutcome::Reject);
    }
}
