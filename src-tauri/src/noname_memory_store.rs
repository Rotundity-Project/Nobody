use crate::noname_memory_retrieval::{
    retrieve_memories, retrieve_memories_with_explanations, NoNameMemoryQuery,
    NoNameRetrievedMemories, NoNameRetrievedMemoryReport,
};
use crate::noname_memory_types::{
    NoNameEpisodicMemoryItem, NoNameNarrativeMemoryItem, NoNameSemanticMemoryItem,
    NoNameWorkingMemoryItem,
};

#[derive(Debug, Default, Clone)]
pub struct NoNameMemoryStore {
    working: Vec<NoNameWorkingMemoryItem>,
    episodic: Vec<NoNameEpisodicMemoryItem>,
    semantic: Vec<NoNameSemanticMemoryItem>,
    narrative: Vec<NoNameNarrativeMemoryItem>,
}

impl NoNameMemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_working(&mut self, item: NoNameWorkingMemoryItem, max_len: usize) {
        self.working.push(item);
        if self.working.len() > max_len {
            let trim = self.working.len() - max_len;
            self.working.drain(0..trim);
        }
    }

    pub fn push_episodic(&mut self, item: NoNameEpisodicMemoryItem) {
        self.episodic.push(item);
    }

    pub fn upsert_semantic(&mut self, item: NoNameSemanticMemoryItem) {
        if let Some(existing) = self
            .semantic
            .iter_mut()
            .find(|fact| fact.fact_id == item.fact_id)
        {
            *existing = item;
            return;
        }
        self.semantic.push(item);
    }

    pub fn upsert_narrative(&mut self, item: NoNameNarrativeMemoryItem) {
        if let Some(existing) = self
            .narrative
            .iter_mut()
            .find(|note| note.note_id == item.note_id)
        {
            *existing = item;
            return;
        }
        self.narrative.push(item);
    }

    pub fn working(&self) -> &[NoNameWorkingMemoryItem] {
        &self.working
    }

    pub fn episodic(&self) -> &[NoNameEpisodicMemoryItem] {
        &self.episodic
    }

    pub fn episodic_by_chapter(&self, chapter_index: u32) -> Vec<NoNameEpisodicMemoryItem> {
        self.episodic
            .iter()
            .filter(|item| item.chapter_index == chapter_index)
            .cloned()
            .collect()
    }

    pub fn semantic(&self) -> &[NoNameSemanticMemoryItem] {
        &self.semantic
    }

    pub fn narrative(&self) -> &[NoNameNarrativeMemoryItem] {
        &self.narrative
    }

    pub fn retrieve(&self, query: &NoNameMemoryQuery) -> NoNameRetrievedMemories {
        retrieve_memories(
            query,
            self.working(),
            self.episodic(),
            self.semantic(),
            self.narrative(),
        )
    }

    pub fn retrieve_with_explanations(
        &self,
        query: &NoNameMemoryQuery,
    ) -> NoNameRetrievedMemoryReport {
        retrieve_memories_with_explanations(
            query,
            self.working(),
            self.episodic(),
            self.semantic(),
            self.narrative(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noname_memory_types::{
        NoNameMemoryImportance, NoNameNarrativeNoteType, NoNameNarrativeStatus,
    };
    use crate::noname_types::NoNameRole;

    #[test]
    fn working_memory_respects_max_len() {
        let mut store = NoNameMemoryStore::new();
        for index in 0..3 {
            store.push_working(
                NoNameWorkingMemoryItem {
                    memory_id: format!("mem-{}", index),
                    turn_id: format!("turn-{}", index),
                    source: "test".to_string(),
                    category: "recent_turn".to_string(),
                    summary: format!("summary-{}", index),
                    expires_at: None,
                    priority: 1,
                },
                2,
            );
        }

        assert_eq!(store.working().len(), 2);
        assert_eq!(store.working()[0].memory_id, "mem-1");
    }

    #[test]
    fn semantic_and_narrative_support_upsert() {
        let mut store = NoNameMemoryStore::new();
        store.upsert_semantic(NoNameSemanticMemoryItem {
            fact_id: "fact-1".to_string(),
            subject: "玩家".to_string(),
            predicate: "位于".to_string(),
            object: "山门".to_string(),
            confidence: 90,
            source: "test".to_string(),
            updated_at: 1,
            tags: vec![],
        });
        store.upsert_semantic(NoNameSemanticMemoryItem {
            fact_id: "fact-1".to_string(),
            subject: "玩家".to_string(),
            predicate: "位于".to_string(),
            object: "大殿".to_string(),
            confidence: 95,
            source: "test".to_string(),
            updated_at: 2,
            tags: vec![],
        });
        store.upsert_narrative(NoNameNarrativeMemoryItem {
            note_id: "note-1".to_string(),
            chapter_index: 1,
            arc_id: None,
            note_type: NoNameNarrativeNoteType::Goal,
            title: "守住山门".to_string(),
            summary: "抵御外敌".to_string(),
            status: NoNameNarrativeStatus::Active,
            related_entities: vec![],
            updated_at: 1,
        });
        store.push_episodic(NoNameEpisodicMemoryItem {
            memory_id: "event-1".to_string(),
            event_type: "battle".to_string(),
            timestamp: 1,
            chapter_index: 1,
            location_id: None,
            actors: vec!["玩家".to_string()],
            summary: "击退来敌".to_string(),
            detail_ref: None,
            importance: NoNameMemoryImportance::High,
        });

        assert_eq!(store.semantic()[0].object, "大殿");
        assert_eq!(store.narrative().len(), 1);
        assert_eq!(store.episodic().len(), 1);
    }

    #[test]
    fn store_can_delegate_structured_retrieval() {
        let mut store = NoNameMemoryStore::new();
        store.push_episodic(NoNameEpisodicMemoryItem {
            memory_id: "event-1".to_string(),
            event_type: "battle".to_string(),
            timestamp: 1,
            chapter_index: 1,
            location_id: Some("山门".to_string()),
            actors: vec!["青河长老".to_string()],
            summary: "青河长老在山门布阵".to_string(),
            detail_ref: None,
            importance: NoNameMemoryImportance::High,
        });

        let result = store.retrieve(&NoNameMemoryQuery {
            role: NoNameRole::NpcIntent,
            search_term: None,
            actor: Some("青河长老".to_string()),
            location: Some("山门".to_string()),
            goal: None,
            keyword: Some("布阵".to_string()),
            token_budget: 200,
            per_section_limit: 2,
        });

        assert_eq!(result.episodic.len(), 1);
        assert_eq!(result.episodic[0].memory_id, "event-1");
    }

    #[test]
    fn store_can_delegate_explainable_retrieval() {
        let mut store = NoNameMemoryStore::new();
        store.push_episodic(NoNameEpisodicMemoryItem {
            memory_id: "event-1".to_string(),
            event_type: "battle".to_string(),
            timestamp: 3,
            chapter_index: 1,
            location_id: Some("山门".to_string()),
            actors: vec!["青河长老".to_string()],
            summary: "青河长老在山门布阵".to_string(),
            detail_ref: None,
            importance: NoNameMemoryImportance::High,
        });

        let report = store.retrieve_with_explanations(&NoNameMemoryQuery {
            role: NoNameRole::NpcIntent,
            search_term: None,
            actor: Some("青河长老".to_string()),
            location: Some("山门".to_string()),
            goal: None,
            keyword: Some("布阵".to_string()),
            token_budget: 200,
            per_section_limit: 2,
        });

        assert_eq!(report.memories.episodic[0].memory_id, "event-1");
        assert_eq!(report.explanations[0].item_id, "event-1");
        assert_eq!(report.explanations[0].rank, 1);
    }

    #[test]
    fn episodic_by_chapter_filters_other_chapters() {
        let mut store = NoNameMemoryStore::new();
        store.push_episodic(NoNameEpisodicMemoryItem {
            memory_id: "event-1".to_string(),
            event_type: "scene".to_string(),
            timestamp: 1,
            chapter_index: 1,
            location_id: None,
            actors: vec!["player".to_string()],
            summary: "chapter one".to_string(),
            detail_ref: None,
            importance: NoNameMemoryImportance::Low,
        });
        store.push_episodic(NoNameEpisodicMemoryItem {
            memory_id: "event-2".to_string(),
            event_type: "scene".to_string(),
            timestamp: 2,
            chapter_index: 2,
            location_id: None,
            actors: vec!["player".to_string()],
            summary: "chapter two".to_string(),
            detail_ref: None,
            importance: NoNameMemoryImportance::Low,
        });

        let chapter_one = store.episodic_by_chapter(1);
        assert_eq!(chapter_one.len(), 1);
        assert_eq!(chapter_one[0].memory_id, "event-1");
    }
}
