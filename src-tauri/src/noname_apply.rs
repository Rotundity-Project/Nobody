use crate::noname_output_interface::NoNameControlledOutputDecision;
use crate::noname_trace::{NoNameHumanReviewDecision, NoNameSecondGuardrailDecision, NoNameTrace};
use crate::noname_types::{
    NoNameApplyScope, NoNameMode, NoNameProposal, NoNameProposalStatus, NoNameTargetSegment,
    NoNameTraceStage,
};
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
pub struct NoNameApplyPlotAugmentationSnapshot {
    pub chapter_index: u32,
    pub expected_plot_augmentation_hints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoNameReviewedApplyRequest {
    pub request_id: String,
    pub scope: NoNameApplyScope,
    pub segment_snapshot: Option<NoNameApplySegmentSnapshot>,
    pub summary_snapshot: Option<NoNameApplySummarySnapshot>,
    pub diagnostics_snapshot: Option<NoNameApplyDiagnosticsSnapshot>,
    pub plot_augmentation_snapshot: Option<NoNameApplyPlotAugmentationSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoNameReviewedApplyRequestInput {
    pub request_id: String,
    pub scope: NoNameApplyScope,
    pub chapter_index: Option<u32>,
    pub segment_index: Option<usize>,
    pub expected_segment_text: Option<String>,
    pub expected_summary: Option<String>,
    pub expected_generation_diagnostics: Option<String>,
    pub expected_plot_augmentation_hints: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoNameReviewedApplyOutcome {
    pub trace: NoNameTrace,
    pub plot_state: PlotState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoNameApplyTargetDecision {
    Apply,
    Skip { outcome: &'static str, note: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoNameApplyTargetPlan {
    pub target: &'static str,
    pub priority: u32,
    pub order: u32,
    pub decision: NoNameApplyTargetDecision,
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
        plot_augmentation_snapshot: None,
    }
}

pub fn build_reviewed_apply_request(
    input: NoNameReviewedApplyRequestInput,
) -> Result<NoNameReviewedApplyRequest, String> {
    let NoNameReviewedApplyRequestInput {
        request_id,
        scope,
        chapter_index,
        segment_index,
        expected_segment_text,
        expected_summary,
        expected_generation_diagnostics,
        expected_plot_augmentation_hints,
    } = input;

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
    let plot_augmentation_snapshot = match scope {
        NoNameApplyScope::PlotAugmentationHint => Some(NoNameApplyPlotAugmentationSnapshot {
            chapter_index: chapter_index.ok_or_else(|| {
                "plot_augmentation_hint manual apply requires chapter_index".to_string()
            })?,
            expected_plot_augmentation_hints: expected_plot_augmentation_hints.ok_or_else(
                || {
                    "plot_augmentation_hint manual apply requires expected_plot_augmentation_hints"
                        .to_string()
                },
            )?,
        }),
        _ => None,
    };

    Ok(NoNameReviewedApplyRequest {
        request_id,
        scope,
        segment_snapshot,
        summary_snapshot,
        diagnostics_snapshot,
        plot_augmentation_snapshot,
    })
}

fn priority_for_apply_scope(scope: NoNameApplyScope) -> u32 {
    match scope {
        NoNameApplyScope::PlotTextHint => 300,
        NoNameApplyScope::PlotAugmentationHint => 250,
        NoNameApplyScope::ChapterSummaryHint => 200,
        NoNameApplyScope::OptionBiasHint => 100,
        NoNameApplyScope::Diagnostics => 50,
    }
}

fn build_noname_summary_hint(proposal: &NoNameProposal) -> String {
    format!("NoName提示：后续重点关注{}", proposal.focus.trim())
}

fn build_noname_option_bias_hint(proposal: &NoNameProposal) -> String {
    format!(
        "NoName选项偏置：下轮优先围绕{}提供行动切入点",
        proposal.focus.trim()
    )
}

fn build_noname_plot_text_hint(proposal: &NoNameProposal) -> String {
    format!("【NoName】重点关注：{}", proposal.focus.trim())
}

fn proposal_allows_apply_scope(proposal: &NoNameProposal, scope: NoNameApplyScope) -> bool {
    proposal.apply_scopes.is_empty() || proposal.apply_scopes.contains(&scope)
}

pub fn plan_noname_apply_target(
    proposal: &NoNameProposal,
    scope: NoNameApplyScope,
    plot_state: Option<&PlotState>,
    plot_text: Option<&str>,
) -> NoNameApplyTargetPlan {
    let target = scope.as_str();
    let priority = priority_for_apply_scope(scope);
    if !proposal_allows_apply_scope(proposal, scope) {
        return NoNameApplyTargetPlan {
            target,
            priority,
            order: 0,
            decision: NoNameApplyTargetDecision::Skip {
                outcome: "skipped_scope_forbidden",
                note: format!("提案未声明 {} 作用域，跳过对应输出", target),
            },
        };
    }

    if !target_segment_supports_apply_scope(proposal.target_segment, scope) {
        return NoNameApplyTargetPlan {
            target,
            priority,
            order: 0,
            decision: NoNameApplyTargetDecision::Skip {
                outcome: "skipped_target_mismatch",
                note: format!(
                    "目标段 {} 不支持 {}，跳过对应输出",
                    proposal.target_segment.as_str(),
                    target
                ),
            },
        };
    }

    match scope {
        NoNameApplyScope::PlotTextHint => {
            let current_text = plot_text.unwrap_or_default();
            if current_text.trim().is_empty() {
                return NoNameApplyTargetPlan {
                    target,
                    priority,
                    order: 0,
                    decision: NoNameApplyTargetDecision::Skip {
                        outcome: "skipped_empty_plot_text",
                        note: "当前正文为空，跳过受控正文提示".to_string(),
                    },
                };
            }
            if current_text.contains("【NoName】") || current_text.contains("NoName提示") {
                return NoNameApplyTargetPlan {
                    target,
                    priority,
                    order: 0,
                    decision: NoNameApplyTargetDecision::Skip {
                        outcome: "skipped_duplicate",
                        note: "正文已包含 NoName 标记，跳过重复提示".to_string(),
                    },
                };
            }
        }
        NoNameApplyScope::PlotAugmentationHint => {
            let Some(state) = plot_state else {
                return NoNameApplyTargetPlan {
                    target,
                    priority,
                    order: 0,
                    decision: NoNameApplyTargetDecision::Skip {
                        outcome: "skipped_missing_plot_state",
                        note: "缺少 plot_state，跳过剧情增强提示".to_string(),
                    },
                };
            };
            if !state.is_waiting_for_input {
                return NoNameApplyTargetPlan {
                    target,
                    priority,
                    order: 0,
                    decision: NoNameApplyTargetDecision::Skip {
                        outcome: "skipped_not_waiting",
                        note: "当前不处于等待输入状态，跳过剧情增强提示".to_string(),
                    },
                };
            }
            let hint = format!(
                "NoName plot augmentation: focus={} | effect={}",
                proposal.focus.trim(),
                proposal.intended_effect.trim()
            );
            if state
                .pending_plot_augmentation_hints
                .iter()
                .any(|existing| existing == &hint)
            {
                return NoNameApplyTargetPlan {
                    target,
                    priority,
                    order: 0,
                    decision: NoNameApplyTargetDecision::Skip {
                        outcome: "skipped_duplicate",
                        note: format!(
                            "剧情增强提示已包含“{}”，跳过重复写入",
                            proposal.focus.trim()
                        ),
                    },
                };
            }
        }
        NoNameApplyScope::ChapterSummaryHint => {
            if let Some(state) = plot_state {
                let summary = state.current_chapter.summary.trim();
                if !summary.is_empty() && summary.contains(proposal.focus.trim()) {
                    return NoNameApplyTargetPlan {
                        target,
                        priority,
                        order: 0,
                        decision: NoNameApplyTargetDecision::Skip {
                            outcome: "skipped_duplicate",
                            note: format!(
                                "章节摘要已覆盖“{}”，跳过重复提示",
                                proposal.focus.trim()
                            ),
                        },
                    };
                }
            }
        }
        NoNameApplyScope::OptionBiasHint => {
            let Some(state) = plot_state else {
                return NoNameApplyTargetPlan {
                    target,
                    priority,
                    order: 0,
                    decision: NoNameApplyTargetDecision::Skip {
                        outcome: "skipped_missing_plot_state",
                        note: "缺少 plot_state，跳过选项偏置提示".to_string(),
                    },
                };
            };
            if !state.is_waiting_for_input {
                return NoNameApplyTargetPlan {
                    target,
                    priority,
                    order: 0,
                    decision: NoNameApplyTargetDecision::Skip {
                        outcome: "skipped_not_waiting",
                        note: "当前不处于等待输入状态，跳过选项偏置提示".to_string(),
                    },
                };
            }
            let option_hint = build_noname_option_bias_hint(proposal);
            let diagnostics = state
                .last_generation_diagnostics
                .as_deref()
                .unwrap_or_default();
            if diagnostics.contains(option_hint.as_str()) {
                return NoNameApplyTargetPlan {
                    target,
                    priority,
                    order: 0,
                    decision: NoNameApplyTargetDecision::Skip {
                        outcome: "skipped_duplicate",
                        note: format!(
                            "诊断中已包含选项偏置提示“{}”，跳过重复写入",
                            proposal.focus.trim()
                        ),
                    },
                };
            }
        }
        NoNameApplyScope::Diagnostics => {}
    }

    NoNameApplyTargetPlan {
        target,
        priority,
        order: 0,
        decision: NoNameApplyTargetDecision::Apply,
    }
}

pub fn build_noname_apply_plan_set(
    proposal: &NoNameProposal,
    plot_state: Option<&PlotState>,
    plot_text: Option<&str>,
) -> Vec<NoNameApplyTargetPlan> {
    let mut plans = vec![
        plan_noname_apply_target(
            proposal,
            NoNameApplyScope::PlotTextHint,
            plot_state,
            plot_text,
        ),
        plan_noname_apply_target(
            proposal,
            NoNameApplyScope::PlotAugmentationHint,
            plot_state,
            None,
        ),
        plan_noname_apply_target(
            proposal,
            NoNameApplyScope::ChapterSummaryHint,
            plot_state,
            None,
        ),
        plan_noname_apply_target(proposal, NoNameApplyScope::OptionBiasHint, plot_state, None),
    ];
    plans.sort_by(|left, right| {
        right
            .priority
            .cmp(&left.priority)
            .then_with(|| left.target.cmp(right.target))
    });
    for (index, plan) in plans.iter_mut().enumerate() {
        plan.order = (index + 1) as u32;
    }
    plans
}

pub fn record_noname_apply_plan(trace: &mut NoNameTrace, plan: &NoNameApplyTargetPlan) {
    let (decision, note) = match &plan.decision {
        NoNameApplyTargetDecision::Apply => (
            "apply",
            Some(format!(
                "允许执行 {}，优先级 {}，顺位 #{}",
                plan.target, plan.priority, plan.order
            )),
        ),
        NoNameApplyTargetDecision::Skip { note, .. } => ("skip", Some(note.clone())),
    };
    trace.record_apply_plan(plan.order, plan.target, decision, plan.priority, note);
}

fn record_noname_apply_plan_and_skip_execution(
    trace: &mut NoNameTrace,
    plan: &NoNameApplyTargetPlan,
) {
    record_noname_apply_plan(trace, plan);
    if let NoNameApplyTargetDecision::Skip { outcome, note } = &plan.decision {
        trace.record_apply_execution(plan.target, *outcome, Some(note.clone()));
    }
}

pub fn apply_noname_plot_text_hint(
    plot_text: &mut String,
    trace: &mut NoNameTrace,
    proposal: &NoNameProposal,
) -> bool {
    if proposal.status != NoNameProposalStatus::Applied {
        return false;
    }

    let plan = plan_noname_apply_target(
        proposal,
        NoNameApplyScope::PlotTextHint,
        None,
        Some(plot_text.as_str()),
    );
    if !matches!(plan.decision, NoNameApplyTargetDecision::Apply) {
        if let NoNameApplyTargetDecision::Skip { outcome, note } = &plan.decision {
            trace.record_apply_execution(plan.target, *outcome, Some(note.clone()));
        }
        return false;
    }

    let hint = build_noname_plot_text_hint(proposal);
    match proposal.target_segment {
        NoNameTargetSegment::CurrentTurnHead => {
            let original = plot_text.trim().to_string();
            *plot_text = format!("{}\n\n{}", hint, original);
        }
        _ => {
            plot_text.push_str("\n\n");
            plot_text.push_str(&hint);
        }
    }
    trace.record_proposal_transition(format!("{}:applied:plot_text_hint", proposal.proposal_id));
    trace.record_apply_execution(
        "plot_text_hint",
        "applied",
        Some(format!("已将提案提示插入正文，聚焦“{}”", proposal.focus)),
    );
    true
}

pub fn apply_noname_low_risk_outputs(
    plot_state: &mut PlotState,
    trace: &mut NoNameTrace,
    proposal: &NoNameProposal,
    plot_text_applied: bool,
) {
    if proposal.status != NoNameProposalStatus::Applied {
        return;
    }

    let mut applied_targets: Vec<&str> = Vec::new();

    let summary_plan = plan_noname_apply_target(
        proposal,
        NoNameApplyScope::ChapterSummaryHint,
        Some(plot_state),
        None,
    );
    if matches!(summary_plan.decision, NoNameApplyTargetDecision::Apply) {
        let hint = build_noname_summary_hint(proposal);
        let summary = plot_state.current_chapter.summary.trim();
        if summary.is_empty() {
            plot_state.current_chapter.summary = hint.clone();
        } else {
            plot_state.current_chapter.summary = match proposal.target_segment {
                NoNameTargetSegment::ChapterSummaryHead => format!("{}；{}", hint, summary),
                _ => format!("{}；{}", summary, hint),
            };
        }
        trace.record_proposal_transition(format!(
            "{}:applied:chapter_summary_hint",
            proposal.proposal_id
        ));
        trace.record_apply_execution(
            "chapter_summary_hint",
            "applied",
            Some(format!("已写入章节摘要提示，聚焦“{}”", proposal.focus)),
        );
        applied_targets.push("chapter_summary_hint");
    } else {
        record_noname_apply_plan_and_skip_execution(trace, &summary_plan);
    }

    let option_bias_plan = plan_noname_apply_target(
        proposal,
        NoNameApplyScope::OptionBiasHint,
        Some(plot_state),
        None,
    );
    if matches!(option_bias_plan.decision, NoNameApplyTargetDecision::Apply) {
        let option_hint = build_noname_option_bias_hint(proposal);
        let diagnostics = plot_state
            .last_generation_diagnostics
            .clone()
            .unwrap_or_default();
        if diagnostics.is_empty() {
            plot_state.last_generation_diagnostics = Some(option_hint.clone());
        } else if !diagnostics.contains(option_hint.as_str()) {
            plot_state.last_generation_diagnostics =
                Some(format!("{}；{}", diagnostics, option_hint));
        }
        trace.record_proposal_transition(format!(
            "{}:applied:option_bias_hint",
            proposal.proposal_id
        ));
        trace.record_apply_execution(
            "option_bias_hint",
            "applied",
            Some(format!("已写入下轮选项偏置提示，聚焦“{}”", proposal.focus)),
        );
        applied_targets.push("option_bias_hint");
    } else {
        record_noname_apply_plan_and_skip_execution(trace, &option_bias_plan);
    }

    if plot_text_applied {
        applied_targets.push("plot_text_hint");
    }
    let apply_outcome = if applied_targets.is_empty() {
        "applied_no_scoped_output"
    } else {
        "applied_scoped_outputs"
    };
    trace.set_apply_result(
        true,
        apply_outcome,
        Some(format!(
            "已应用作用域：{}；聚焦“{}”",
            if applied_targets.is_empty() {
                "无".to_string()
            } else {
                applied_targets.join(", ")
            },
            proposal.focus
        )),
    );
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

fn find_controlled_output_review_index(trace: &NoNameTrace, request_id: &str) -> Option<usize> {
    trace
        .controlled_output_reviews
        .iter()
        .position(|review| review.request_id == request_id)
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
        NoNameApplyScope::PlotTextHint | NoNameApplyScope::PlotAugmentationHint => matches!(
            target_segment,
            NoNameTargetSegment::CurrentTurnHead | NoNameTargetSegment::CurrentTurnTail
        ),
        _ => true,
    }
}

fn review_is_waiting_for_second_guardrail(
    trace: &NoNameTrace,
    request_id: &str,
    scope: NoNameApplyScope,
) -> bool {
    let transition = format!("{}:apply_intent:awaiting_second_guardrail", request_id);
    trace
        .proposal_transition_log
        .iter()
        .any(|entry| entry == &transition)
        || trace.apply_execution_log.iter().any(|entry| {
            entry.outcome == "awaiting_second_guardrail" && entry.target == scope.as_str()
        })
}

fn second_guardrail_revalidation_reason(
    trace: &NoNameTrace,
    request_id: &str,
    safe_apply_scope: Option<NoNameApplyScope>,
) -> Option<String> {
    if trace.mode != NoNameMode::Assisted {
        return Some("trace is not assisted mode; second guardrail rejected".to_string());
    }
    let Some(scope) = safe_apply_scope else {
        return Some("second guardrail missing safe_apply_scope".to_string());
    };
    if !matches!(
        scope,
        NoNameApplyScope::PlotTextHint | NoNameApplyScope::PlotAugmentationHint
    ) {
        return Some(
            "only plot_text_hint / plot_augmentation_hint can enter second guardrail".to_string(),
        );
    }
    if !review_is_waiting_for_second_guardrail(trace, request_id, scope) {
        return Some("review has not entered awaiting_second_guardrail".to_string());
    }

    match find_review_proposal(trace, request_id) {
        Some(proposal)
            if proposal.status == NoNameProposalStatus::Applied
                && proposal.applyable
                && target_segment_supports_apply_scope(proposal.target_segment, scope) =>
        {
            None
        }
        Some(proposal)
            if proposal.status != NoNameProposalStatus::Applied || !proposal.applyable =>
        {
            Some(format!(
                "proposal status is {} / applyable={}; second guardrail rejected",
                proposal.status.as_str(),
                proposal.applyable
            ))
        }
        Some(proposal) => Some(format!(
            "target_segment={} does not support {} second guardrail",
            proposal.target_segment.as_str(),
            scope.as_str()
        )),
        None => Some("NoName proposal not found for second guardrail".to_string()),
    }
}

fn human_review_apply_intent_rejection_reason(
    trace: &NoNameTrace,
    request_id: &str,
    safe_apply_scope: Option<NoNameApplyScope>,
    target: &str,
) -> Option<String> {
    if trace.mode != NoNameMode::Assisted {
        return Some("trace is not assisted mode; apply intent rejected".to_string());
    }
    let Some(scope) = safe_apply_scope else {
        return Some(format!(
            "safe_apply_scope is required for apply intent: {}",
            target
        ));
    };
    if !matches!(
        scope,
        NoNameApplyScope::PlotTextHint | NoNameApplyScope::PlotAugmentationHint
    ) {
        return Some(format!(
            "only plot_text_hint / plot_augmentation_hint can enter second guardrail, got {}",
            target
        ));
    }

    match find_review_proposal(trace, request_id) {
        Some(proposal)
            if proposal.status == NoNameProposalStatus::Applied
                && proposal.applyable
                && target_segment_supports_apply_scope(proposal.target_segment, scope) =>
        {
            None
        }
        Some(proposal)
            if proposal.status != NoNameProposalStatus::Applied || !proposal.applyable =>
        {
            Some(format!(
                "proposal status is {} / applyable={}; apply intent rejected",
                proposal.status.as_str(),
                proposal.applyable
            ))
        }
        Some(proposal) => Some(format!(
            "target_segment={} does not support {} second apply",
            proposal.target_segment.as_str(),
            scope.as_str()
        )),
        None => Some("NoName proposal not found for apply intent".to_string()),
    }
}

pub fn apply_human_review_decision_to_trace(
    trace: &mut NoNameTrace,
    request_id: &str,
    decision: NoNameHumanReviewDecision,
    reviewed_at: u64,
) -> Result<(), String> {
    let review_index = find_controlled_output_review_index(trace, request_id)
        .ok_or_else(|| format!("NoName controlled output review not found: {}", request_id))?;

    let safe_apply_scope = {
        let review = &trace.controlled_output_reviews[review_index];
        if review.decision != NoNameControlledOutputDecision::NeedsReview
            || !review.requires_human_review
        {
            return Err(format!(
                "NoName controlled output review does not require human review: {}",
                request_id
            ));
        }

        review.safe_apply_scope.ok_or_else(|| {
            format!(
                "NoName controlled output review has no safe apply scope: {}",
                request_id
            )
        })?
    };

    let review = &mut trace.controlled_output_reviews[review_index];
    review.human_review_decision = Some(decision);
    review.human_reviewed_at = Some(reviewed_at);
    review.human_review_note = Some(decision.note().to_string());

    trace.record_proposal_transition(format!("{}:human_review:{}", request_id, decision.as_str()));
    record_human_review_apply_intent(trace, request_id, decision, Some(safe_apply_scope));

    Ok(())
}

pub fn record_human_review_apply_intent(
    trace: &mut NoNameTrace,
    request_id: &str,
    decision: NoNameHumanReviewDecision,
    safe_apply_scope: Option<NoNameApplyScope>,
) {
    let target = safe_apply_scope
        .map(|scope| scope.as_str())
        .unwrap_or("controlled_output");
    let order = next_apply_plan_order(trace);
    let priority = safe_apply_scope.map(priority_for_apply_scope).unwrap_or(10) + 25;

    match decision {
        NoNameHumanReviewDecision::Pending => {
            trace.record_apply_plan(
                order,
                target,
                "review_intent_pending",
                priority,
                Some("人工复核已重置为待确认，未进入二次 guardrail".to_string()),
            );
            trace.record_apply_execution(
                target,
                "human_review_pending",
                Some("等待人工确认，不触发高层 apply".to_string()),
            );
            trace.record_proposal_transition(format!("{}:apply_intent:pending", request_id));
        }
        NoNameHumanReviewDecision::RejectedForHigherApply => {
            trace.record_apply_plan(
                order,
                target,
                "reject",
                priority,
                Some("人工复核拒绝进入高层 apply，保持安全边界".to_string()),
            );
            trace.record_apply_execution(
                target,
                "rejected_by_human_review",
                Some("开发者选择暂不应用，未触发二次 guardrail".to_string()),
            );
            trace.record_proposal_transition(format!(
                "{}:apply_intent:rejected_by_human_review",
                request_id
            ));
        }
        NoNameHumanReviewDecision::ApprovedForHigherApply => {
            if let Some(reason) = human_review_apply_intent_rejection_reason(
                trace,
                request_id,
                safe_apply_scope,
                target,
            ) {
                trace.record_apply_plan(
                    order,
                    target,
                    "second_guardrail_reject",
                    priority,
                    Some(reason.clone()),
                );
                trace.record_apply_execution(target, "second_guardrail_rejected", Some(reason));
                trace.record_proposal_transition(format!(
                    "{}:apply_intent:second_guardrail_rejected",
                    request_id
                ));
                return;
            }

            trace.record_apply_plan(
                order,
                target,
                "review_intent_ready",
                priority,
                Some(
                    "人工确认已通过，已排入二次 guardrail / apply planner，未写入正文".to_string(),
                ),
            );
            trace.record_apply_execution(
                target,
                "awaiting_second_guardrail",
                Some("高层 apply intent 已记录，等待后续二次 guardrail 决策".to_string()),
            );
            trace.record_proposal_transition(format!(
                "{}:apply_intent:awaiting_second_guardrail",
                request_id
            ));
        }
    }
}

pub fn record_second_guardrail_decision(
    trace: &mut NoNameTrace,
    request_id: &str,
    decision: NoNameSecondGuardrailDecision,
    safe_apply_scope: Option<NoNameApplyScope>,
) -> Result<(), String> {
    let target = safe_apply_scope
        .map(|scope| scope.as_str())
        .unwrap_or("controlled_output");
    let order = next_apply_plan_order(trace);
    let priority = safe_apply_scope.map(priority_for_apply_scope).unwrap_or(10) + 50;
    let revalidation_reason =
        second_guardrail_revalidation_reason(trace, request_id, safe_apply_scope);

    if let Some(reason) = revalidation_reason {
        trace.record_apply_plan(
            order,
            target,
            "second_guardrail_reject",
            priority,
            Some(reason.clone()),
        );
        trace.record_apply_execution(target, "second_guardrail_rejected", Some(reason.clone()));
        trace.record_proposal_transition(format!("{}:second_guardrail:reject", request_id));
        trace.set_apply_result(true, "second_guardrail_reject", Some(reason));
        return Ok(());
    }

    match decision {
        NoNameSecondGuardrailDecision::Allow => {
            trace.record_apply_plan(
                order,
                target,
                "second_guardrail_allow",
                priority,
                Some("二次 guardrail 允许进入后续人工 apply 命令；当前不写正文".to_string()),
            );
            trace.record_apply_execution(
                target,
                "second_guardrail_allowed",
                Some("已允许进入下一步人工 apply，但未改写剧情正文".to_string()),
            );
            trace.record_proposal_transition(format!("{}:second_guardrail:allow", request_id));
            trace.set_apply_result(
                true,
                "second_guardrail_allow",
                Some("二次 guardrail 已允许；等待显式人工 apply 命令".to_string()),
            );
        }
        NoNameSecondGuardrailDecision::Reject => {
            trace.record_apply_plan(
                order,
                target,
                "second_guardrail_reject",
                priority,
                Some("二次 guardrail 人工拒绝高层 apply".to_string()),
            );
            trace.record_apply_execution(
                target,
                "second_guardrail_rejected",
                Some("二次 guardrail 决策为拒绝，未改写剧情正文".to_string()),
            );
            trace.record_proposal_transition(format!("{}:second_guardrail:reject", request_id));
            trace.set_apply_result(
                true,
                "second_guardrail_reject",
                Some("二次 guardrail 已拒绝高层 apply".to_string()),
            );
        }
        NoNameSecondGuardrailDecision::Fallback => {
            trace.push_stage(NoNameTraceStage::ApplyFallback);
            trace.fallback_used = true;
            trace.record_apply_plan(
                order,
                target,
                "second_guardrail_fallback",
                priority,
                Some("二次 guardrail 要求回退经典链路".to_string()),
            );
            trace.record_apply_execution(
                target,
                "second_guardrail_fallback",
                Some("已记录回退决策，未改写剧情正文".to_string()),
            );
            trace.record_proposal_transition(format!("{}:second_guardrail:fallback", request_id));
            trace.set_apply_result(
                true,
                "second_guardrail_fallback",
                Some("二次 guardrail 决策为 fallback，继续依赖经典链路".to_string()),
            );
        }
    }

    Ok(())
}

pub fn resolve_second_guardrail_for_trace(
    trace: &mut NoNameTrace,
    request_id: &str,
    decision: NoNameSecondGuardrailDecision,
) -> Result<(), String> {
    let safe_apply_scope = trace
        .controlled_output_reviews
        .iter()
        .find(|review| review.request_id == request_id)
        .ok_or_else(|| format!("NoName controlled output review not found: {}", request_id))
        .and_then(|review| {
            if review.human_review_decision
                != Some(NoNameHumanReviewDecision::ApprovedForHigherApply)
            {
                return Err(format!(
                    "NoName controlled output review is not approved for second guardrail: {}",
                    request_id
                ));
            }

            Ok(review.safe_apply_scope)
        })?;

    record_second_guardrail_decision(trace, request_id, decision, safe_apply_scope)
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

fn build_plot_augmentation_hint(proposal: &NoNameProposal) -> String {
    format!(
        "NoName plot augmentation: focus={} | effect={}",
        proposal.focus.trim(),
        proposal.intended_effect.trim()
    )
}

fn manual_apply_outcome_for_scope(scope: NoNameApplyScope) -> &'static str {
    match scope {
        NoNameApplyScope::PlotTextHint => "manual_plot_text_applied",
        NoNameApplyScope::PlotAugmentationHint => "manual_plot_augmentation_hint_applied",
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

    plot_state.apply_low_risk_chapter_summary_hint(
        snapshot.chapter_index,
        &snapshot.expected_summary,
        &hint,
        proposal.target_segment == NoNameTargetSegment::ChapterSummaryHead,
    )?;

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
    let current_diagnostics = plot_state
        .last_generation_diagnostics
        .clone()
        .unwrap_or_default();

    let hint = build_option_bias_hint(proposal);
    if current_diagnostics.contains(&hint) {
        return Err("diagnostics already contains this NoName option bias hint".to_string());
    }

    plot_state.apply_low_risk_generation_diagnostics_hint(
        snapshot.chapter_index,
        &snapshot.expected_generation_diagnostics,
        &hint,
    )?;

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

fn apply_plot_augmentation_hint(
    mut trace: NoNameTrace,
    mut plot_state: PlotState,
    request: &NoNameReviewedApplyRequest,
    proposal: &NoNameProposal,
) -> Result<NoNameReviewedApplyOutcome, String> {
    let Some(snapshot) = &request.plot_augmentation_snapshot else {
        return Err(
            "plot_augmentation_hint manual apply requires a plot augmentation snapshot".to_string(),
        );
    };

    let hint = build_plot_augmentation_hint(proposal);
    if snapshot
        .expected_plot_augmentation_hints
        .iter()
        .any(|existing| existing == &hint)
    {
        return Err("plot augmentation already contains this NoName hint".to_string());
    }

    plot_state.apply_pending_plot_augmentation_hint(
        snapshot.chapter_index,
        &snapshot.expected_plot_augmentation_hints,
        &hint,
    )?;

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
            "manual plot augmentation hint staged for focus={}",
            proposal.focus
        )),
    );
    trace.record_proposal_transition(format!(
        "{}:manual_apply:plot_augmentation_hint",
        request.request_id
    ));
    trace.set_apply_result(
        true,
        manual_apply_outcome_for_scope(request.scope),
        Some("manual plot augmentation hint staged".to_string()),
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
        NoNameApplyScope::PlotAugmentationHint => {
            apply_plot_augmentation_hint(trace, plot_state, &request, &proposal)
        }
        scope => Err(format!(
            "manual reviewed apply currently does not support {}",
            scope.as_str()
        )),
    }
}

pub fn apply_reviewed_output_input_to_plot_state(
    trace: NoNameTrace,
    input: NoNameReviewedApplyRequestInput,
    plot_state: PlotState,
) -> Result<NoNameReviewedApplyOutcome, String> {
    let request = build_reviewed_apply_request(input)?;
    apply_reviewed_output_to_plot_state(trace, request, plot_state)
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

    fn make_trace_for_plot_augmentation_apply() -> (NoNameTrace, String) {
        let proposal_id = "proposal-plot-augmentation".to_string();
        let request_id =
            "controlled-output-proposal-plot-augmentation-plot_augmentation_hint".to_string();
        let mut trace = NoNameTrace::empty("trace-3", "session-1", "turn-3", NoNameMode::Assisted);
        trace.record_proposal(NoNameProposal {
            proposal_id: proposal_id.clone(),
            kind: NoNameProposalKind::PlotCandidate,
            producer_role: NoNameRole::Director,
            title: "plot augmentation hint".to_string(),
            summary: "suggest non-final plot augmentation".to_string(),
            focus: "hidden cave".to_string(),
            target_segment: NoNameTargetSegment::CurrentTurnTail,
            intended_effect: "stage a reversible narrative beat".to_string(),
            rationale: "keep plot augmentation outside final text".to_string(),
            suggested_action: Some("manual apply".to_string()),
            labels: vec!["test".to_string()],
            apply_scopes: vec![NoNameApplyScope::PlotAugmentationHint],
            status: NoNameProposalStatus::Applied,
            applyable: true,
        });
        trace.record_controlled_output_review(
            Some(proposal_id.clone()),
            NoNameControlledOutputKind::NonFinalPlotAugmentation,
            Vec::new(),
            NoNameControlledOutputReview {
                request_id: request_id.clone(),
                decision: NoNameControlledOutputDecision::NeedsReview,
                reason: "plot augmentation hint requires human review".to_string(),
                normalized_kind: Some(NoNameControlledOutputKind::NonFinalPlotAugmentation),
                safe_apply_scope: Some(NoNameApplyScope::PlotAugmentationHint),
                requires_human_review: true,
            },
        );
        trace.controlled_output_reviews[0].human_review_decision =
            Some(NoNameHumanReviewDecision::ApprovedForHigherApply);
        trace.record_apply_execution(
            NoNameApplyScope::PlotAugmentationHint.as_str(),
            "second_guardrail_allowed",
            None,
        );
        (trace, request_id)
    }

    fn make_trace_for_plot_text_review() -> (NoNameTrace, String) {
        let proposal_id = "proposal-plot-text".to_string();
        let request_id = "controlled-output-proposal-plot-text-plot_text_hint".to_string();
        let mut trace = NoNameTrace::empty("trace-4", "session-1", "turn-4", NoNameMode::Assisted);
        trace.record_proposal(NoNameProposal {
            proposal_id: proposal_id.clone(),
            kind: NoNameProposalKind::PlotCandidate,
            producer_role: NoNameRole::Director,
            title: "plot text hint".to_string(),
            summary: "suggest plot text hint".to_string(),
            focus: "mountain gate".to_string(),
            target_segment: NoNameTargetSegment::CurrentTurnTail,
            intended_effect: "append a reviewed narrative hint".to_string(),
            rationale: "keep the next beat focused".to_string(),
            suggested_action: Some("manual apply".to_string()),
            labels: vec!["test".to_string()],
            apply_scopes: vec![NoNameApplyScope::PlotTextHint],
            status: NoNameProposalStatus::Applied,
            applyable: true,
        });
        trace.record_controlled_output_review(
            Some(proposal_id),
            NoNameControlledOutputKind::SceneAugmentation,
            Vec::new(),
            NoNameControlledOutputReview {
                request_id: request_id.clone(),
                decision: NoNameControlledOutputDecision::NeedsReview,
                reason: "plot text hint requires human review".to_string(),
                normalized_kind: Some(NoNameControlledOutputKind::SceneAugmentation),
                safe_apply_scope: Some(NoNameApplyScope::PlotTextHint),
                requires_human_review: true,
            },
        );
        (trace, request_id)
    }

    fn make_low_risk_proposal() -> NoNameProposal {
        NoNameProposal {
            proposal_id: "proposal-low-risk".to_string(),
            kind: NoNameProposalKind::PlotCandidate,
            producer_role: NoNameRole::Director,
            title: "low risk hint".to_string(),
            summary: "suggest low risk hints".to_string(),
            focus: "sect crisis".to_string(),
            target_segment: NoNameTargetSegment::CurrentTurnTail,
            intended_effect: "guide next turn without mutating final plot state".to_string(),
            rationale: "keep safe outputs scoped".to_string(),
            suggested_action: Some("low risk apply".to_string()),
            labels: vec!["test".to_string()],
            apply_scopes: vec![
                NoNameApplyScope::PlotTextHint,
                NoNameApplyScope::ChapterSummaryHint,
                NoNameApplyScope::OptionBiasHint,
            ],
            status: NoNameProposalStatus::Applied,
            applyable: true,
        }
    }

    #[test]
    fn low_risk_apply_plan_set_orders_and_skips_forbidden_scopes() {
        let proposal = make_low_risk_proposal();
        let plot_state = make_plot_state("");
        let plans = build_noname_apply_plan_set(
            &proposal,
            Some(&plot_state),
            Some("current turn narrative"),
        );

        assert_eq!(
            plans.iter().map(|plan| plan.target).collect::<Vec<_>>(),
            vec![
                "plot_text_hint",
                "plot_augmentation_hint",
                "chapter_summary_hint",
                "option_bias_hint"
            ]
        );
        assert_eq!(
            plans.iter().map(|plan| plan.order).collect::<Vec<_>>(),
            vec![1, 2, 3, 4]
        );
        assert!(matches!(
            plans[0].decision,
            NoNameApplyTargetDecision::Apply
        ));
        assert!(matches!(
            &plans[1].decision,
            NoNameApplyTargetDecision::Skip { outcome, .. }
                if *outcome == "skipped_scope_forbidden"
        ));
    }

    #[test]
    fn apply_noname_plot_text_hint_updates_text_and_trace() {
        let proposal = make_low_risk_proposal();
        let mut trace = NoNameTrace::empty(
            "trace-low-risk",
            "session-1",
            "turn-1",
            NoNameMode::Assisted,
        );
        let mut plot_text = "The mountain gate falls quiet.".to_string();

        let applied = apply_noname_plot_text_hint(&mut plot_text, &mut trace, &proposal);

        assert!(applied);
        assert!(plot_text.contains("【NoName】重点关注：sect crisis"));
        assert!(trace
            .proposal_transition_log
            .iter()
            .any(|entry| entry == "proposal-low-risk:applied:plot_text_hint"));
        assert!(trace
            .apply_execution_log
            .iter()
            .any(|entry| entry.target == "plot_text_hint" && entry.outcome == "applied"));
    }

    #[test]
    fn apply_noname_low_risk_outputs_updates_summary_diagnostics_and_trace() {
        let proposal = make_low_risk_proposal();
        let mut trace = NoNameTrace::empty(
            "trace-low-risk",
            "session-1",
            "turn-1",
            NoNameMode::Assisted,
        );
        let mut plot_state = make_plot_state("");

        apply_noname_low_risk_outputs(&mut plot_state, &mut trace, &proposal, true);

        assert!(plot_state
            .current_chapter
            .summary
            .contains("NoName提示：后续重点关注sect crisis"));
        assert!(plot_state
            .last_generation_diagnostics
            .as_deref()
            .unwrap_or_default()
            .contains("NoName选项偏置：下轮优先围绕sect crisis提供行动切入点"));
        assert!(trace
            .apply_execution_log
            .iter()
            .any(|entry| { entry.target == "chapter_summary_hint" && entry.outcome == "applied" }));
        assert!(trace
            .apply_execution_log
            .iter()
            .any(|entry| { entry.target == "option_bias_hint" && entry.outcome == "applied" }));
        assert_eq!(
            trace
                .apply_result
                .as_ref()
                .map(|entry| entry.outcome.as_str()),
            Some("applied_scoped_outputs")
        );
        assert!(trace
            .apply_result
            .as_ref()
            .and_then(|entry| entry.reason.as_deref())
            .is_some_and(|note| note.contains("plot_text_hint")));
    }

    #[test]
    fn human_review_apply_intent_records_ready_for_plot_text() {
        let (mut trace, request_id) = make_trace_for_plot_text_review();

        record_human_review_apply_intent(
            &mut trace,
            &request_id,
            NoNameHumanReviewDecision::ApprovedForHigherApply,
            Some(NoNameApplyScope::PlotTextHint),
        );

        assert!(trace.apply_plan_log.iter().any(|entry| {
            entry.target == "plot_text_hint" && entry.decision == "review_intent_ready"
        }));
        assert!(trace.apply_execution_log.iter().any(|entry| {
            entry.target == "plot_text_hint" && entry.outcome == "awaiting_second_guardrail"
        }));
        assert!(trace
            .proposal_transition_log
            .iter()
            .any(|entry| entry.ends_with(":apply_intent:awaiting_second_guardrail")));
    }

    #[test]
    fn apply_human_review_decision_to_trace_updates_review_and_logs() {
        let (mut trace, request_id) = make_trace_for_plot_text_review();

        apply_human_review_decision_to_trace(
            &mut trace,
            &request_id,
            NoNameHumanReviewDecision::ApprovedForHigherApply,
            123,
        )
        .expect("human review decision should be applied to trace");

        let review = &trace.controlled_output_reviews[0];
        assert_eq!(
            review.human_review_decision,
            Some(NoNameHumanReviewDecision::ApprovedForHigherApply)
        );
        assert_eq!(review.human_reviewed_at, Some(123));
        assert_eq!(
            review.human_review_note.as_deref(),
            Some(NoNameHumanReviewDecision::ApprovedForHigherApply.note())
        );
        assert!(trace.proposal_transition_log.iter().any(|entry| {
            entry
                == &format!(
                    "{}:human_review:{}",
                    request_id,
                    NoNameHumanReviewDecision::ApprovedForHigherApply.as_str()
                )
        }));
        assert!(trace.apply_execution_log.iter().any(|entry| {
            entry.target == "plot_text_hint" && entry.outcome == "awaiting_second_guardrail"
        }));
    }

    #[test]
    fn apply_human_review_decision_to_trace_rejects_missing_safe_scope_without_mutation() {
        let (mut trace, request_id) = make_trace_for_plot_text_review();
        trace.controlled_output_reviews[0].safe_apply_scope = None;

        let error = apply_human_review_decision_to_trace(
            &mut trace,
            &request_id,
            NoNameHumanReviewDecision::ApprovedForHigherApply,
            456,
        )
        .expect_err("missing safe apply scope should fail");

        assert_eq!(
            error,
            format!(
                "NoName controlled output review has no safe apply scope: {}",
                request_id
            )
        );
        let review = &trace.controlled_output_reviews[0];
        assert_eq!(review.human_review_decision, None);
        assert_eq!(review.human_reviewed_at, None);
        assert_eq!(review.human_review_note, None);
    }

    #[test]
    fn human_review_apply_intent_rejects_non_second_guardrail_scope() {
        let (mut trace, request_id) = make_trace_for_summary_apply();

        record_human_review_apply_intent(
            &mut trace,
            &request_id,
            NoNameHumanReviewDecision::ApprovedForHigherApply,
            Some(NoNameApplyScope::ChapterSummaryHint),
        );

        assert!(trace.apply_execution_log.iter().any(|entry| {
            entry.target == "chapter_summary_hint"
                && entry.outcome == "second_guardrail_rejected"
                && entry.note.as_deref().is_some_and(|note| {
                    note.contains("plot_text_hint") && note.contains("plot_augmentation_hint")
                })
        }));
    }

    #[test]
    fn second_guardrail_allow_records_reviewed_apply_gate() {
        let (mut trace, request_id) = make_trace_for_plot_text_review();
        record_human_review_apply_intent(
            &mut trace,
            &request_id,
            NoNameHumanReviewDecision::ApprovedForHigherApply,
            Some(NoNameApplyScope::PlotTextHint),
        );

        record_second_guardrail_decision(
            &mut trace,
            &request_id,
            NoNameSecondGuardrailDecision::Allow,
            Some(NoNameApplyScope::PlotTextHint),
        )
        .expect("second guardrail allow should be recorded");

        assert!(trace.apply_execution_log.iter().any(|entry| {
            entry.target == "plot_text_hint" && entry.outcome == "second_guardrail_allowed"
        }));
        assert_eq!(
            trace
                .apply_result
                .as_ref()
                .map(|item| item.outcome.as_str()),
            Some("second_guardrail_allow")
        );
    }

    #[test]
    fn resolve_second_guardrail_for_trace_requires_review_approval() {
        let (mut trace, request_id) = make_trace_for_plot_text_review();

        let error = resolve_second_guardrail_for_trace(
            &mut trace,
            &request_id,
            NoNameSecondGuardrailDecision::Allow,
        )
        .expect_err("second guardrail should require approved human review");

        assert_eq!(
            error,
            format!(
                "NoName controlled output review is not approved for second guardrail: {}",
                request_id
            )
        );
    }

    #[test]
    fn resolve_second_guardrail_for_trace_records_allow() {
        let (mut trace, request_id) = make_trace_for_plot_text_review();
        apply_human_review_decision_to_trace(
            &mut trace,
            &request_id,
            NoNameHumanReviewDecision::ApprovedForHigherApply,
            789,
        )
        .expect("human review should prepare second guardrail");

        resolve_second_guardrail_for_trace(
            &mut trace,
            &request_id,
            NoNameSecondGuardrailDecision::Allow,
        )
        .expect("second guardrail allow should be recorded");

        assert!(trace.apply_execution_log.iter().any(|entry| {
            entry.target == "plot_text_hint" && entry.outcome == "second_guardrail_allowed"
        }));
    }

    #[test]
    fn second_guardrail_allows_plot_augmentation_hint() {
        let (mut trace, request_id) = make_trace_for_plot_augmentation_apply();
        trace.apply_execution_log.clear();
        trace.proposal_transition_log.clear();
        trace.controlled_output_reviews[0].human_review_decision = None;

        record_human_review_apply_intent(
            &mut trace,
            &request_id,
            NoNameHumanReviewDecision::ApprovedForHigherApply,
            Some(NoNameApplyScope::PlotAugmentationHint),
        );
        record_second_guardrail_decision(
            &mut trace,
            &request_id,
            NoNameSecondGuardrailDecision::Allow,
            Some(NoNameApplyScope::PlotAugmentationHint),
        )
        .expect("plot augmentation second guardrail allow should be recorded");

        assert!(trace.apply_execution_log.iter().any(|entry| {
            entry.target == "plot_augmentation_hint" && entry.outcome == "second_guardrail_allowed"
        }));
    }

    #[test]
    fn second_guardrail_rejects_review_that_is_not_waiting() {
        let (mut trace, request_id) = make_trace_for_plot_text_review();

        record_second_guardrail_decision(
            &mut trace,
            &request_id,
            NoNameSecondGuardrailDecision::Allow,
            Some(NoNameApplyScope::PlotTextHint),
        )
        .expect("second guardrail rejection should be recorded as trace outcome");

        assert!(trace.apply_execution_log.iter().any(|entry| {
            entry.target == "plot_text_hint"
                && entry.outcome == "second_guardrail_rejected"
                && entry
                    .note
                    .as_deref()
                    .is_some_and(|note| note.contains("awaiting_second_guardrail"))
        }));
        assert_eq!(
            trace
                .apply_result
                .as_ref()
                .map(|item| item.outcome.as_str()),
            Some("second_guardrail_reject")
        );
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
                plot_augmentation_snapshot: None,
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
    fn reviewed_apply_input_can_write_chapter_summary_hint() {
        let (trace, request_id) = make_trace_for_summary_apply();
        let plot_state = make_plot_state("existing summary");

        let outcome = apply_reviewed_output_input_to_plot_state(
            trace,
            NoNameReviewedApplyRequestInput {
                request_id,
                scope: NoNameApplyScope::ChapterSummaryHint,
                chapter_index: Some(1),
                segment_index: None,
                expected_segment_text: None,
                expected_summary: Some("existing summary".to_string()),
                expected_generation_diagnostics: None,
                expected_plot_augmentation_hints: None,
            },
            plot_state,
        )
        .expect("summary hint should be manually applied from request input");

        assert!(outcome
            .plot_state
            .current_chapter
            .summary
            .starts_with("NoName summary hint: sect crisis; existing summary"));
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
                plot_augmentation_snapshot: None,
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
                plot_augmentation_snapshot: None,
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
                plot_augmentation_snapshot: None,
            },
            plot_state,
        )
        .expect_err("stale diagnostics should be rejected");

        assert!(error.contains("diagnostics snapshot mismatch"));
    }

    #[test]
    fn reviewed_apply_can_stage_plot_augmentation_hint() {
        let (trace, request_id) = make_trace_for_plot_augmentation_apply();
        let mut plot_state = make_plot_state("");
        plot_state
            .pending_plot_augmentation_hints
            .push("existing hint".to_string());

        let outcome = apply_reviewed_output_to_plot_state(
            trace,
            NoNameReviewedApplyRequest {
                request_id,
                scope: NoNameApplyScope::PlotAugmentationHint,
                segment_snapshot: None,
                summary_snapshot: None,
                diagnostics_snapshot: None,
                plot_augmentation_snapshot: Some(NoNameApplyPlotAugmentationSnapshot {
                    chapter_index: 1,
                    expected_plot_augmentation_hints: vec!["existing hint".to_string()],
                }),
            },
            plot_state,
        )
        .expect("plot augmentation hint should be staged");

        assert_eq!(outcome.plot_state.pending_plot_augmentation_hints.len(), 2);
        assert!(outcome
            .plot_state
            .pending_plot_augmentation_hints
            .iter()
            .any(|hint| hint.contains("NoName plot augmentation: focus=hidden cave")));
        assert!(outcome.trace.apply_execution_log.iter().any(|entry| {
            entry.target == "plot_augmentation_hint"
                && entry.outcome == "manual_plot_augmentation_hint_applied"
        }));
    }

    #[test]
    fn reviewed_apply_rejects_stale_plot_augmentation_snapshot() {
        let (trace, request_id) = make_trace_for_plot_augmentation_apply();
        let mut plot_state = make_plot_state("");
        plot_state
            .pending_plot_augmentation_hints
            .push("new hint".to_string());

        let error = apply_reviewed_output_to_plot_state(
            trace,
            NoNameReviewedApplyRequest {
                request_id,
                scope: NoNameApplyScope::PlotAugmentationHint,
                segment_snapshot: None,
                summary_snapshot: None,
                diagnostics_snapshot: None,
                plot_augmentation_snapshot: Some(NoNameApplyPlotAugmentationSnapshot {
                    chapter_index: 1,
                    expected_plot_augmentation_hints: vec!["old hint".to_string()],
                }),
            },
            plot_state,
        )
        .expect_err("stale plot augmentation snapshot should be rejected");

        assert!(error.contains("plot augmentation snapshot mismatch"));
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
                plot_augmentation_snapshot: None,
            },
            plot_state,
        )
        .expect_err("missing reviewed proposal binding should be rejected");

        assert!(error.contains("NoName proposal not found"));
    }
}
