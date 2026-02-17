use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryEntry {
    pub event_id: String,
    pub summary: String,
    pub turn: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterSummary {
    pub chapter_id: String,
    pub title: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldFact {
    pub fact_id: String,
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

#[derive(Default)]
pub struct MemoryLayers {
    pub recent_events: Vec<MemoryEntry>,
    pub chapter_summaries: Vec<ChapterSummary>,
    pub world_facts: Vec<WorldFact>,
}

impl MemoryLayers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_recent_event(&mut self, event: MemoryEntry, max_len: usize) {
        self.recent_events.push(event);
        if self.recent_events.len() > max_len {
            let trim = self.recent_events.len() - max_len;
            self.recent_events.drain(0..trim);
        }
    }

    pub fn upsert_chapter_summary(&mut self, chapter: ChapterSummary) {
        if let Some(existing) = self
            .chapter_summaries
            .iter_mut()
            .find(|c| c.chapter_id == chapter.chapter_id)
        {
            *existing = chapter;
            return;
        }
        self.chapter_summaries.push(chapter);
    }

    pub fn upsert_world_fact(&mut self, fact: WorldFact) {
        if let Some(existing) = self.world_facts.iter_mut().find(|f| f.fact_id == fact.fact_id) {
            *existing = fact;
            return;
        }
        self.world_facts.push(fact);
    }
}
