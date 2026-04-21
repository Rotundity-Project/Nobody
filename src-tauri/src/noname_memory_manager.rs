use crate::memory_layers::MemoryLayers;
use crate::noname_memory_compaction::{
    NoNameChapterCompactionInput, NoNameCompactionSummary, NoNameMemoryCompactionService,
    NoNameTraceCompactionInput, NoNameTurnCompactionInput,
};
use crate::noname_memory_retrieval::{
    NoNameMemoryQuery, NoNameRetrievedMemories, NoNameRetrievedMemoryReport,
};
use crate::noname_memory_store::NoNameMemoryStore;
use crate::noname_memory_types::{
    NoNameEpisodicMemoryItem, NoNameNarrativeMemoryItem, NoNameSemanticMemoryItem,
    NoNameWorkingMemoryItem,
};
use crate::noname_note_store::{
    NoNameChapterNoteReview, NoNameNoteLifecycleAction, NoNameNoteLifecycleResult, NoNameNoteStore,
    NoNameNoteUpdate,
};

#[derive(Debug, Clone)]
pub struct NoNameMemoryManager {
    store: NoNameMemoryStore,
    notes: NoNameNoteStore,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoNameNoteRetrievalContext {
    pub note_id: String,
    pub title: String,
    pub related_entities: Vec<String>,
    pub derived_actor: Option<String>,
    pub derived_keyword: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoNameNoteAugmentedRetrievalReport {
    pub memories: NoNameRetrievedMemories,
    pub note_contexts: Vec<NoNameNoteRetrievalContext>,
    pub explanations: Vec<crate::noname_memory_retrieval::NoNameRetrievalExplanation>,
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

    pub fn reopen_note(
        &mut self,
        note_id: &str,
        updated_at: u64,
    ) -> Option<NoNameNarrativeMemoryItem> {
        let updated = self.notes.reopen(note_id, updated_at)?;
        self.store.upsert_narrative(updated.clone());
        Some(updated)
    }

    pub fn apply_note_lifecycle_action(
        &mut self,
        note_id: &str,
        action: NoNameNoteLifecycleAction,
        updated_at: u64,
    ) -> Option<NoNameNoteLifecycleResult> {
        let result = self
            .notes
            .apply_lifecycle_action(note_id, action, updated_at)?;
        self.store.upsert_narrative(result.note.clone());
        Some(result)
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

    pub fn retrieve_with_active_note_context(
        &self,
        query: &NoNameMemoryQuery,
    ) -> NoNameNoteAugmentedRetrievalReport {
        let mut report = self.store.retrieve_with_explanations(query);
        let note_contexts = self.matching_note_contexts(query);

        for context in &note_contexts {
            let augmented_query = context.augment_query(query);
            let augmented_report = self.store.retrieve_with_explanations(&augmented_query);
            merge_note_augmented_report(&mut report, augmented_report, context, query);
        }

        NoNameNoteAugmentedRetrievalReport {
            memories: report.memories,
            note_contexts,
            explanations: report.explanations,
        }
    }

    pub fn compact_turn_memory(&self, input: NoNameTurnCompactionInput) -> NoNameCompactionSummary {
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

    fn matching_note_contexts(&self, query: &NoNameMemoryQuery) -> Vec<NoNameNoteRetrievalContext> {
        if !query.has_structured_filters() {
            return Vec::new();
        }

        self.notes
            .list_active()
            .into_iter()
            .filter(|note| note_matches_query(note, query))
            .take(query.per_section_limit)
            .map(NoNameNoteRetrievalContext::from_note)
            .collect()
    }
}

impl NoNameNoteRetrievalContext {
    fn from_note(note: NoNameNarrativeMemoryItem) -> Self {
        let derived_actor = note.related_entities.first().cloned();
        let derived_keyword = first_non_empty(&[note.title.as_str(), note.summary.as_str()]);

        Self {
            note_id: note.note_id,
            title: note.title,
            related_entities: note.related_entities,
            derived_actor,
            derived_keyword,
        }
    }

    fn augment_query(&self, query: &NoNameMemoryQuery) -> NoNameMemoryQuery {
        let mut augmented = query.clone();
        if augmented.actor.is_none() {
            augmented.actor = self.derived_actor.clone();
        }
        if augmented.keyword.is_none() && augmented.search_term.is_none() {
            augmented.keyword = self.derived_keyword.clone();
        }
        if augmented.goal.is_none() {
            augmented.goal = self.derived_keyword.clone();
        }
        augmented
    }
}

fn merge_note_augmented_report(
    target: &mut NoNameRetrievedMemoryReport,
    incoming: NoNameRetrievedMemoryReport,
    context: &NoNameNoteRetrievalContext,
    query: &NoNameMemoryQuery,
) {
    merge_unique_by_id(
        &mut target.memories.working,
        incoming.memories.working,
        query.per_section_limit,
        |item| item.memory_id.clone(),
    );
    merge_unique_by_id(
        &mut target.memories.episodic,
        incoming.memories.episodic,
        query.per_section_limit,
        |item| item.memory_id.clone(),
    );
    merge_unique_by_id(
        &mut target.memories.semantic,
        incoming.memories.semantic,
        query.per_section_limit,
        |item| item.fact_id.clone(),
    );
    merge_unique_by_id(
        &mut target.memories.narrative,
        incoming.memories.narrative,
        query.per_section_limit,
        |item| item.note_id.clone(),
    );

    for mut explanation in incoming.explanations {
        if target.explanations.iter().any(|existing| {
            existing.section == explanation.section && existing.item_id == explanation.item_id
        }) {
            continue;
        }
        explanation
            .reasons
            .push(format!("note_context={}", context.note_id));
        target.explanations.push(explanation);
    }
}

fn merge_unique_by_id<T, F>(target: &mut Vec<T>, incoming: Vec<T>, limit: usize, item_id: F)
where
    F: Fn(&T) -> String,
{
    for item in incoming {
        if target.len() >= limit {
            break;
        }
        let incoming_id = item_id(&item);
        if !target
            .iter()
            .any(|existing| item_id(existing) == incoming_id)
        {
            target.push(item);
        }
    }
}

fn note_matches_query(note: &NoNameNarrativeMemoryItem, query: &NoNameMemoryQuery) -> bool {
    let fields = [
        note.title.as_str(),
        note.summary.as_str(),
        note.arc_id.as_deref().unwrap_or(""),
    ];

    query
        .search_term
        .as_deref()
        .is_some_and(|term| note_contains(&fields, &note.related_entities, term))
        || query
            .keyword
            .as_deref()
            .is_some_and(|term| note_contains(&fields, &note.related_entities, term))
        || query
            .goal
            .as_deref()
            .is_some_and(|term| note_contains(&fields, &note.related_entities, term))
        || query
            .actor
            .as_deref()
            .is_some_and(|term| note_contains(&fields, &note.related_entities, term))
        || query
            .location
            .as_deref()
            .is_some_and(|term| note_contains(&fields, &note.related_entities, term))
}

fn note_contains(fields: &[&str], related_entities: &[String], term: &str) -> bool {
    let normalized_term = term.to_lowercase();
    fields
        .iter()
        .any(|field| field.to_lowercase().contains(&normalized_term))
        || related_entities
            .iter()
            .any(|entity| entity.to_lowercase().contains(&normalized_term))
}

fn first_non_empty(values: &[&str]) -> Option<String> {
    values
        .iter()
        .map(|value| value.trim())
        .find(|value| !value.is_empty())
        .map(str::to_string)
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

    #[test]
    fn manager_applies_note_lifecycle_actions_and_syncs_store() {
        let mut manager = NoNameMemoryManager::new();
        manager.upsert_narrative_memory(NoNameNarrativeMemoryItem {
            note_id: "note-1".to_string(),
            chapter_index: 1,
            arc_id: None,
            note_type: NoNameNarrativeNoteType::UnresolvedThread,
            title: "Broken Ward".to_string(),
            summary: "Find who broke the mountain gate ward.".to_string(),
            status: NoNameNarrativeStatus::Active,
            related_entities: vec!["player".to_string()],
            updated_at: 1,
        });

        let resolved = manager
            .apply_note_lifecycle_action("note-1", NoNameNoteLifecycleAction::Resolve, 2)
            .expect("manager should resolve note");
        assert_eq!(resolved.next_status, NoNameNarrativeStatus::Resolved);

        manager.reopen_note("note-1", 3);
        let reopened_notes = manager.retrieve(&NoNameMemoryQuery {
            role: NoNameRole::Director,
            search_term: Some("Broken Ward".to_string()),
            actor: None,
            location: None,
            goal: None,
            keyword: None,
            token_budget: 200,
            per_section_limit: 2,
        });

        assert_eq!(reopened_notes.narrative.len(), 1);
        assert_eq!(
            reopened_notes.narrative[0].status,
            NoNameNarrativeStatus::Active
        );
    }

    #[test]
    fn active_notes_can_expand_retrieval_recall() {
        let mut manager = NoNameMemoryManager::new();
        manager.push_episodic_memory(NoNameEpisodicMemoryItem {
            memory_id: "event-qinghe".to_string(),
            event_type: "dialogue".to_string(),
            timestamp: 4,
            chapter_index: 1,
            location_id: Some("mountain_gate".to_string()),
            actors: vec!["Elder Qinghe".to_string()],
            summary: "Elder Qinghe hesitated before naming the saboteur.".to_string(),
            detail_ref: None,
            importance: crate::noname_memory_types::NoNameMemoryImportance::High,
        });
        manager.upsert_narrative_memory(NoNameNarrativeMemoryItem {
            note_id: "note-ward".to_string(),
            chapter_index: 1,
            arc_id: None,
            note_type: NoNameNarrativeNoteType::UnresolvedThread,
            title: "Broken Ward".to_string(),
            summary: "The saboteur clue points back to Elder Qinghe.".to_string(),
            status: NoNameNarrativeStatus::Active,
            related_entities: vec!["Elder Qinghe".to_string()],
            updated_at: 5,
        });

        let query = NoNameMemoryQuery {
            role: NoNameRole::Director,
            search_term: Some("ward".to_string()),
            actor: None,
            location: None,
            goal: None,
            keyword: None,
            token_budget: 200,
            per_section_limit: 4,
        };
        let base = manager.retrieve(&query);
        let expanded = manager.retrieve_with_active_note_context(&query);

        assert!(base.episodic.is_empty());
        assert_eq!(expanded.note_contexts.len(), 1);
        assert_eq!(expanded.note_contexts[0].note_id, "note-ward");
        assert_eq!(
            expanded.note_contexts[0].derived_actor.as_deref(),
            Some("Elder Qinghe")
        );
        assert_eq!(expanded.memories.episodic.len(), 1);
        assert_eq!(expanded.memories.episodic[0].memory_id, "event-qinghe");
        assert!(expanded.explanations.iter().any(|item| {
            item.item_id == "event-qinghe"
                && item
                    .reasons
                    .iter()
                    .any(|reason| reason == "note_context=note-ward")
        }));
    }

    #[test]
    fn archived_notes_do_not_expand_retrieval_recall() {
        let mut manager = NoNameMemoryManager::new();
        manager.push_episodic_memory(NoNameEpisodicMemoryItem {
            memory_id: "event-qinghe".to_string(),
            event_type: "dialogue".to_string(),
            timestamp: 4,
            chapter_index: 1,
            location_id: None,
            actors: vec!["Elder Qinghe".to_string()],
            summary: "Elder Qinghe mentioned a saboteur.".to_string(),
            detail_ref: None,
            importance: crate::noname_memory_types::NoNameMemoryImportance::High,
        });
        manager.upsert_narrative_memory(NoNameNarrativeMemoryItem {
            note_id: "note-ward".to_string(),
            chapter_index: 1,
            arc_id: None,
            note_type: NoNameNarrativeNoteType::UnresolvedThread,
            title: "Broken Ward".to_string(),
            summary: "The saboteur clue points back to Elder Qinghe.".to_string(),
            status: NoNameNarrativeStatus::Archived,
            related_entities: vec!["Elder Qinghe".to_string()],
            updated_at: 5,
        });

        let expanded = manager.retrieve_with_active_note_context(&NoNameMemoryQuery {
            role: NoNameRole::Director,
            search_term: Some("ward".to_string()),
            actor: None,
            location: None,
            goal: None,
            keyword: None,
            token_budget: 200,
            per_section_limit: 4,
        });

        assert!(expanded.note_contexts.is_empty());
        assert!(expanded.memories.episodic.is_empty());
    }
}
