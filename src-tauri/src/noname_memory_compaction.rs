use crate::noname_memory_types::{
    NoNameEpisodicMemoryItem, NoNameNarrativeMemoryItem, NoNameNarrativeNoteType,
    NoNameNarrativeStatus,
};
use crate::noname_trace::NoNameTrace;
use serde::{Deserialize, Serialize};

const DEFAULT_SUMMARY_CHAR_LIMIT: usize = 320;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoNameCompactionKind {
    Turn,
    Chapter,
    Trace,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NoNameTurnCompactionInput {
    pub turn_id: String,
    pub chapter_index: Option<u32>,
    pub location_id: Option<String>,
    #[serde(default)]
    pub actor_mentions: Vec<String>,
    pub goal: Option<String>,
    #[serde(default)]
    pub segments: Vec<String>,
    #[serde(default)]
    pub conflicts: Vec<String>,
    #[serde(default)]
    pub unresolved_threads: Vec<String>,
    #[serde(default)]
    pub relationships: Vec<String>,
    pub created_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NoNameChapterCompactionInput {
    pub chapter_id: String,
    pub chapter_index: u32,
    pub title: String,
    #[serde(default)]
    pub events: Vec<NoNameEpisodicMemoryItem>,
    #[serde(default)]
    pub notes: Vec<NoNameNarrativeMemoryItem>,
    pub created_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NoNameTraceCompactionInput {
    pub session_id: String,
    #[serde(default)]
    pub traces: Vec<NoNameTrace>,
    pub created_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameCompactionSummary {
    pub summary_id: String,
    pub kind: NoNameCompactionKind,
    pub title: String,
    pub summary: String,
    pub chapter_index: Option<u32>,
    #[serde(default)]
    pub source_ids: Vec<String>,
    #[serde(default)]
    pub key_entities: Vec<String>,
    #[serde(default)]
    pub locations: Vec<String>,
    #[serde(default)]
    pub goals: Vec<String>,
    #[serde(default)]
    pub conflicts: Vec<String>,
    #[serde(default)]
    pub unresolved_threads: Vec<String>,
    #[serde(default)]
    pub relationships: Vec<String>,
    #[serde(default)]
    pub diagnostics: Vec<String>,
    pub estimated_tokens: usize,
    pub created_at: u64,
}

impl NoNameCompactionSummary {
    pub fn to_narrative_memory(&self) -> NoNameNarrativeMemoryItem {
        NoNameNarrativeMemoryItem {
            note_id: self.summary_id.clone(),
            chapter_index: self.chapter_index.unwrap_or_default(),
            arc_id: None,
            note_type: self.primary_note_type(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            status: NoNameNarrativeStatus::Active,
            related_entities: self.key_entities.clone(),
            updated_at: self.created_at,
        }
    }

    fn primary_note_type(&self) -> NoNameNarrativeNoteType {
        if !self.unresolved_threads.is_empty() {
            return NoNameNarrativeNoteType::UnresolvedThread;
        }
        if !self.conflicts.is_empty() {
            return NoNameNarrativeNoteType::Conflict;
        }
        if !self.relationships.is_empty() {
            return NoNameNarrativeNoteType::CharacterArc;
        }
        NoNameNarrativeNoteType::Goal
    }
}

#[derive(Debug, Clone)]
pub struct NoNameMemoryCompactionService {
    summary_char_limit: usize,
}

impl Default for NoNameMemoryCompactionService {
    fn default() -> Self {
        Self::new()
    }
}

impl NoNameMemoryCompactionService {
    pub fn new() -> Self {
        Self {
            summary_char_limit: DEFAULT_SUMMARY_CHAR_LIMIT,
        }
    }

    pub fn with_summary_char_limit(summary_char_limit: usize) -> Self {
        Self { summary_char_limit }
    }

    pub fn compact_turn(&self, input: NoNameTurnCompactionInput) -> NoNameCompactionSummary {
        let mut summary_fragments = input.segments.clone();
        if let Some(goal) = input.goal.as_ref() {
            summary_fragments.push(format!("Goal: {}", goal));
        }

        let summary = compact_fragments(summary_fragments, self.summary_char_limit);
        let locations = dedupe_optional(input.location_id.into_iter());
        let goals = dedupe_optional(input.goal.into_iter());

        NoNameCompactionSummary {
            summary_id: format!("compact-turn-{}", stable_key(&input.turn_id)),
            kind: NoNameCompactionKind::Turn,
            title: format!("Turn {}", input.turn_id),
            estimated_tokens: estimate_tokens(&summary),
            summary,
            chapter_index: input.chapter_index,
            source_ids: vec![input.turn_id],
            key_entities: dedupe_strings(input.actor_mentions),
            locations,
            goals,
            conflicts: dedupe_strings(input.conflicts),
            unresolved_threads: dedupe_strings(input.unresolved_threads),
            relationships: dedupe_strings(input.relationships),
            diagnostics: Vec::new(),
            created_at: input.created_at,
        }
    }

    pub fn compact_chapter(
        &self,
        input: NoNameChapterCompactionInput,
    ) -> NoNameCompactionSummary {
        let mut source_ids = Vec::new();
        let mut fragments = Vec::new();
        let mut key_entities = Vec::new();
        let mut locations = Vec::new();

        for event in &input.events {
            source_ids.push(event.memory_id.clone());
            fragments.push(event.summary.clone());
            key_entities.extend(event.actors.clone());
            if let Some(location_id) = event.location_id.as_ref() {
                locations.push(location_id.clone());
            }
        }

        let mut goals = Vec::new();
        let mut conflicts = Vec::new();
        let mut unresolved_threads = Vec::new();
        let mut relationships = Vec::new();

        for note in &input.notes {
            source_ids.push(note.note_id.clone());
            fragments.push(format!("{}: {}", note.title, note.summary));
            key_entities.extend(note.related_entities.clone());

            match note.note_type {
                NoNameNarrativeNoteType::Goal => goals.push(note_line(note)),
                NoNameNarrativeNoteType::Conflict => conflicts.push(note_line(note)),
                NoNameNarrativeNoteType::Foreshadowing
                | NoNameNarrativeNoteType::UnresolvedThread => {
                    if note.status != NoNameNarrativeStatus::Resolved {
                        unresolved_threads.push(note_line(note));
                    }
                }
                NoNameNarrativeNoteType::CharacterArc => relationships.push(note_line(note)),
            }
        }

        if source_ids.is_empty() {
            source_ids.push(input.chapter_id.clone());
        }

        let summary = compact_fragments(fragments, self.summary_char_limit);
        NoNameCompactionSummary {
            summary_id: format!("compact-chapter-{}", stable_key(&input.chapter_id)),
            kind: NoNameCompactionKind::Chapter,
            title: input.title,
            estimated_tokens: estimate_tokens(&summary),
            summary,
            chapter_index: Some(input.chapter_index),
            source_ids: dedupe_strings(source_ids),
            key_entities: dedupe_strings(key_entities),
            locations: dedupe_strings(locations),
            goals: dedupe_strings(goals),
            conflicts: dedupe_strings(conflicts),
            unresolved_threads: dedupe_strings(unresolved_threads),
            relationships: dedupe_strings(relationships),
            diagnostics: vec![
                format!("events_compacted={}", input.events.len()),
                format!("notes_compacted={}", input.notes.len()),
            ],
            created_at: input.created_at,
        }
    }

    pub fn compact_trace(&self, input: NoNameTraceCompactionInput) -> NoNameCompactionSummary {
        let mut source_ids = Vec::new();
        let mut fragments = Vec::new();
        let mut goals = Vec::new();
        let mut diagnostics = Vec::new();
        let mut key_entities = Vec::new();

        for trace in &input.traces {
            source_ids.push(trace.trace_id.clone());
            fragments.push(format!(
                "trace={} mode={:?} stages={} proposals={} fallback={}",
                trace.trace_id,
                trace.mode,
                trace.graph_path.len(),
                trace.proposals.len(),
                trace.fallback_used
            ));

            for proposal in &trace.proposals {
                key_entities.push(proposal.producer_role.as_str().to_string());
                goals.push(proposal.focus.clone());
                fragments.push(format!(
                    "{} proposal: {}",
                    proposal.producer_role.as_str(),
                    proposal.summary
                ));
            }

            if let Some(guardrail) = trace.guardrail_result.as_ref() {
                diagnostics.push(format!(
                    "guardrail={}",
                    format_outcome(&guardrail.outcome, guardrail.reason.as_deref())
                ));
            }
            if let Some(apply_result) = trace.apply_result.as_ref() {
                diagnostics.push(format!(
                    "apply={}",
                    format_outcome(&apply_result.outcome, apply_result.reason.as_deref())
                ));
            }
            if trace.fallback_used {
                diagnostics.push(format!("fallback_used trace={}", trace.trace_id));
            }
            if !trace.capability_calls.is_empty() {
                diagnostics.push(format!("capability_calls={}", trace.capability_calls.len()));
            }
            if !trace.protocol_events.is_empty() {
                diagnostics.push(format!("protocol_events={}", trace.protocol_events.len()));
            }
        }

        let summary = compact_fragments(fragments, self.summary_char_limit);
        NoNameCompactionSummary {
            summary_id: format!("compact-trace-{}", stable_key(&input.session_id)),
            kind: NoNameCompactionKind::Trace,
            title: format!("Trace diagnostics {}", input.session_id),
            estimated_tokens: estimate_tokens(&summary),
            summary,
            chapter_index: None,
            source_ids: dedupe_strings(source_ids),
            key_entities: dedupe_strings(key_entities),
            locations: Vec::new(),
            goals: dedupe_strings(goals),
            conflicts: Vec::new(),
            unresolved_threads: Vec::new(),
            relationships: Vec::new(),
            diagnostics: dedupe_strings(diagnostics),
            created_at: input.created_at,
        }
    }
}

fn note_line(note: &NoNameNarrativeMemoryItem) -> String {
    format!("{}: {}", note.title, note.summary)
}

fn format_outcome(outcome: &str, reason: Option<&str>) -> String {
    match reason.and_then(clean_fragment) {
        Some(reason) => format!("{} ({})", outcome, reason),
        None => outcome.to_string(),
    }
}

fn compact_fragments(fragments: Vec<String>, max_chars: usize) -> String {
    let joined = dedupe_strings(fragments).join(" ");
    if joined.is_empty() {
        return "No source text provided.".to_string();
    }
    truncate_chars(&joined, max_chars)
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let char_count = value.chars().count();
    if char_count <= max_chars {
        return value.to_string();
    }
    if max_chars <= 3 {
        return value.chars().take(max_chars).collect();
    }
    let mut truncated: String = value.chars().take(max_chars - 3).collect();
    truncated.push_str("...");
    truncated
}

fn estimate_tokens(value: &str) -> usize {
    let by_words = value.split_whitespace().count();
    by_words.max(value.chars().count().div_ceil(4))
}

fn dedupe_optional(values: impl IntoIterator<Item = String>) -> Vec<String> {
    dedupe_strings(values.into_iter().collect())
}

fn dedupe_strings(values: Vec<String>) -> Vec<String> {
    let mut deduped = Vec::new();
    for value in values {
        if let Some(cleaned) = clean_fragment(&value) {
            if !deduped.iter().any(|existing| existing == &cleaned) {
                deduped.push(cleaned);
            }
        }
    }
    deduped
}

fn clean_fragment(value: &str) -> Option<String> {
    let cleaned = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

fn stable_key(value: &str) -> String {
    let key = value
        .chars()
        .map(|item| {
            if item.is_ascii_alphanumeric() || item == '-' || item == '_' {
                item
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if key.is_empty() {
        "unknown".to_string()
    } else {
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noname_memory_types::NoNameMemoryImportance;
    use crate::noname_types::{
        NoNameApplyScope, NoNameMode, NoNameProposal, NoNameProposalKind, NoNameProposalStatus,
        NoNameRole, NoNameTargetSegment,
    };

    #[test]
    fn turn_compaction_preserves_structured_fields_and_shortens_text() {
        let service = NoNameMemoryCompactionService::with_summary_char_limit(80);

        let summary = service.compact_turn(NoNameTurnCompactionInput {
            turn_id: "turn-9".to_string(),
            chapter_index: Some(2),
            location_id: Some("mountain_gate".to_string()),
            actor_mentions: vec!["player".to_string(), "elder".to_string()],
            goal: Some("hold the gate".to_string()),
            segments: vec![
                "The player returns to the mountain gate with a damaged talisman.".to_string(),
                "The elder warns that the outer sect is about to breach the ward.".to_string(),
            ],
            conflicts: vec!["outer sect pressure".to_string()],
            unresolved_threads: vec!["who broke the ward".to_string()],
            relationships: vec!["elder trusts player".to_string()],
            created_at: 42,
        });

        assert_eq!(summary.kind, NoNameCompactionKind::Turn);
        assert!(summary.summary.chars().count() <= 80);
        assert_eq!(summary.key_entities, vec!["player", "elder"]);
        assert_eq!(summary.locations, vec!["mountain_gate"]);
        assert_eq!(summary.goals, vec!["hold the gate"]);
        assert_eq!(summary.conflicts, vec!["outer sect pressure"]);
        assert_eq!(summary.unresolved_threads, vec!["who broke the ward"]);
    }

    #[test]
    fn chapter_compaction_collects_entities_locations_and_threads() {
        let service = NoNameMemoryCompactionService::new();
        let summary = service.compact_chapter(NoNameChapterCompactionInput {
            chapter_id: "chapter-1".to_string(),
            chapter_index: 1,
            title: "Gate Crisis".to_string(),
            events: vec![NoNameEpisodicMemoryItem {
                memory_id: "event-1".to_string(),
                event_type: "battle".to_string(),
                timestamp: 1,
                chapter_index: 1,
                location_id: Some("mountain_gate".to_string()),
                actors: vec!["player".to_string(), "elder".to_string()],
                summary: "The ward shakes during the first assault.".to_string(),
                detail_ref: None,
                importance: NoNameMemoryImportance::High,
            }],
            notes: vec![
                note("note-goal", NoNameNarrativeNoteType::Goal, "Hold Gate", "Keep the gate standing"),
                note(
                    "note-conflict",
                    NoNameNarrativeNoteType::Conflict,
                    "Sect Pressure",
                    "Outer sect probes the ward",
                ),
                note(
                    "note-thread",
                    NoNameNarrativeNoteType::UnresolvedThread,
                    "Broken Ward",
                    "The saboteur is unknown",
                ),
                note(
                    "note-arc",
                    NoNameNarrativeNoteType::CharacterArc,
                    "Elder Trust",
                    "Elder gives player command",
                ),
            ],
            created_at: 99,
        });

        assert_eq!(summary.kind, NoNameCompactionKind::Chapter);
        assert_eq!(summary.chapter_index, Some(1));
        assert!(summary.key_entities.contains(&"player".to_string()));
        assert_eq!(summary.locations, vec!["mountain_gate"]);
        assert_eq!(summary.goals.len(), 1);
        assert_eq!(summary.conflicts.len(), 1);
        assert_eq!(summary.unresolved_threads.len(), 1);
        assert_eq!(summary.relationships.len(), 1);
    }

    #[test]
    fn trace_compaction_distills_diagnostics() {
        let service = NoNameMemoryCompactionService::new();
        let mut trace = NoNameTrace::empty("trace-1", "session-1", "turn-1", NoNameMode::Assisted);
        trace.record_capability_call("tool.director", "tool", "ok");
        trace.set_guardrail_result("reject", Some("missing target".to_string()));
        trace.fallback_used = true;
        trace.record_proposal(NoNameProposal {
            proposal_id: "proposal-1".to_string(),
            kind: NoNameProposalKind::PlotCandidate,
            producer_role: NoNameRole::Director,
            title: "Gate Crisis".to_string(),
            summary: "Keep the scene focused on the gate crisis.".to_string(),
            focus: "gate crisis".to_string(),
            target_segment: NoNameTargetSegment::CurrentTurnTail,
            intended_effect: "stabilize next turn".to_string(),
            rationale: "main conflict is active".to_string(),
            suggested_action: None,
            labels: vec!["director".to_string()],
            apply_scopes: vec![NoNameApplyScope::Diagnostics],
            status: NoNameProposalStatus::Observed,
            applyable: false,
        });

        let summary = service.compact_trace(NoNameTraceCompactionInput {
            session_id: "session-1".to_string(),
            traces: vec![trace],
            created_at: 100,
        });

        assert_eq!(summary.kind, NoNameCompactionKind::Trace);
        assert_eq!(summary.source_ids, vec!["trace-1"]);
        assert!(summary.goals.contains(&"gate crisis".to_string()));
        assert!(summary.diagnostics.iter().any(|item| item.contains("guardrail=reject")));
        assert!(summary
            .diagnostics
            .iter()
            .any(|item| item.contains("fallback_used")));
    }

    #[test]
    fn compaction_summary_can_be_stored_as_narrative_memory() {
        let service = NoNameMemoryCompactionService::new();
        let summary = service.compact_turn(NoNameTurnCompactionInput {
            turn_id: "turn-1".to_string(),
            chapter_index: Some(1),
            location_id: None,
            actor_mentions: vec!["player".to_string()],
            goal: None,
            segments: vec!["Player notices a loose thread in the ward.".to_string()],
            conflicts: Vec::new(),
            unresolved_threads: vec!["loose ward thread".to_string()],
            relationships: Vec::new(),
            created_at: 1,
        });

        let note = summary.to_narrative_memory();

        assert_eq!(note.note_id, "compact-turn-turn-1");
        assert_eq!(note.note_type, NoNameNarrativeNoteType::UnresolvedThread);
        assert_eq!(note.related_entities, vec!["player"]);
    }

    fn note(
        note_id: &str,
        note_type: NoNameNarrativeNoteType,
        title: &str,
        summary: &str,
    ) -> NoNameNarrativeMemoryItem {
        NoNameNarrativeMemoryItem {
            note_id: note_id.to_string(),
            chapter_index: 1,
            arc_id: None,
            note_type,
            title: title.to_string(),
            summary: summary.to_string(),
            status: NoNameNarrativeStatus::Active,
            related_entities: vec!["player".to_string()],
            updated_at: 1,
        }
    }
}
