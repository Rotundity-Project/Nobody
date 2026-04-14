use crate::memory_layers::MemoryLayers;
use crate::noname_memory_retrieval::{
    retrieve_memories, NoNameMemoryQuery, NoNameRetrievedMemories,
};
use crate::noname_memory_store::NoNameMemoryStore;
use crate::noname_memory_types::{
    NoNameEpisodicMemoryItem, NoNameNarrativeMemoryItem, NoNameSemanticMemoryItem,
    NoNameWorkingMemoryItem,
};
use crate::noname_note_store::NoNameNoteStore;

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

    pub fn retrieve(&self, query: &NoNameMemoryQuery) -> NoNameRetrievedMemories {
        retrieve_memories(
            query,
            self.store.working(),
            self.store.episodic(),
            self.store.semantic(),
            self.store.narrative(),
        )
    }

    pub fn active_notes(&self) -> Vec<NoNameNarrativeMemoryItem> {
        self.notes.list_active()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_layers::{ChapterSummary, MemoryEntry, WorldFact};
    use crate::noname_memory_types::NoNameNarrativeStatus;
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
}
