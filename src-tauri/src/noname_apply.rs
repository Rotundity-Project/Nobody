use crate::noname_trace::{NoNameHumanReviewDecision, NoNameTrace};
use crate::noname_types::{NoNameApplyScope, NoNameProposal, NoNameTargetSegment};
use crate::plot_engine::PlotState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoNameApplySegmentSnapshot {
    pub chapter_index: u32,
    pub segment_index: usize,
    pub expected_segment_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoNameApplySummarySnapshot {
    pub chapter_index: u32,
    pub expected_summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoNameApplyDiagnosticsSnapshot {
    pub chapter_index: u32,
    pub expected_generation_diagnostics: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoNameReviewedApplyRequest {
    pub request_id: String,
    pub scope: NoNameApplyScope,
    pub segment_snapshot: Option<NoNameApplySegmentSnapshot>,
    pub summary_snapshot: Option<NoNameApplySummarySnapshot>,
    pub diagnostics_snapshot: Option<NoNameApplyDiagnosticsSnapshot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoNameReviewedApplyOutcome {
    pub trace: NoNameTrace,
    pub plot_state: PlotState,
}

pub fn build_manual_plot_text_apply_request(
    request_id: impl Into<String>,
    chapter_index: u32,
    segment_index: usize,
    expected_segment_text: impl Into<String>,
) -> NoNameReviewedApplyRequest {
    NoNameReviewedApplyRequest {
        request_id: request_id.into(),
        scope: NoNameApplyScope::PlotTextHint,
        segment_snapshot: Some(NoNameApplySegmentSnapshot {
            chapter_index,
            segment_index,
            expected_segment_text: expected_segment_text.into(),
        }),
        summary_snapshot: None,
        diagnostics_snapshot: None,
    }
}

pub fn build_reviewed_apply_request(
    request_id: String,
    scope: NoNameApplyScope,
    chapter_index: Option<u32>,
    segment_index: Option<usize>,
    expected_segment_text: Option<String>,
    expected_summary: Option<String>,
    expected_generation_diagnostics: Option<String>,
) -> Result<NoNameReviewedApplyRequest, String> {
    let segment_snapshot = match scope {
        NoNameApplyScope::PlotTextHint => Some(NoNameApplySegmentSnapshot {
            chapter_index: chapter_index
                .ok_or_else(|| "plot_text_hint manual apply requires chapter_index".to_string())?,
            segment_index: segment_index
                .ok_or_else(|| "plot_text_hint manual apply requires segment_index".to_string())?,
            expected_segment_text: expected_segment_text.ok_or_else(|| {
                "plot_text_hint manual apply requires expected_segment_text".to_string()
            })?,
        }),
        _ => None,
    };
    let summary_snapshot = match scope {
        NoNameApplyScope::ChapterSummaryHint => Some(NoNameApplySummarySnapshot {
            chapter_index: chapter_index.ok_or_else(|| {
                "chapter_summary_hint manual apply requires chapter_index".to_string()
            })?,
            expected_summary: expected_summary.ok_or_else(|| {
                "chapter_summary_hint manual apply requires expected_summary".to_string()
            })?,
        }),
        _ => None,
    };
    let diagnostics_snapshot = match scope {
        NoNameApplyScope::OptionBiasHint => Some(NoNameApplyDiagnosticsSnapshot {
            chapter_index: chapter_index.ok_or_else(|| {
                "option_bias_hint manual apply requires chapter_index".to_string()
            })?,
            expected_generation_diagnostics: expected_generation_diagnostics.ok_or_else(|| {
                "option_bias_hint manual apply requires expected_generation_diagnostics".to_string()
            })?,
        }),
        _ => None,
    };

    Ok(NoNameReviewedApplyRequest {
        request_id,
        scope,
        segment_snapshot,
        summary_snapshot,
        diagnostics_snapshot,
    })
}

fn priority_for_apply_scope(scope: NoNameApplyScope) -> u32 {
    match scope {
        NoNameApplyScope::PlotTextHint => 300,
        NoNameApplyScope::ChapterSummaryHint => 200,
        NoNameApplyScope::OptionBiasHint => 100,
        NoNameApplyScope::Diagnostics => 50,
    }
}

fn next_apply_plan_order(trace: &NoNameTrace) -> u32 {
    trace
        .apply_plan_log
        .iter()
        .map(|item| item.order)
        .max()
        .unwrap_or_default()
        + 1
}

fn find_review_proposal<'a>(
    trace: &'a NoNameTrace,
    request_id: &str,
) -> Option<&'a NoNameProposal> {
    let proposal_id = trace
        .controlled_output_reviews
        .iter()
        .find(|review| review.request_id == request_id)
        .and_then(|review| review.proposal_id.as_deref())?;

    trace
        .proposals
        .iter()
        .rev()
        .find(|proposal| proposal.proposal_id == proposal_id)
}

fn target_segment_supports_apply_scope(
    target_segment: NoNameTargetSegment,
    scope: NoNameApplyScope,
) -> bool {
    match scope {
        NoNameApplyScope::PlotTextHint => matches!(
            target_segment,
            NoNameTargetSegment::CurrentTurnHead | NoNameTargetSegment::CurrentTurnTail
        ),
        _ => true,
    }
}

fn trace_has_second_guardrail_allow(
    trace: &NoNameTrace,
    request_id: &str,
    scope: NoNameApplyScope,
) -> bool {
    let transition = format!("{}:second_guardrail:allow", request_id);
    trace
        .proposal_transition_log
        .iter()
        .any(|entry| entry == &transition)
        || trace.apply_execution_log.iter().any(|entry| {
            entry.target == scope.as_str() && entry.outcome == "second_guardrail_allowed"
        })
}

fn build_plot_text_hint(proposal: &NoNameProposal) -> String {
    format!("【NoName】重点关注：{}", proposal.focus.trim())
}

fn build_chapter_summary_hint(proposal: &NoNameProposal) -> String {
    format!("NoName summary hint: {}", proposal.focus.trim())
}

fn build_option_bias_hint(proposal: &NoNameProposal) -> String {
    format!(
        "NoName option bias: next turn should prioritize actions around {}",
        proposal.focus.trim()
    )
}

fn manual_apply_outcome_for_scope(scope: NoNameApplyScope) -> &'static str {
    match scope {
        NoNameApplyScope::PlotTextHint => "manual_plot_text_applied",
        NoNameApplyScope::ChapterSummaryHint => "manual_chapter_summary_hint_applied",
        NoNameApplyScope::OptionBiasHint => "manual_option_bias_hint_applied",
        NoNameApplyScope::Diagnostics => "manual_diagnostics_applied",
    }
}

fn validate_reviewed_apply_request<'a>(
    trace: &'a NoNameTrace,
    request: &'a NoNameReviewedApplyRequest,
) -> Result<&'a NoNameProposal, String> {
    let review = trace
        .controlled_output_reviews
        .iter()
        .find(|review| review.request_id == request.request_id)
        .ok_or_else(|| {
            format!(
                "NoName controlled output review not found: {}",
                request.request_id
            )
        })?;

    if review.human_review_decision != Some(NoNameHumanReviewDecision::ApprovedForHigherApply) {
        return Err(format!(
            "NoName controlled output review is not approved for manual apply: {}",
            request.request_id
        ));
    }
    if review.safe_apply_scope != Some(request.scope) {
        return Err(format!(
            "manual apply scope mismatch: expected {}, review scope {:?}",
            request.scope.as_str(),
            review.safe_apply_scope
        ));
    }
    if !trace_has_second_guardrail_allow(trace, &request.request_id, request.scope) {
        return Err("second guardrail has not allowed this review".to_string());
    }
    if trace.apply_execution_log.iter().any(|entry| {
        entry.target == request.scope.as_str()
            && entry.outcome == manual_apply_outcome_for_scope(request.scope)
    }) {
        return Err(format!(
            "manual {} has already been applied for this trace",
            request.scope.as_str()
        ));
    }

    let proposal = find_review_proposal(trace, &request.request_id)
        .ok_or_else(|| "NoName proposal not found for manual apply".to_string())?;
    if !target_segment_supports_apply_scope(proposal.target_segment, request.scope) {
        return Err(format!(
            "target_segment={} does not support manual {}",
            proposal.target_segment.as_str(),
            request.scope.as_str()
        ));
    }

    Ok(proposal)
}

fn apply_plot_text_hint_to_segment(
    mut trace: NoNameTrace,
    mut plot_state: PlotState,
    request: &NoNameReviewedApplyRequest,
    proposal: &NoNameProposal,
) -> Result<NoNameReviewedApplyOutcome, String> {
    let Some(snapshot) = &request.segment_snapshot else {
        return Err("plot_text_hint manual apply requires a segment snapshot".to_string());
    };
    if plot_state.current_chapter.index != snapshot.chapter_index {
        return Err(format!(
            "chapter mismatch: expected {}, current {}",
            snapshot.chapter_index, plot_state.current_chapter.index
        ));
    }
    let Some(current_segment) = plot_state
        .current_chapter
        .content
        .get(snapshot.segment_index)
    else {
        return Err(format!(
            "segment index out of range: {}",
            snapshot.segment_index
        ));
    };
    if current_segment != &snapshot.expected_segment_text {
        return Err("segment snapshot mismatch; refusing stale manual apply".to_string());
    }
    if snapshot.expected_segment_text.contains("【NoName】")
        || snapshot.expected_segment_text.contains("NoName提示")
    {
        return Err("segment already contains a NoName marker".to_string());
    }

    let hint = build_plot_text_hint(proposal);
    let updated_segment = match proposal.target_segment {
        NoNameTargetSegment::CurrentTurnHead => {
            format!("{}\n\n{}", hint, snapshot.expected_segment_text.trim())
        }
        _ => format!("{}\n\n{}", snapshot.expected_segment_text.trim(), hint),
    };

    plot_state.current_chapter.content[snapshot.segment_index] = updated_segment.clone();
    if let Some(history_segment) = plot_state
        .plot_history
        .iter_mut()
        .rev()
        .find(|item| item.as_str() == snapshot.expected_segment_text)
    {
        *history_segment = updated_segment.clone();
    } else {
        return Err("plot history snapshot mismatch; refusing partial manual apply".to_string());
    }
    plot_state.current_scene.description = plot_state.current_chapter.content.join("\n\n");
    if let Some(result) = plot_state.last_action_result.as_mut() {
        if result.description == snapshot.expected_segment_text {
            result.description = updated_segment;
        }
    }

    let order = next_apply_plan_order(&trace);
    trace.record_apply_plan(
        order,
        request.scope.as_str(),
        "manual_apply",
        priority_for_apply_scope(request.scope) + 75,
        Some(format!(
            "显式人工 apply 已确认 chapter={} segment={}，准备写入 {}",
            snapshot.chapter_index,
            snapshot.segment_index,
            request.scope.as_str()
        )),
    );
    trace.record_apply_execution(
        request.scope.as_str(),
        "manual_plot_text_applied",
        Some(format!(
            "已由显式人工命令写入正文提示，聚焦“{}”",
            proposal.focus
        )),
    );
    trace.record_proposal_transition(format!(
        "{}:manual_apply:plot_text_hint",
        request.request_id
    ));
    trace.set_apply_result(
        true,
        "manual_plot_text_applied",
        Some("显式人工 apply 已写入正文提示".to_string()),
    );

    Ok(NoNameReviewedApplyOutcome { trace, plot_state })
}

fn apply_chapter_summary_hint(
    mut trace: NoNameTrace,
    mut plot_state: PlotState,
    request: &NoNameReviewedApplyRequest,
    proposal: &NoNameProposal,
) -> Result<NoNameReviewedApplyOutcome, String> {
    let Some(snapshot) = &request.summary_snapshot else {
        return Err("chapter_summary_hint manual apply requires a summary snapshot".to_string());
    };
    if plot_state.current_chapter.index != snapshot.chapter_index {
        return Err(format!(
            "chapter mismatch: expected {}, current {}",
            snapshot.chapter_index, plot_state.current_chapter.index
        ));
    }
    if plot_state.current_chapter.summary != snapshot.expected_summary {
        return Err("summary snapshot mismatch; refusing stale manual apply".to_string());
    }

    let focus = proposal.focus.trim();
    let hint = build_chapter_summary_hint(proposal);
    if !focus.is_empty()
        && (snapshot.expected_summary.contains(focus) || snapshot.expected_summary.contains(&hint))
    {
        return Err("chapter summary already contains this NoName hint".to_string());
    }

    let summary = snapshot.expected_summary.trim();
    let updated_summary = if summary.is_empty() {
        hint.clone()
    } else {
        match proposal.target_segment {
            NoNameTargetSegment::ChapterSummaryHead => format!("{}; {}", hint, summary),
            _ => format!("{}; {}", summary, hint),
        }
    };

    plot_state.current_chapter.summary = updated_summary.clone();
    if let Some(history_chapter) = plot_state
        .chapters
        .iter_mut()
        .find(|chapter| chapter.index == snapshot.chapter_index)
    {
        history_chapter.summary = updated_summary;
    }

    let order = next_apply_plan_order(&trace);
    trace.record_apply_plan(
        order,
        request.scope.as_str(),
        "manual_apply",
        priority_for_apply_scope(request.scope) + 75,
        Some(format!(
            "manual apply confirmed for chapter={} scope={}",
            snapshot.chapter_index,
            request.scope.as_str()
        )),
    );
    trace.record_apply_execution(
        request.scope.as_str(),
        manual_apply_outcome_for_scope(request.scope),
        Some(format!(
            "manual chapter summary hint applied for focus={}",
            proposal.focus
        )),
    );
    trace.record_proposal_transition(format!(
        "{}:manual_apply:chapter_summary_hint",
        request.request_id
    ));
    trace.set_apply_result(
        true,
        manual_apply_outcome_for_scope(request.scope),
        Some("manual chapter summary hint applied".to_string()),
    );

    Ok(NoNameReviewedApplyOutcome { trace, plot_state })
}

fn apply_option_bias_hint(
    mut trace: NoNameTrace,
    mut plot_state: PlotState,
    request: &NoNameReviewedApplyRequest,
    proposal: &NoNameProposal,
) -> Result<NoNameReviewedApplyOutcome, String> {
    let Some(snapshot) = &request.diagnostics_snapshot else {
        return Err("option_bias_hint manual apply requires a diagnostics snapshot".to_string());
    };
    if plot_state.current_chapter.index != snapshot.chapter_index {
        return Err(format!(
            "chapter mismatch: expected {}, current {}",
            snapshot.chapter_index, plot_state.current_chapter.index
        ));
    }
    if !plot_state.is_waiting_for_input {
        return Err("option_bias_hint manual apply requires waiting-for-input state".to_string());
    }

    let current_diagnostics = plot_state
        .last_generation_diagnostics
        .clone()
        .unwrap_or_default();
    if current_diagnostics != snapshot.expected_generation_diagnostics {
        return Err("diagnostics snapshot mismatch; refusing stale manual apply".to_string());
    }

    let hint = build_option_bias_hint(proposal);
    if current_diagnostics.contains(&hint) {
        return Err("diagnostics already contains this NoName option bias hint".to_string());
    }

    plot_state.last_generation_diagnostics = Some(if current_diagnostics.trim().is_empty() {
        hint.clone()
    } else {
        format!("{}; {}", current_diagnostics, hint)
    });

    let order = next_apply_plan_order(&trace);
    trace.record_apply_plan(
        order,
        request.scope.as_str(),
        "manual_apply",
        priority_for_apply_scope(request.scope) + 75,
        Some(format!(
            "manual apply confirmed for chapter={} scope={}",
            snapshot.chapter_index,
            request.scope.as_str()
        )),
    );
    trace.record_apply_execution(
        request.scope.as_str(),
        manual_apply_outcome_for_scope(request.scope),
        Some(format!(
            "manual option bias hint applied for focus={}",
            proposal.focus
        )),
    );
    trace.record_proposal_transition(format!(
        "{}:manual_apply:option_bias_hint",
        request.request_id
    ));
    trace.set_apply_result(
        true,
        manual_apply_outcome_for_scope(request.scope),
        Some("manual option bias hint applied".to_string()),
    );

    Ok(NoNameReviewedApplyOutcome { trace, plot_state })
}

pub fn apply_reviewed_output_to_plot_state(
    trace: NoNameTrace,
    request: NoNameReviewedApplyRequest,
    plot_state: PlotState,
) -> Result<NoNameReviewedApplyOutcome, String> {
    let proposal = validate_reviewed_apply_request(&trace, &request)?.clone();
    match request.scope {
        NoNameApplyScope::PlotTextHint => {
            apply_plot_text_hint_to_segment(trace, plot_state, &request, &proposal)
        }
        NoNameApplyScope::ChapterSummaryHint => {
            apply_chapter_summary_hint(trace, plot_state, &request, &proposal)
        }
        NoNameApplyScope::OptionBiasHint => {
            apply_option_bias_hint(trace, plot_state, &request, &proposal)
        }
        scope => Err(format!(
            "manual reviewed apply currently does not support {}",
            scope.as_str()
        )),
    }
}

pub fn apply_manual_plot_text_hint_to_plot_state(
    trace: NoNameTrace,
    request_id: &str,
    plot_state: PlotState,
    chapter_index: u32,
    segment_index: usize,
    expected_segment_text: &str,
) -> Result<NoNameReviewedApplyOutcome, String> {
    apply_reviewed_output_to_plot_state(
        trace,
        build_manual_plot_text_apply_request(
            request_id,
            chapter_index,
            segment_index,
            expected_segment_text,
        ),
        plot_state,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noname_output_interface::{
        NoNameControlledOutputDecision, NoNameControlledOutputKind, NoNameControlledOutputReview,
    };
    use crate::noname_types::{NoNameMode, NoNameProposalKind, NoNameProposalStatus, NoNameRole};
    use crate::plot_engine::Scene;

    fn make_plot_state(summary: &str) -> PlotState {
        let scene = Scene {
            id: "scene-1".to_string(),
            name: "test scene".to_string(),
            description: "test description".to_string(),
            location: "test".to_string(),
            available_options: Vec::new(),
        };
        let mut state = PlotState::new(scene);
        state.current_chapter.summary = summary.to_string();
        state
    }

    fn make_trace_for_summary_apply() -> (NoNameTrace, String) {
        let proposal_id = "proposal-summary".to_string();
        let request_id = "controlled-output-proposal-summary-chapter_summary_hint".to_string();
        let mut trace = NoNameTrace::empty("trace-1", "session-1", "turn-1", NoNameMode::Assisted);
        trace.record_proposal(NoNameProposal {
            proposal_id,
            kind: NoNameProposalKind::PlotCandidate,
            producer_role: NoNameRole::Director,
            title: "summary hint".to_string(),
            summary: "suggest summary hint".to_string(),
            focus: "sect crisis".to_string(),
            target_segment: NoNameTargetSegment::ChapterSummaryHead,
            intended_effect: "prepend chapter summary hint".to_string(),
            rationale: "keep recap focused".to_string(),
            suggested_action: Some("manual apply".to_string()),
            labels: vec!["test".to_string()],
            apply_scopes: vec![NoNameApplyScope::ChapterSummaryHint],
            status: NoNameProposalStatus::Applied,
            applyable: true,
        });
        trace.record_controlled_output_review(
            Some("proposal-summary".to_string()),
            NoNameControlledOutputKind::RecapNote,
            Vec::new(),
            NoNameControlledOutputReview {
                request_id: request_id.clone(),
                decision: NoNameControlledOutputDecision::NeedsReview,
                reason: "summary hint requires human review".to_string(),
                normalized_kind: Some(NoNameControlledOutputKind::RecapNote),
                safe_apply_scope: Some(NoNameApplyScope::ChapterSummaryHint),
                requires_human_review: true,
            },
        );
        trace.controlled_output_reviews[0].human_review_decision =
            Some(NoNameHumanReviewDecision::ApprovedForHigherApply);
        trace.record_apply_execution(
            NoNameApplyScope::ChapterSummaryHint.as_str(),
            "second_guardrail_allowed",
            None,
        );
        (trace, request_id)
    }

    fn make_trace_for_option_bias_apply() -> (NoNameTrace, String) {
        let proposal_id = "proposal-option-bias".to_string();
        let request_id = "controlled-output-proposal-option-bias-option_bias_hint".to_string();
        let mut trace = NoNameTrace::empty("trace-2", "session-1", "turn-2", NoNameMode::Assisted);
        trace.record_proposal(NoNameProposal {
            proposal_id,
            kind: NoNameProposalKind::PlotCandidate,
            producer_role: NoNameRole::Director,
            title: "option bias hint".to_string(),
            summary: "suggest option bias hint".to_string(),
            focus: "hidden cave".to_string(),
            target_segment: NoNameTargetSegment::CurrentTurnTail,
            intended_effect: "bias next player options".to_string(),
            rationale: "keep choices focused".to_string(),
            suggested_action: Some("manual apply".to_string()),
            labels: vec!["test".to_string()],
            apply_scopes: vec![NoNameApplyScope::OptionBiasHint],
            status: NoNameProposalStatus::Applied,
            applyable: true,
        });
        trace.record_controlled_output_review(
            Some("proposal-option-bias".to_string()),
            NoNameControlledOutputKind::IntermediateNarrativeHint,
            Vec::new(),
            NoNameControlledOutputReview {
                request_id: request_id.clone(),
                decision: NoNameControlledOutputDecision::NeedsReview,
                reason: "option bias hint requires human review".to_string(),
                normalized_kind: Some(NoNameControlledOutputKind::IntermediateNarrativeHint),
                safe_apply_scope: Some(NoNameApplyScope::OptionBiasHint),
                requires_human_review: true,
            },
        );
        trace.controlled_output_reviews[0].human_review_decision =
            Some(NoNameHumanReviewDecision::ApprovedForHigherApply);
        trace.record_apply_execution(
            NoNameApplyScope::OptionBiasHint.as_str(),
            "second_guardrail_allowed",
            None,
        );
        (trace, request_id)
    }

    #[test]
    fn reviewed_apply_can_write_chapter_summary_hint() {
        let (trace, request_id) = make_trace_for_summary_apply();
        let plot_state = make_plot_state("existing summary");

        let outcome = apply_reviewed_output_to_plot_state(
            trace,
            NoNameReviewedApplyRequest {
                request_id,
                scope: NoNameApplyScope::ChapterSummaryHint,
                segment_snapshot: None,
                summary_snapshot: Some(NoNameApplySummarySnapshot {
                    chapter_index: 1,
                    expected_summary: "existing summary".to_string(),
                }),
                diagnostics_snapshot: None,
            },
            plot_state,
        )
        .expect("summary hint should be manually applied");

        assert!(outcome
            .plot_state
            .current_chapter
            .summary
            .starts_with("NoName summary hint: sect crisis; existing summary"));
        assert!(outcome.trace.apply_execution_log.iter().any(|entry| {
            entry.target == "chapter_summary_hint"
                && entry.outcome == "manual_chapter_summary_hint_applied"
        }));
        assert!(outcome
            .trace
            .proposal_transition_log
            .iter()
            .any(|entry| entry.ends_with(":manual_apply:chapter_summary_hint")));
    }

    #[test]
    fn reviewed_apply_rejects_stale_chapter_summary_snapshot() {
        let (trace, request_id) = make_trace_for_summary_apply();
        let plot_state = make_plot_state("newer summary");

        let error = apply_reviewed_output_to_plot_state(
            trace,
            NoNameReviewedApplyRequest {
                request_id,
                scope: NoNameApplyScope::ChapterSummaryHint,
                segment_snapshot: None,
                summary_snapshot: Some(NoNameApplySummarySnapshot {
                    chapter_index: 1,
                    expected_summary: "older summary".to_string(),
                }),
                diagnostics_snapshot: None,
            },
            plot_state,
        )
        .expect_err("stale summary should be rejected");

        assert!(error.contains("summary snapshot mismatch"));
    }

    #[test]
    fn reviewed_apply_can_write_option_bias_hint() {
        let (trace, request_id) = make_trace_for_option_bias_apply();
        let mut plot_state = make_plot_state("");
        plot_state.last_generation_diagnostics = Some("base diagnostics".to_string());

        let outcome = apply_reviewed_output_to_plot_state(
            trace,
            NoNameReviewedApplyRequest {
                request_id,
                scope: NoNameApplyScope::OptionBiasHint,
                segment_snapshot: None,
                summary_snapshot: None,
                diagnostics_snapshot: Some(NoNameApplyDiagnosticsSnapshot {
                    chapter_index: 1,
                    expected_generation_diagnostics: "base diagnostics".to_string(),
                }),
            },
            plot_state,
        )
        .expect("option bias hint should be manually applied");

        assert_eq!(
            outcome.plot_state.last_generation_diagnostics.as_deref(),
            Some(
                "base diagnostics; NoName option bias: next turn should prioritize actions around hidden cave"
            )
        );
        assert!(outcome.trace.apply_execution_log.iter().any(|entry| {
            entry.target == "option_bias_hint" && entry.outcome == "manual_option_bias_hint_applied"
        }));
        assert!(outcome
            .trace
            .proposal_transition_log
            .iter()
            .any(|entry| entry.ends_with(":manual_apply:option_bias_hint")));
    }

    #[test]
    fn reviewed_apply_rejects_stale_option_bias_snapshot() {
        let (trace, request_id) = make_trace_for_option_bias_apply();
        let mut plot_state = make_plot_state("");
        plot_state.last_generation_diagnostics = Some("newer diagnostics".to_string());

        let error = apply_reviewed_output_to_plot_state(
            trace,
            NoNameReviewedApplyRequest {
                request_id,
                scope: NoNameApplyScope::OptionBiasHint,
                segment_snapshot: None,
                summary_snapshot: None,
                diagnostics_snapshot: Some(NoNameApplyDiagnosticsSnapshot {
                    chapter_index: 1,
                    expected_generation_diagnostics: "older diagnostics".to_string(),
                }),
            },
            plot_state,
        )
        .expect_err("stale diagnostics should be rejected");

        assert!(error.contains("diagnostics snapshot mismatch"));
    }

    #[test]
    fn reviewed_apply_rejects_when_reviewed_proposal_binding_is_missing() {
        let (mut trace, request_id) = make_trace_for_summary_apply();
        trace.controlled_output_reviews[0].proposal_id = Some("proposal-missing".to_string());
        let plot_state = make_plot_state("existing summary");

        let error = apply_reviewed_output_to_plot_state(
            trace,
            NoNameReviewedApplyRequest {
                request_id,
                scope: NoNameApplyScope::ChapterSummaryHint,
                segment_snapshot: None,
                summary_snapshot: Some(NoNameApplySummarySnapshot {
                    chapter_index: 1,
                    expected_summary: "existing summary".to_string(),
                }),
                diagnostics_snapshot: None,
            },
            plot_state,
        )
        .expect_err("missing reviewed proposal binding should be rejected");

        assert!(error.contains("NoName proposal not found"));
    }
}
