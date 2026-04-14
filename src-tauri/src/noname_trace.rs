use crate::noname_types::{NoNameMode, NoNameProposal, NoNameTraceStage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameCapabilityCallRecord {
    pub capability_id: String,
    pub call_kind: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameGuardrailTraceResult {
    pub outcome: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameApplyTraceResult {
    pub attempted: bool,
    pub outcome: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameApplyExecutionRecord {
    pub target: String,
    pub outcome: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameApplyPlanRecord {
    pub order: u32,
    pub target: String,
    pub decision: String,
    pub priority: u32,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameTrace {
    pub trace_id: String,
    pub session_id: String,
    pub turn_id: String,
    pub mode: NoNameMode,
    pub graph_path: Vec<NoNameTraceStage>,
    #[serde(default)]
    pub capability_calls: Vec<NoNameCapabilityCallRecord>,
    #[serde(default)]
    pub proposals: Vec<NoNameProposal>,
    #[serde(default)]
    pub proposal_transition_log: Vec<String>,
    #[serde(default)]
    pub apply_plan_log: Vec<NoNameApplyPlanRecord>,
    #[serde(default)]
    pub apply_execution_log: Vec<NoNameApplyExecutionRecord>,
    pub guardrail_result: Option<NoNameGuardrailTraceResult>,
    pub apply_result: Option<NoNameApplyTraceResult>,
    pub fallback_used: bool,
    pub elapsed_ms: u64,
}

impl NoNameTrace {
    pub fn empty(
        trace_id: impl Into<String>,
        session_id: impl Into<String>,
        turn_id: impl Into<String>,
        mode: NoNameMode,
    ) -> Self {
        Self {
            trace_id: trace_id.into(),
            session_id: session_id.into(),
            turn_id: turn_id.into(),
            mode,
            graph_path: Vec::new(),
            capability_calls: Vec::new(),
            proposals: Vec::new(),
            proposal_transition_log: Vec::new(),
            apply_plan_log: Vec::new(),
            apply_execution_log: Vec::new(),
            guardrail_result: None,
            apply_result: None,
            fallback_used: false,
            elapsed_ms: 0,
        }
    }

    pub fn push_stage(&mut self, stage: NoNameTraceStage) {
        self.graph_path.push(stage);
    }

    pub fn record_capability_call(
        &mut self,
        capability_id: impl Into<String>,
        call_kind: impl Into<String>,
        status: impl Into<String>,
    ) {
        self.capability_calls.push(NoNameCapabilityCallRecord {
            capability_id: capability_id.into(),
            call_kind: call_kind.into(),
            status: status.into(),
        });
    }

    pub fn record_proposal(&mut self, proposal: NoNameProposal) {
        self.proposals.push(proposal);
    }

    pub fn replace_last_proposal(&mut self, proposal: NoNameProposal) {
        if let Some(last) = self.proposals.last_mut() {
            *last = proposal;
        } else {
            self.proposals.push(proposal);
        }
    }

    pub fn set_guardrail_result(&mut self, outcome: impl Into<String>, reason: Option<String>) {
        self.guardrail_result = Some(NoNameGuardrailTraceResult {
            outcome: outcome.into(),
            reason,
        });
    }

    pub fn record_proposal_transition(&mut self, entry: impl Into<String>) {
        self.proposal_transition_log.push(entry.into());
    }

    pub fn set_apply_result(
        &mut self,
        attempted: bool,
        outcome: impl Into<String>,
        reason: Option<String>,
    ) {
        self.apply_result = Some(NoNameApplyTraceResult {
            attempted,
            outcome: outcome.into(),
            reason,
        });
    }

    pub fn record_apply_plan(
        &mut self,
        order: u32,
        target: impl Into<String>,
        decision: impl Into<String>,
        priority: u32,
        note: Option<String>,
    ) {
        self.apply_plan_log.push(NoNameApplyPlanRecord {
            order,
            target: target.into(),
            decision: decision.into(),
            priority,
            note,
        });
    }

    pub fn record_apply_execution(
        &mut self,
        target: impl Into<String>,
        outcome: impl Into<String>,
        note: Option<String>,
    ) {
        self.apply_execution_log.push(NoNameApplyExecutionRecord {
            target: target.into(),
            outcome: outcome.into(),
            note,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_trace_starts_without_graph_path() {
        let trace = NoNameTrace::empty("trace-1", "session-1", "turn-1", NoNameMode::ObserveOnly);
        assert!(trace.graph_path.is_empty());
        assert!(!trace.fallback_used);
    }

    #[test]
    fn push_stage_records_path_order() {
        let mut trace =
            NoNameTrace::empty("trace-1", "session-1", "turn-1", NoNameMode::ObserveOnly);
        trace.push_stage(NoNameTraceStage::CollectTurnInput);
        trace.push_stage(NoNameTraceStage::PersistTrace);

        assert_eq!(
            trace.graph_path,
            vec![
                NoNameTraceStage::CollectTurnInput,
                NoNameTraceStage::PersistTrace
            ]
        );
    }

    #[test]
    fn record_capability_call_appends_trace_entry() {
        let mut trace =
            NoNameTrace::empty("trace-1", "session-1", "turn-1", NoNameMode::ObserveOnly);
        trace.record_capability_call("tool.echo", "tool", "ok");

        assert_eq!(trace.capability_calls.len(), 1);
        assert_eq!(trace.capability_calls[0].capability_id, "tool.echo");
    }

    #[test]
    fn record_proposal_appends_trace_entry() {
        let mut trace =
            NoNameTrace::empty("trace-1", "session-1", "turn-1", NoNameMode::ObserveOnly);
        trace.record_proposal(NoNameProposal {
            proposal_id: "proposal-1".to_string(),
            kind: crate::noname_types::NoNameProposalKind::PlotCandidate,
            producer_role: crate::noname_types::NoNameRole::Director,
            title: "山门危机".to_string(),
            summary: "建议优先观察山门危机".to_string(),
            focus: "山门危机".to_string(),
            target_segment: crate::noname_types::NoNameTargetSegment::CurrentTurnTail,
            intended_effect: "维持低风险诊断导向".to_string(),
            rationale: "当前线索最集中".to_string(),
            suggested_action: None,
            labels: vec!["director".to_string()],
            apply_scopes: vec![crate::noname_types::NoNameApplyScope::Diagnostics],
            status: crate::noname_types::NoNameProposalStatus::Observed,
            applyable: false,
        });

        assert_eq!(trace.proposals.len(), 1);
        assert_eq!(trace.proposals[0].title, "山门危机");
    }

    #[test]
    fn replace_last_proposal_updates_existing_entry() {
        let mut trace =
            NoNameTrace::empty("trace-1", "session-1", "turn-1", NoNameMode::ObserveOnly);
        trace.record_proposal(NoNameProposal {
            proposal_id: "proposal-1".to_string(),
            kind: crate::noname_types::NoNameProposalKind::PlotCandidate,
            producer_role: crate::noname_types::NoNameRole::Director,
            title: "旧提案".to_string(),
            summary: "旧摘要".to_string(),
            focus: "旧焦点".to_string(),
            target_segment: crate::noname_types::NoNameTargetSegment::CurrentTurnTail,
            intended_effect: "旧效果".to_string(),
            rationale: "旧理由".to_string(),
            suggested_action: None,
            labels: vec!["director".to_string()],
            apply_scopes: vec![crate::noname_types::NoNameApplyScope::Diagnostics],
            status: crate::noname_types::NoNameProposalStatus::Observed,
            applyable: false,
        });
        trace.replace_last_proposal(NoNameProposal {
            proposal_id: "proposal-1".to_string(),
            kind: crate::noname_types::NoNameProposalKind::PlotCandidate,
            producer_role: crate::noname_types::NoNameRole::Director,
            title: "新提案".to_string(),
            summary: "新摘要".to_string(),
            focus: "新焦点".to_string(),
            target_segment: crate::noname_types::NoNameTargetSegment::CurrentTurnTail,
            intended_effect: "新效果".to_string(),
            rationale: "新理由".to_string(),
            suggested_action: Some("进入 assisted".to_string()),
            labels: vec!["director".to_string(), "assisted_ready".to_string()],
            apply_scopes: vec![
                crate::noname_types::NoNameApplyScope::Diagnostics,
                crate::noname_types::NoNameApplyScope::ChapterSummaryHint,
            ],
            status: crate::noname_types::NoNameProposalStatus::Ready,
            applyable: true,
        });

        assert_eq!(trace.proposals.len(), 1);
        assert_eq!(trace.proposals[0].title, "新提案");
        assert!(trace.proposals[0].applyable);
        assert_eq!(
            trace.proposals[0].status,
            crate::noname_types::NoNameProposalStatus::Ready
        );
    }

    #[test]
    fn set_guardrail_result_records_outcome() {
        let mut trace =
            NoNameTrace::empty("trace-1", "session-1", "turn-1", NoNameMode::ObserveOnly);
        trace.set_guardrail_result("accept", None);

        assert_eq!(
            trace
                .guardrail_result
                .as_ref()
                .map(|item| item.outcome.as_str()),
            Some("accept")
        );
    }

    #[test]
    fn apply_result_and_transition_can_be_recorded() {
        let mut trace = NoNameTrace::empty("trace-1", "session-1", "turn-1", NoNameMode::Assisted);
        trace.record_proposal_transition("proposal-1:ready");
        trace.record_apply_plan(
            1,
            "chapter_summary_hint",
            "apply",
            200,
            Some("允许写入章节摘要提示".to_string()),
        );
        trace.record_apply_execution(
            "chapter_summary_hint",
            "applied",
            Some("已写入章节摘要提示".to_string()),
        );
        trace.set_apply_result(
            true,
            "preflight_ready",
            Some("已进入 assisted apply 预检".to_string()),
        );

        assert_eq!(trace.proposal_transition_log, vec!["proposal-1:ready"]);
        assert_eq!(trace.apply_plan_log.len(), 1);
        assert_eq!(trace.apply_plan_log[0].order, 1);
        assert_eq!(trace.apply_plan_log[0].decision, "apply");
        assert_eq!(trace.apply_plan_log[0].priority, 200);
        assert_eq!(trace.apply_execution_log.len(), 1);
        assert_eq!(trace.apply_execution_log[0].target, "chapter_summary_hint");
        assert_eq!(
            trace
                .apply_result
                .as_ref()
                .map(|item| item.outcome.as_str()),
            Some("preflight_ready")
        );
        assert_eq!(
            trace.apply_result.as_ref().map(|item| item.attempted),
            Some(true)
        );
    }
}
