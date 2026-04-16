use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum NoNameMode {
    #[default]
    Disabled,
    ObserveOnly,
    Assisted,
}

impl NoNameMode {
    pub fn is_enabled(self) -> bool {
        !matches!(self, Self::Disabled)
    }

    pub fn allows_apply(self) -> bool {
        matches!(self, Self::Assisted)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoNameRole {
    Director,
    WorldCurator,
    NpcIntent,
    CombatNarrator,
    System,
}

impl NoNameRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Director => "director",
            Self::WorldCurator => "world_curator",
            Self::NpcIntent => "npc_intent",
            Self::CombatNarrator => "combat_narrator",
            Self::System => "system",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoNameProposalKind {
    PlotCandidate,
    WorldPatchProposal,
    NpcIntentProposal,
    CombatNarration,
    Diagnostic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum NoNameProposalStatus {
    #[default]
    Observed,
    Ready,
    Blocked,
    Applied,
    Fallback,
}

impl NoNameProposalStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Observed => "observed",
            Self::Ready => "ready",
            Self::Blocked => "blocked",
            Self::Applied => "applied",
            Self::Fallback => "fallback",
        }
    }

    pub fn is_applyable(self) -> bool {
        matches!(self, Self::Ready | Self::Applied)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoNameApplyScope {
    Diagnostics,
    ChapterSummaryHint,
    OptionBiasHint,
    PlotTextHint,
}

impl NoNameApplyScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Diagnostics => "diagnostics",
            Self::ChapterSummaryHint => "chapter_summary_hint",
            Self::OptionBiasHint => "option_bias_hint",
            Self::PlotTextHint => "plot_text_hint",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoNameTargetSegment {
    CurrentTurnHead,
    CurrentTurnTail,
    ChapterSummaryHead,
    ChapterSummaryTail,
}

impl NoNameTargetSegment {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CurrentTurnHead => "current_turn_head",
            Self::CurrentTurnTail => "current_turn_tail",
            Self::ChapterSummaryHead => "chapter_summary_head",
            Self::ChapterSummaryTail => "chapter_summary_tail",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoNameEnvelopeKind {
    Task,
    Proposal,
    CapabilityCall,
    CapabilityResult,
    TraceEvent,
    Diagnostic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoNameTraceStage {
    CollectTurnInput,
    BuildContextBundle,
    PlanTurn,
    ExecuteToolSteps,
    AssembleProposal,
    ValidateProposal,
    ApplyProposal,
    PersistTrace,
    ApplyFallback,
    Fallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameIdentity {
    pub trace_id: String,
    pub session_id: String,
    pub turn_id: String,
    pub role: NoNameRole,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NoNameMeta {
    pub created_at_ms: u64,
    #[serde(default)]
    pub labels: Vec<String>,
    pub token_budget: usize,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameEnvelope {
    pub identity: NoNameIdentity,
    pub kind: NoNameEnvelopeKind,
    pub payload: Value,
    pub meta: NoNameMeta,
}

impl NoNameEnvelope {
    pub fn new(
        identity: NoNameIdentity,
        kind: NoNameEnvelopeKind,
        payload: Value,
        meta: NoNameMeta,
    ) -> Self {
        Self {
            identity,
            kind,
            payload,
            meta,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameProposalRef {
    pub proposal_id: String,
    pub kind: NoNameProposalKind,
    pub producer_role: NoNameRole,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameProposal {
    pub proposal_id: String,
    pub kind: NoNameProposalKind,
    pub producer_role: NoNameRole,
    pub title: String,
    pub summary: String,
    pub focus: String,
    pub target_segment: NoNameTargetSegment,
    pub intended_effect: String,
    pub rationale: String,
    pub suggested_action: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub apply_scopes: Vec<NoNameApplyScope>,
    #[serde(default)]
    pub status: NoNameProposalStatus,
    pub applyable: bool,
}

impl NoNameProposal {
    pub fn to_ref(&self) -> NoNameProposalRef {
        NoNameProposalRef {
            proposal_id: self.proposal_id.clone(),
            kind: self.kind,
            producer_role: self.producer_role,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn mode_defaults_to_disabled() {
        assert_eq!(NoNameMode::default(), NoNameMode::Disabled);
    }

    #[test]
    fn observe_only_is_enabled_but_not_applyable() {
        assert!(NoNameMode::ObserveOnly.is_enabled());
        assert!(!NoNameMode::ObserveOnly.allows_apply());
    }

    #[test]
    fn envelope_round_trip_serialization() {
        let envelope = NoNameEnvelope::new(
            NoNameIdentity {
                trace_id: "trace-1".to_string(),
                session_id: "session-1".to_string(),
                turn_id: "turn-1".to_string(),
                role: NoNameRole::Director,
            },
            NoNameEnvelopeKind::Proposal,
            json!({
                "summary": "推进一段剧情候选"
            }),
            NoNameMeta {
                created_at_ms: 123,
                labels: vec!["director".to_string()],
                token_budget: 256,
                timeout_ms: 1500,
            },
        );

        let serialized = serde_json::to_string(&envelope).expect("serialize envelope");
        let deserialized: NoNameEnvelope =
            serde_json::from_str(&serialized).expect("deserialize envelope");

        assert_eq!(deserialized.kind, NoNameEnvelopeKind::Proposal);
        assert_eq!(deserialized.identity.role, NoNameRole::Director);
        assert_eq!(deserialized.payload["summary"], "推进一段剧情候选");
    }

    #[test]
    fn proposal_ref_keeps_kind_and_role() {
        let proposal = NoNameProposalRef {
            proposal_id: "proposal-1".to_string(),
            kind: NoNameProposalKind::WorldPatchProposal,
            producer_role: NoNameRole::WorldCurator,
        };

        assert_eq!(proposal.kind, NoNameProposalKind::WorldPatchProposal);
        assert_eq!(proposal.producer_role, NoNameRole::WorldCurator);
    }

    #[test]
    fn proposal_can_convert_to_ref() {
        let proposal = NoNameProposal {
            proposal_id: "proposal-1".to_string(),
            kind: NoNameProposalKind::PlotCandidate,
            producer_role: NoNameRole::Director,
            title: "山门危机".to_string(),
            summary: "建议优先观察山门危机".to_string(),
            focus: "山门危机".to_string(),
            target_segment: NoNameTargetSegment::CurrentTurnTail,
            intended_effect: "为下一轮提供稳定的冲突承接".to_string(),
            rationale: "当前章节冲突正在汇聚".to_string(),
            suggested_action: Some("保持观察并延后落地".to_string()),
            labels: vec!["director".to_string()],
            apply_scopes: vec![NoNameApplyScope::Diagnostics],
            status: NoNameProposalStatus::Observed,
            applyable: false,
        };

        let proposal_ref = proposal.to_ref();
        assert_eq!(proposal_ref.kind, NoNameProposalKind::PlotCandidate);
        assert_eq!(proposal_ref.producer_role, NoNameRole::Director);
    }

    #[test]
    fn proposal_status_reports_applyable_states() {
        assert!(NoNameProposalStatus::Ready.is_applyable());
        assert!(NoNameProposalStatus::Applied.is_applyable());
        assert!(!NoNameProposalStatus::Observed.is_applyable());
        assert_eq!(NoNameProposalStatus::Blocked.as_str(), "blocked");
    }

    #[test]
    fn apply_scope_reports_stable_keys() {
        assert_eq!(NoNameApplyScope::Diagnostics.as_str(), "diagnostics");
        assert_eq!(
            NoNameApplyScope::ChapterSummaryHint.as_str(),
            "chapter_summary_hint"
        );
    }

    #[test]
    fn target_segment_reports_stable_keys() {
        assert_eq!(
            NoNameTargetSegment::CurrentTurnHead.as_str(),
            "current_turn_head"
        );
        assert_eq!(
            NoNameTargetSegment::ChapterSummaryTail.as_str(),
            "chapter_summary_tail"
        );
    }
}
