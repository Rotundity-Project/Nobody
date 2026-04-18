use crate::noname_memory_types::{NoNameNarrativeMemoryItem, NoNameNarrativeStatus};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NoNameNoteUpdate {
    pub title: Option<String>,
    pub summary: Option<String>,
    pub status: Option<NoNameNarrativeStatus>,
    pub related_entities: Option<Vec<String>>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NoNameChapterNoteReview {
    pub chapter_index: u32,
    pub active_note_ids: Vec<String>,
    pub archived_note_ids: Vec<String>,
    pub goal_note_ids: Vec<String>,
    pub conflict_note_ids: Vec<String>,
    pub foreshadowing_note_ids: Vec<String>,
    pub unresolved_thread_note_ids: Vec<String>,
    pub character_arc_note_ids: Vec<String>,
    pub carried_forward_count: usize,
    pub archived_from_resolved_count: usize,
    pub updated_at: u64,
}

#[derive(Debug, Default, Clone)]
pub struct NoNameNoteStore {
    notes: Vec<NoNameNarrativeMemoryItem>,
}

impl NoNameNoteStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upsert(&mut self, note: NoNameNarrativeMemoryItem) {
        if let Some(existing) = self
            .notes
            .iter_mut()
            .find(|item| item.note_id == note.note_id)
        {
            *existing = note;
            return;
        }
        self.notes.push(note);
    }

    pub fn get(&self, note_id: &str) -> Option<&NoNameNarrativeMemoryItem> {
        self.notes.iter().find(|item| item.note_id == note_id)
    }

    pub fn list_all(&self) -> Vec<NoNameNarrativeMemoryItem> {
        self.notes.clone()
    }

    pub fn list_active(&self) -> Vec<NoNameNarrativeMemoryItem> {
        self.notes
            .iter()
            .filter(|item| item.status == NoNameNarrativeStatus::Active)
            .cloned()
            .collect()
    }

    pub fn list_by_chapter(&self, chapter_index: u32) -> Vec<NoNameNarrativeMemoryItem> {
        self.notes
            .iter()
            .filter(|item| item.chapter_index == chapter_index)
            .cloned()
            .collect()
    }

    pub fn list_by_status(&self, status: NoNameNarrativeStatus) -> Vec<NoNameNarrativeMemoryItem> {
        self.notes
            .iter()
            .filter(|item| item.status == status)
            .cloned()
            .collect()
    }

    pub fn update(
        &mut self,
        note_id: &str,
        update: NoNameNoteUpdate,
    ) -> Option<NoNameNarrativeMemoryItem> {
        let note = self.notes.iter_mut().find(|item| item.note_id == note_id)?;
        if let Some(title) = update.title {
            note.title = title;
        }
        if let Some(summary) = update.summary {
            note.summary = summary;
        }
        if let Some(status) = update.status {
            note.status = status;
        }
        if let Some(related_entities) = update.related_entities {
            note.related_entities = dedupe_strings(related_entities);
        }
        note.updated_at = update.updated_at;
        Some(note.clone())
    }

    pub fn set_status(
        &mut self,
        note_id: &str,
        status: NoNameNarrativeStatus,
        updated_at: u64,
    ) -> Option<NoNameNarrativeMemoryItem> {
        self.update(
            note_id,
            NoNameNoteUpdate {
                status: Some(status),
                updated_at,
                ..NoNameNoteUpdate::default()
            },
        )
    }

    pub fn resolve(&mut self, note_id: &str, updated_at: u64) -> Option<NoNameNarrativeMemoryItem> {
        self.set_status(note_id, NoNameNarrativeStatus::Resolved, updated_at)
    }

    pub fn close(&mut self, note_id: &str, updated_at: u64) -> Option<NoNameNarrativeMemoryItem> {
        self.resolve(note_id, updated_at)
    }

    pub fn archive(&mut self, note_id: &str, updated_at: u64) -> Option<NoNameNarrativeMemoryItem> {
        self.set_status(note_id, NoNameNarrativeStatus::Archived, updated_at)
    }

    pub fn organize_chapter_end(
        &mut self,
        chapter_index: u32,
        updated_at: u64,
    ) -> NoNameChapterNoteReview {
        let mut archived_from_resolved_count = 0;
        for note in self
            .notes
            .iter_mut()
            .filter(|item| item.chapter_index == chapter_index)
        {
            if note.status == NoNameNarrativeStatus::Resolved {
                note.status = NoNameNarrativeStatus::Archived;
                note.updated_at = updated_at;
                archived_from_resolved_count += 1;
            }
        }

        let mut review = self.review_chapter(chapter_index, updated_at);
        review.archived_from_resolved_count = archived_from_resolved_count;
        review
    }

    pub fn review_chapter(&self, chapter_index: u32, updated_at: u64) -> NoNameChapterNoteReview {
        let mut review = NoNameChapterNoteReview {
            chapter_index,
            updated_at,
            ..NoNameChapterNoteReview::default()
        };

        for note in self
            .notes
            .iter()
            .filter(|item| item.chapter_index == chapter_index)
        {
            match note.status {
                NoNameNarrativeStatus::Active => review.active_note_ids.push(note.note_id.clone()),
                NoNameNarrativeStatus::Resolved | NoNameNarrativeStatus::Archived => {
                    review.archived_note_ids.push(note.note_id.clone())
                }
            }

            match note.note_type {
                crate::noname_memory_types::NoNameNarrativeNoteType::Goal => {
                    review.goal_note_ids.push(note.note_id.clone())
                }
                crate::noname_memory_types::NoNameNarrativeNoteType::Conflict => {
                    review.conflict_note_ids.push(note.note_id.clone())
                }
                crate::noname_memory_types::NoNameNarrativeNoteType::Foreshadowing => {
                    review.foreshadowing_note_ids.push(note.note_id.clone())
                }
                crate::noname_memory_types::NoNameNarrativeNoteType::UnresolvedThread => {
                    review.unresolved_thread_note_ids.push(note.note_id.clone())
                }
                crate::noname_memory_types::NoNameNarrativeNoteType::CharacterArc => {
                    review.character_arc_note_ids.push(note.note_id.clone())
                }
            }
        }

        review.carried_forward_count = review.active_note_ids.len();
        review
    }
}

fn dedupe_strings(values: Vec<String>) -> Vec<String> {
    let mut deduped = Vec::new();
    for value in values {
        let cleaned = value.split_whitespace().collect::<Vec<_>>().join(" ");
        if !cleaned.is_empty() && !deduped.iter().any(|item| item == &cleaned) {
            deduped.push(cleaned);
        }
    }
    deduped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noname_memory_types::{NoNameNarrativeNoteType, NoNameNarrativeStatus};

    #[test]
    fn note_store_returns_only_active_notes() {
        let mut store = NoNameNoteStore::new();
        store.upsert(NoNameNarrativeMemoryItem {
            note_id: "note-1".to_string(),
            chapter_index: 1,
            arc_id: None,
            note_type: NoNameNarrativeNoteType::Conflict,
            title: "山门危机".to_string(),
            summary: "敌人逼近".to_string(),
            status: NoNameNarrativeStatus::Active,
            related_entities: vec![],
            updated_at: 1,
        });
        store.upsert(NoNameNarrativeMemoryItem {
            note_id: "note-2".to_string(),
            chapter_index: 1,
            arc_id: None,
            note_type: NoNameNarrativeNoteType::Goal,
            title: "旧目标".to_string(),
            summary: "已完成".to_string(),
            status: NoNameNarrativeStatus::Resolved,
            related_entities: vec![],
            updated_at: 2,
        });

        assert_eq!(store.list_active().len(), 1);
    }

    #[test]
    fn note_store_updates_resolves_and_archives_notes() {
        let mut store = NoNameNoteStore::new();
        store.upsert(note("note-1", NoNameNarrativeNoteType::Goal));

        let updated = store
            .update(
                "note-1",
                NoNameNoteUpdate {
                    title: Some("Hold Gate".to_string()),
                    summary: Some("Keep the mountain gate secure.".to_string()),
                    related_entities: Some(vec![
                        "player".to_string(),
                        "player".to_string(),
                        "elder".to_string(),
                    ]),
                    updated_at: 10,
                    ..NoNameNoteUpdate::default()
                },
            )
            .expect("note should update");

        assert_eq!(updated.title, "Hold Gate");
        assert_eq!(updated.related_entities, vec!["player", "elder"]);

        let resolved = store.resolve("note-1", 11).expect("note should resolve");
        assert_eq!(resolved.status, NoNameNarrativeStatus::Resolved);

        let archived = store.archive("note-1", 12).expect("note should archive");
        assert_eq!(archived.status, NoNameNarrativeStatus::Archived);
        assert_eq!(
            store.list_by_status(NoNameNarrativeStatus::Archived).len(),
            1
        );
    }

    #[test]
    fn chapter_end_review_archives_resolved_and_carries_active_notes() {
        let mut store = NoNameNoteStore::new();
        store.upsert(note("goal-1", NoNameNarrativeNoteType::Goal));
        store.upsert(note("conflict-1", NoNameNarrativeNoteType::Conflict));
        store.upsert(note("thread-1", NoNameNarrativeNoteType::UnresolvedThread));
        store.upsert(note("arc-1", NoNameNarrativeNoteType::CharacterArc));
        store.upsert(note("shadow-1", NoNameNarrativeNoteType::Foreshadowing));
        store.resolve("goal-1", 2);

        let review = store.organize_chapter_end(1, 3);

        assert_eq!(review.archived_from_resolved_count, 1);
        assert!(review.archived_note_ids.contains(&"goal-1".to_string()));
        assert!(review.active_note_ids.contains(&"conflict-1".to_string()));
        assert_eq!(review.goal_note_ids, vec!["goal-1"]);
        assert_eq!(review.conflict_note_ids, vec!["conflict-1"]);
        assert_eq!(review.unresolved_thread_note_ids, vec!["thread-1"]);
        assert_eq!(review.character_arc_note_ids, vec!["arc-1"]);
        assert_eq!(review.foreshadowing_note_ids, vec!["shadow-1"]);
        assert_eq!(review.carried_forward_count, 4);
    }

    fn note(note_id: &str, note_type: NoNameNarrativeNoteType) -> NoNameNarrativeMemoryItem {
        NoNameNarrativeMemoryItem {
            note_id: note_id.to_string(),
            chapter_index: 1,
            arc_id: None,
            note_type,
            title: note_id.to_string(),
            summary: format!("{} summary", note_id),
            status: NoNameNarrativeStatus::Active,
            related_entities: Vec::new(),
            updated_at: 1,
        }
    }
}
