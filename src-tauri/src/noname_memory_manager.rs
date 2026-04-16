use crate::memory_layers::MemoryLayers;
use crate::noname_memory_compaction::{
    NoNameChapterCompactionInput, NoNameCompactionSummary, NoNameMemoryCompactionService,
    NoNameTraceCompactionInput, NoNameTurnCompactionInput,
};
use crate::noname_memory_retrieval::{NoNameMemoryQuery, NoNameRetrievedMemories};
use crate::noname_memory_store::NoNameMemoryStore;
use crate::noname_memory_types::{
    NoNameEpisodicMemoryItem, NoNameNarrativeMemoryItem, NoNameSemanticMemoryItem,
    NoNameWorkingMemoryItem,
};
use crate::noname_note_store::{NoNameChapterNoteReview, NoNameNoteStore, NoNameNoteUpdate};

#[derive(Debug, Clone)]
pub struct NoNameMemoryManager {
    store: NoNameMemoryStore,
    notes: NoNameNoteStore,
}

impl Default for NoNameMemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NoNameMemoryManager {
    pub fn new() -> Self {
        Self {
            store: NoNameMemoryStore::new(),
            notes: NoNameNoteStore::new(),
        }
    }

    pub fn ingest_legacy_layers(&mut self, layers: &MemoryLayers) {
        for event in &layers.recent_events {
            self.store.push_episodic(event.clone().into());
        }
        for fact in &layers.world_facts {
            self.store.upsert_semantic(fact.clone().into());
        }
        for chapter in &layers.chapter_summaries {
            let narrative: NoNameNarrativeMemoryItem = chapter.clone().into();
            self.notes.upsert(narrative.clone());
            self.store.upsert_narrative(narrative);
        }
    }

    pub fn push_working_memory(&mut self, item: NoNameWorkingMemoryItem, max_len: usize) {
        self.store.push_working(item, max_len);
    }

    pub fn push_episodic_memory(&mut self, item: NoNameEpisodicMemoryItem) {
        self.store.push_episodic(item);
    }

    pub fn upsert_semantic_memory(&mut self, item: NoNameSemanticMemoryItem) {
        self.store.upsert_semantic(item);
    }

    pub fn upsert_narrative_memory(&mut self, item: NoNameNarrativeMemoryItem) {
        self.notes.upsert(item.clone());
        self.store.upsert_narrative(item);
    }

    pub fn update_note(
        &mut self,
        note_id: &str,
        update: NoNameNoteUpdate,
    ) -> Option<NoNameNarrativeMemoryItem> {
        let updated = self.notes.update(note_id, update)?;
        self.store.upsert_narrative(updated.clone());
        Some(updated)
    }

    pub fn resolve_note(
        &mut self,
        note_id: &str,
        updated_at: u64,
    ) -> Option<NoNameNarrativeMemoryItem> {
        let updated = self.notes.resolve(note_id, updated_at)?;
        self.store.upsert_narrative(updated.clone());
        Some(updated)
    }

    pub fn archive_note(
        &mut self,
        note_id: &str,
        updated_at: u64,
    ) -> Option<NoNameNarrativeMemoryItem> {
        let updated = self.notes.archive(note_id, updated_at)?;
        self.store.upsert_narrative(updated.clone());
        Some(updated)
    }

    pub fn organize_chapter_notes(
        &mut self,
        chapter_index: u32,
        updated_at: u64,
    ) -> NoNameChapterNoteReview {
        let review = self.notes.organize_chapter_end(chapter_index, updated_at);
        for note in self.notes.list_by_chapter(chapter_index) {
            self.store.upsert_narrative(note);
        }
        review
    }

    pub fn retrieve(&self, query: &NoNameMemoryQuery) -> NoNameRetrievedMemories {
        self.store.retrieve(query)
    }

    pub fn compact_turn_memory(
        &self,
        input: NoNameTurnCompactionInput,
    ) -> NoNameCompactionSummary {
        NoNameMemoryCompactionService::new().compact_turn(input)
    }

    pub fn compact_chapter_memory(
        &self,
        input: NoNameChapterCompactionInput,
    ) -> NoNameCompactionSummary {
        NoNameMemoryCompactionService::new().compact_chapter(input)
    }

    pub fn compact_trace_memory(
        &self,
        input: NoNameTraceCompactionInput,
    ) -> NoNameCompactionSummary {
        NoNameMemoryCompactionService::new().compact_trace(input)
    }

    pub fn upsert_compaction_summary(&mut self, summary: &NoNameCompactionSummary) {
        self.upsert_narrative_memory(summary.to_narrative_memory());
    }

    pub fn active_notes(&self) -> Vec<NoNameNarrativeMemoryItem> {
        self.notes.list_active()
    }

    pub fn notes_by_chapter(&self, chapter_index: u32) -> Vec<NoNameNarrativeMemoryItem> {
        self.notes.list_by_chapter(chapter_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_layers::{ChapterSummary, MemoryEntry, WorldFact};
    use crate::noname_memory_types::{NoNameNarrativeNoteType, NoNameNarrativeStatus};
    use crate::noname_types::NoNameRole;

    #[test]
    fn manager_ingests_legacy_layers_and_retrieves_memories() {
        let mut manager = NoNameMemoryManager::new();
        let layers = MemoryLayers {
            recent_events: vec![MemoryEntry {
                event_id: "event-1".to_string(),
                summary: "玩家返回山门".to_string(),
                turn: 1,
            }],
            chapter_summaries: vec![ChapterSummary {
                chapter_id: "chapter-1".to_string(),
                title: "山门风云".to_string(),
                summary: "危机渐近".to_string(),
            }],
            world_facts: vec![WorldFact {
                fact_id: "fact-1".to_string(),
                subject: "玩家".to_string(),
                predicate: "位于".to_string(),
                object: "山门".to_string(),
            }],
        };

        manager.ingest_legacy_layers(&layers);
        let result = manager.retrieve(&NoNameMemoryQuery {
            role: NoNameRole::Director,
            search_term: Some("山门".to_string()),
            actor: None,
            location: None,
            goal: None,
            keyword: None,
            token_budget: 200,
            per_section_limit: 4,
        });

        assert_eq!(result.episodic.len(), 1);
        assert_eq!(result.semantic.len(), 1);
        assert_eq!(
            manager.active_notes()[0].status,
            NoNameNarrativeStatus::Active
        );
    }

    #[test]
    fn manager_can_store_compaction_summary_for_later_retrieval() {
        let mut manager = NoNameMemoryManager::new();
        let summary = manager.compact_turn_memory(NoNameTurnCompactionInput {
            turn_id: "turn-1".to_string(),
            chapter_index: Some(1),
            location_id: Some("gate".to_string()),
            actor_mentions: vec!["player".to_string()],
            goal: Some("hold gate".to_string()),
            segments: vec!["Player holds the gate while the ward flickers.".to_string()],
            conflicts: vec!["ward flickers".to_string()],
            unresolved_threads: Vec::new(),
            relationships: Vec::new(),
            created_at: 1,
        });

        manager.upsert_compaction_summary(&summary);

        let active_notes = manager.active_notes();
        assert_eq!(active_notes.len(), 1);
        assert_eq!(active_notes[0].note_id, "compact-turn-turn-1");
        assert_eq!(active_notes[0].summary, summary.summary);
    }

    #[test]
    fn manager_keeps_note_lifecycle_in_sync_with_retrieval_store() {
        let mut manager = NoNameMemoryManager::new();
        manager.upsert_narrative_memory(NoNameNarrativeMemoryItem {
            note_id: "note-1".to_string(),
            chapter_index: 1,
            arc_id: None,
            note_type: NoNameNarrativeNoteType::Goal,
            title: "Hold Gate".to_string(),
            summary: "Keep the mountain gate secure.".to_string(),
            status: NoNameNarrativeStatus::Active,
            related_entities: vec!["player".to_string()],
            updated_at: 1,
        });

        manager.resolve_note("note-1", 2);
        let review = manager.organize_chapter_notes(1, 3);

        assert_eq!(review.archived_from_resolved_count, 1);
        assert_eq!(manager.active_notes().len(), 0);

        let chapter_notes = manager.notes_by_chapter(1);
        assert_eq!(chapter_notes.len(), 1);
        assert_eq!(chapter_notes[0].status, NoNameNarrativeStatus::Archived);
    }
}
