use crate::noname_memory_types::{NoNameNarrativeMemoryItem, NoNameNarrativeStatus};

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

    pub fn list_active(&self) -> Vec<NoNameNarrativeMemoryItem> {
        self.notes
            .iter()
            .filter(|item| item.status == NoNameNarrativeStatus::Active)
            .cloned()
            .collect()
    }
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
}
