use crate::noname_types::{NoNameApplyScope, NoNameProposalRef, NoNameRole};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoNameControlledOutputKind {
    RecapNote,
    SceneAugmentation,
    NarrativeNote,
    IntermediateNarrativeHint,
}

impl NoNameControlledOutputKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RecapNote => "recap_note",
            Self::SceneAugmentation => "scene_augmentation",
            Self::NarrativeNote => "narrative_note",
            Self::IntermediateNarrativeHint => "intermediate_narrative_hint",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoNameForbiddenOutputScope {
    FinalPlotState,
    CanonWorldFact,
    CharacterStats,
    InventoryOrResource,
    MapTopology,
    ChapterLifecycle,
    PlayerChoice,
    CombatOutcome,
}

impl NoNameForbiddenOutputScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FinalPlotState => "final_plot_state",
            Self::CanonWorldFact => "canon_world_fact",
            Self::CharacterStats => "character_stats",
            Self::InventoryOrResource => "inventory_or_resource",
            Self::MapTopology => "map_topology",
            Self::ChapterLifecycle => "chapter_lifecycle",
            Self::PlayerChoice => "player_choice",
            Self::CombatOutcome => "combat_outcome",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoNameControlledOutputDecision {
    Allow,
    Reject,
    NeedsReview,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameControlledOutputPolicy {
    pub allowed_kinds: Vec<NoNameControlledOutputKind>,
    pub forbidden_scopes: Vec<NoNameForbiddenOutputScope>,
    pub max_content_chars: usize,
    pub requires_human_review_for_plot_text: bool,
}

impl Default for NoNameControlledOutputPolicy {
    fn default() -> Self {
        Self {
            allowed_kinds: vec![
                NoNameControlledOutputKind::RecapNote,
                NoNameControlledOutputKind::SceneAugmentation,
                NoNameControlledOutputKind::NarrativeNote,
                NoNameControlledOutputKind::IntermediateNarrativeHint,
            ],
            forbidden_scopes: vec![
                NoNameForbiddenOutputScope::FinalPlotState,
                NoNameForbiddenOutputScope::CanonWorldFact,
                NoNameForbiddenOutputScope::CharacterStats,
                NoNameForbiddenOutputScope::InventoryOrResource,
                NoNameForbiddenOutputScope::MapTopology,
                NoNameForbiddenOutputScope::ChapterLifecycle,
                NoNameForbiddenOutputScope::PlayerChoice,
                NoNameForbiddenOutputScope::CombatOutcome,
            ],
            max_content_chars: 800,
            requires_human_review_for_plot_text: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameControlledOutputRequest {
    pub request_id: String,
    pub kind: NoNameControlledOutputKind,
    pub producer_role: NoNameRole,
    pub proposal_ref: Option<NoNameProposalRef>,
    pub target_scope: NoNameApplyScope,
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub touched_forbidden_scopes: Vec<NoNameForbiddenOutputScope>,
    #[serde(default)]
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameControlledOutputReview {
    pub request_id: String,
    pub decision: NoNameControlledOutputDecision,
    pub reason: String,
    pub normalized_kind: Option<NoNameControlledOutputKind>,
    pub safe_apply_scope: Option<NoNameApplyScope>,
    pub requires_human_review: bool,
}

#[derive(Debug, Clone)]
pub struct NoNameControlledOutputInterface {
    policy: NoNameControlledOutputPolicy,
}

impl Default for NoNameControlledOutputInterface {
    fn default() -> Self {
        Self::new(NoNameControlledOutputPolicy::default())
    }
}

impl NoNameControlledOutputInterface {
    pub fn new(policy: NoNameControlledOutputPolicy) -> Self {
        Self { policy }
    }

    pub fn policy(&self) -> &NoNameControlledOutputPolicy {
        &self.policy
    }

    pub fn review(&self, request: &NoNameControlledOutputRequest) -> NoNameControlledOutputReview {
        if request.request_id.trim().is_empty() {
            return self.reject(request, "missing request id");
        }
        if request.title.trim().is_empty() || request.content.trim().is_empty() {
            return self.reject(request, "title and content are required");
        }
        if !self.policy.allowed_kinds.contains(&request.kind) {
            return self.reject(request, "output kind is not allowed by policy");
        }
        if request.content.chars().count() > self.policy.max_content_chars {
            return self.reject(request, "content exceeds controlled output character budget");
        }

        let touched_forbidden = request
            .touched_forbidden_scopes
            .iter()
            .copied()
            .filter(|scope| self.policy.forbidden_scopes.contains(scope))
            .collect::<Vec<_>>();
        if !touched_forbidden.is_empty() {
            return self.reject(
                request,
                format!(
                    "touches forbidden scope: {}",
                    touched_forbidden
                        .iter()
                        .map(|scope| scope.as_str())
                        .collect::<Vec<_>>()
                        .join(",")
                ),
            );
        }

        if request.target_scope == NoNameApplyScope::PlotTextHint
            && self.policy.requires_human_review_for_plot_text
        {
            return NoNameControlledOutputReview {
                request_id: request.request_id.clone(),
                decision: NoNameControlledOutputDecision::NeedsReview,
                reason: "plot text hint requires human review before higher-layer apply".to_string(),
                normalized_kind: Some(request.kind),
                safe_apply_scope: Some(request.target_scope),
                requires_human_review: true,
            };
        }

        NoNameControlledOutputReview {
            request_id: request.request_id.clone(),
            decision: NoNameControlledOutputDecision::Allow,
            reason: "controlled output stays within allowed boundary".to_string(),
            normalized_kind: Some(request.kind),
            safe_apply_scope: Some(request.target_scope),
            requires_human_review: false,
        }
    }

    pub fn draft_stub(
        &self,
        kind: NoNameControlledOutputKind,
        producer_role: NoNameRole,
        title: impl Into<String>,
        content: impl Into<String>,
    ) -> NoNameControlledOutputRequest {
        let kind_key = kind.as_str();
        NoNameControlledOutputRequest {
            request_id: format!("controlled-output-{}", kind_key),
            kind,
            producer_role,
            proposal_ref: None,
            target_scope: default_apply_scope_for_kind(kind),
            title: title.into(),
            content: content.into(),
            touched_forbidden_scopes: Vec::new(),
            labels: vec!["noname_controlled_output_v1".to_string(), kind_key.to_string()],
        }
    }

    fn reject(
        &self,
        request: &NoNameControlledOutputRequest,
        reason: impl Into<String>,
    ) -> NoNameControlledOutputReview {
        NoNameControlledOutputReview {
            request_id: request.request_id.clone(),
            decision: NoNameControlledOutputDecision::Reject,
            reason: reason.into(),
            normalized_kind: None,
            safe_apply_scope: None,
            requires_human_review: false,
        }
    }
}

pub fn default_apply_scope_for_kind(kind: NoNameControlledOutputKind) -> NoNameApplyScope {
    match kind {
        NoNameControlledOutputKind::RecapNote => NoNameApplyScope::ChapterSummaryHint,
        NoNameControlledOutputKind::SceneAugmentation => NoNameApplyScope::PlotTextHint,
        NoNameControlledOutputKind::NarrativeNote => NoNameApplyScope::Diagnostics,
        NoNameControlledOutputKind::IntermediateNarrativeHint => NoNameApplyScope::OptionBiasHint,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controlled_output_policy_lists_allowed_and_forbidden_boundaries() {
        let policy = NoNameControlledOutputPolicy::default();

        assert!(policy
            .allowed_kinds
            .contains(&NoNameControlledOutputKind::RecapNote));
        assert!(policy
            .allowed_kinds
            .contains(&NoNameControlledOutputKind::IntermediateNarrativeHint));
        assert!(policy
            .forbidden_scopes
            .contains(&NoNameForbiddenOutputScope::FinalPlotState));
        assert!(policy
            .forbidden_scopes
            .contains(&NoNameForbiddenOutputScope::CharacterStats));
    }

    #[test]
    fn review_allows_safe_recap_note() {
        let interface = NoNameControlledOutputInterface::default();
        let request = interface.draft_stub(
            NoNameControlledOutputKind::RecapNote,
            NoNameRole::Director,
            "Gate recap",
            "The mountain gate remains unstable, but no final state is changed.",
        );

        let review = interface.review(&request);

        assert_eq!(review.decision, NoNameControlledOutputDecision::Allow);
        assert_eq!(review.safe_apply_scope, Some(NoNameApplyScope::ChapterSummaryHint));
        assert!(!review.requires_human_review);
    }

    #[test]
    fn review_rejects_forbidden_scope_touch() {
        let interface = NoNameControlledOutputInterface::default();
        let mut request = interface.draft_stub(
            NoNameControlledOutputKind::NarrativeNote,
            NoNameRole::WorldCurator,
            "Canon change",
            "The sect law is permanently rewritten.",
        );
        request
            .touched_forbidden_scopes
            .push(NoNameForbiddenOutputScope::CanonWorldFact);

        let review = interface.review(&request);

        assert_eq!(review.decision, NoNameControlledOutputDecision::Reject);
        assert!(review.reason.contains("canon_world_fact"));
        assert_eq!(review.safe_apply_scope, None);
    }

    #[test]
    fn plot_text_hint_requires_review_in_v1() {
        let interface = NoNameControlledOutputInterface::default();
        let request = interface.draft_stub(
            NoNameControlledOutputKind::SceneAugmentation,
            NoNameRole::CombatNarrator,
            "Scene hint",
            "Add sensory detail around the ward flicker without resolving combat.",
        );

        let review = interface.review(&request);

        assert_eq!(review.decision, NoNameControlledOutputDecision::NeedsReview);
        assert_eq!(review.safe_apply_scope, Some(NoNameApplyScope::PlotTextHint));
        assert!(review.requires_human_review);
    }
}
