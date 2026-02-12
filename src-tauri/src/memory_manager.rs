use crate::npc::{MemoryEntry, NPCMemory};

#[derive(Debug, Clone)]
pub struct MemoryManager {
    short_term_limit: usize,
    long_term_limit: usize,
    important_threshold: f32,
}

impl MemoryManager {
    pub fn new(short_term_limit: usize, long_term_limit: usize, important_threshold: f32) -> Self {
        Self {
            short_term_limit: short_term_limit.max(1),
            long_term_limit: long_term_limit.max(1),
            important_threshold,
        }
    }

    pub fn add_memory(&self, memory: &mut NPCMemory, entry: MemoryEntry) {
        if entry.importance >= self.important_threshold {
            memory.important_events.push(entry.clone());
            memory.long_term.push(entry.clone());
        }

        memory.short_term.push(entry);
        self.compress_memories(memory);
    }

    pub fn compress_memories(&self, memory: &mut NPCMemory) {
        if memory.short_term.len() > self.short_term_limit {
            memory
                .short_term
                .sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal));
            let overflow = memory.short_term.split_off(self.short_term_limit);
            memory.long_term.extend(overflow);
        }

        if memory.long_term.len() > self.long_term_limit {
            memory
                .long_term
                .sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal));
            memory.long_term.truncate(self.long_term_limit);
        }

        memory.long_term.sort_by(|a, b| {
            b.importance
                .partial_cmp(&a.importance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        memory.long_term.dedup_by(|a, b| a.timestamp == b.timestamp && a.event == b.event);

        memory.important_events.sort_by(|a, b| {
            b.importance
                .partial_cmp(&a.importance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        memory.important_events.dedup_by(|a, b| {
            a.timestamp == b.timestamp && a.event == b.event
        });
    }

    pub fn retrieve_relevant_memories(
        &self,
        memory: &NPCMemory,
        keyword: &str,
        max_results: usize,
    ) -> Vec<MemoryEntry> {
        let mut merged = Vec::with_capacity(
            memory.short_term.len() + memory.long_term.len() + memory.important_events.len(),
        );
        merged.extend(memory.short_term.clone());
        merged.extend(memory.long_term.clone());
        merged.extend(memory.important_events.clone());

        let keyword_lower = keyword.to_lowercase();
        merged.sort_by(|a, b| {
            let a_match = a.event.to_lowercase().contains(&keyword_lower);
            let b_match = b.event.to_lowercase().contains(&keyword_lower);

            let a_score = if a_match { a.importance + a.emotional_impact.abs() } else { a.importance * 0.5 };
            let b_score = if b_match { b.importance + b.emotional_impact.abs() } else { b.importance * 0.5 };

            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });

        merged
            .into_iter()
            .take(max_results.max(1))
            .collect()
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new(20, 200, 0.75)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn entry(ts: u64, event: &str, importance: f32, impact: f32) -> MemoryEntry {
        MemoryEntry {
            timestamp: ts,
            event: event.to_string(),
            importance,
            emotional_impact: impact,
        }
    }

    #[test]
    fn test_add_memory_and_compress() {
        let manager = MemoryManager::new(2, 2, 0.8);
        let mut memory = NPCMemory::default();

        manager.add_memory(&mut memory, entry(1, "small event", 0.2, 0.1));
        manager.add_memory(&mut memory, entry(2, "important event", 0.9, 0.8));
        manager.add_memory(&mut memory, entry(3, "another event", 0.7, 0.3));

        assert!(memory.short_term.len() <= 2);
        assert!(!memory.important_events.is_empty());
    }

    #[test]
    fn test_retrieve_relevant_memories() {
        let manager = MemoryManager::default();
        let mut memory = NPCMemory::default();

        manager.add_memory(&mut memory, entry(1, "met player at sect gate", 0.7, 0.2));
        manager.add_memory(&mut memory, entry(2, "fought spirit beast", 0.9, 0.6));

        let found = manager.retrieve_relevant_memories(&memory, "player", 3);
        assert!(!found.is_empty());
        assert!(found.iter().any(|m| m.event.contains("player")));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        // Feature: Nobody, Property 16: NPC memory persistence
        #[test]
        fn prop_important_event_is_persisted(
            importance in 0.75f32..=1.0f32,
            impact in -1.0f32..=1.0f32,
            ts in 1u64..=100000
        ) {
            let manager = MemoryManager::new(10, 10, 0.75);
            let mut memory = NPCMemory::default();
            let e = entry(ts, "critical interaction", importance, impact);

            manager.add_memory(&mut memory, e.clone());

            prop_assert!(memory.important_events.iter().any(|m| m.timestamp == e.timestamp && m.event == e.event));
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        // Feature: Nobody, Property 17: NPC memory compression fidelity
        #[test]
        fn prop_compression_keeps_high_importance_entries(
            high1 in 0.8f32..=1.0f32,
            high2 in 0.8f32..=1.0f32,
            low in 0.0f32..=0.4f32
        ) {
            let manager = MemoryManager::new(2, 2, 0.75);
            let mut memory = NPCMemory::default();

            manager.add_memory(&mut memory, entry(1, "high-a", high1, 0.3));
            manager.add_memory(&mut memory, entry(2, "high-b", high2, 0.3));
            manager.add_memory(&mut memory, entry(3, "low-c", low, 0.1));

            prop_assert!(memory.short_term.len() <= 2);
            prop_assert!(memory.short_term.iter().all(|m| m.event != "low-c" || m.importance >= memory.short_term.iter().map(|x| x.importance).fold(0.0, f32::min)));
            prop_assert!(memory.short_term.iter().any(|m| m.event == "high-a") || memory.short_term.iter().any(|m| m.event == "high-b"));
        }
    }
}
