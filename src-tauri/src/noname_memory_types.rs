use crate::memory_layers::{ChapterSummary, MemoryEntry, WorldFact};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoNameMemoryImportance {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoNameNarrativeNoteType {
    Goal,
    Conflict,
    Foreshadowing,
    UnresolvedThread,
    CharacterArc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NoNameNarrativeStatus {
    Active,
    Resolved,
    Archived,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameWorkingMemoryItem {
    pub memory_id: String,
    pub turn_id: String,
    pub source: String,
    pub category: String,
    pub summary: String,
    pub expires_at: Option<u64>,
    pub priority: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameEpisodicMemoryItem {
    pub memory_id: String,
    pub event_type: String,
    pub timestamp: u64,
    pub chapter_index: u32,
    pub location_id: Option<String>,
    pub actors: Vec<String>,
    pub summary: String,
    pub detail_ref: Option<String>,
    pub importance: NoNameMemoryImportance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameSemanticMemoryItem {
    pub fact_id: String,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: u8,
    pub source: String,
    pub updated_at: u64,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameNarrativeMemoryItem {
    pub note_id: String,
    pub chapter_index: u32,
    pub arc_id: Option<String>,
    pub note_type: NoNameNarrativeNoteType,
    pub title: String,
    pub summary: String,
    pub status: NoNameNarrativeStatus,
    pub related_entities: Vec<String>,
    pub updated_at: u64,
}

impl From<MemoryEntry> for NoNameEpisodicMemoryItem {
    fn from(value: MemoryEntry) -> Self {
        Self {
            memory_id: value.event_id,
            event_type: "legacyEvent".to_string(),
            timestamp: value.turn,
            chapter_index: 0,
            location_id: None,
            actors: Vec::new(),
            summary: value.summary,
            detail_ref: None,
            importance: NoNameMemoryImportance::Medium,
        }
    }
}

impl From<WorldFact> for NoNameSemanticMemoryItem {
    fn from(value: WorldFact) -> Self {
        Self {
            fact_id: value.fact_id,
            subject: value.subject,
            predicate: value.predicate,
            object: value.object,
            confidence: 90,
            source: "legacyWorldFact".to_string(),
            updated_at: 0,
            tags: Vec::new(),
        }
    }
}

impl From<ChapterSummary> for NoNameNarrativeMemoryItem {
    fn from(value: ChapterSummary) -> Self {
        Self {
            note_id: value.chapter_id,
            chapter_index: 0,
            arc_id: None,
            note_type: NoNameNarrativeNoteType::Goal,
            title: value.title,
            summary: value.summary,
            status: NoNameNarrativeStatus::Active,
            related_entities: Vec::new(),
            updated_at: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_world_fact_can_convert_to_semantic_memory() {
        let fact = WorldFact {
            fact_id: "fact-1".to_string(),
            subject: "玩家".to_string(),
            predicate: "位于".to_string(),
            object: "山门".to_string(),
        };

        let semantic = NoNameSemanticMemoryItem::from(fact);
        assert_eq!(semantic.subject, "玩家");
        assert_eq!(semantic.object, "山门");
    }
}
