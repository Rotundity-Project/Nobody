use crate::noname_memory_types::{
    NoNameEpisodicMemoryItem, NoNameNarrativeMemoryItem, NoNameNarrativeStatus,
    NoNameSemanticMemoryItem, NoNameWorkingMemoryItem,
};
use crate::noname_types::NoNameRole;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoNameMemoryQuery {
    pub role: NoNameRole,
    pub search_term: Option<String>,
    pub token_budget: usize,
    pub per_section_limit: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NoNameRetrievedMemories {
    pub working: Vec<NoNameWorkingMemoryItem>,
    pub episodic: Vec<NoNameEpisodicMemoryItem>,
    pub semantic: Vec<NoNameSemanticMemoryItem>,
    pub narrative: Vec<NoNameNarrativeMemoryItem>,
}

pub fn retrieve_memories(
    query: &NoNameMemoryQuery,
    working: &[NoNameWorkingMemoryItem],
    episodic: &[NoNameEpisodicMemoryItem],
    semantic: &[NoNameSemanticMemoryItem],
    narrative: &[NoNameNarrativeMemoryItem],
) -> NoNameRetrievedMemories {
    let search_term = query.search_term.as_deref().map(str::to_lowercase);

    let mut working_items = working.to_vec();
    working_items.sort_by(|a, b| b.priority.cmp(&a.priority));
    working_items.truncate(query.per_section_limit);

    let mut episodic_items = episodic
        .iter()
        .filter(|item| matches_term(&item.summary, search_term.as_deref()))
        .cloned()
        .collect::<Vec<_>>();
    episodic_items.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    episodic_items.truncate(query.per_section_limit);

    let mut semantic_items = semantic
        .iter()
        .filter(|item| {
            matches_term(
                &format!(
                    "{} {} {} {}",
                    item.subject,
                    item.predicate,
                    item.object,
                    item.tags.join(" ")
                ),
                search_term.as_deref(),
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    semantic_items.sort_by(|a, b| b.confidence.cmp(&a.confidence));
    semantic_items.truncate(query.per_section_limit);

    let mut narrative_items = narrative
        .iter()
        .filter(|item| item.status == NoNameNarrativeStatus::Active)
        .filter(|item| {
            matches_term(
                &format!("{} {}", item.title, item.summary),
                search_term.as_deref(),
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    narrative_items.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    narrative_items.truncate(query.per_section_limit);

    NoNameRetrievedMemories {
        working: working_items,
        episodic: episodic_items,
        semantic: semantic_items,
        narrative: narrative_items,
    }
}

fn matches_term(content: &str, search_term: Option<&str>) -> bool {
    match search_term {
        Some(term) => content.to_lowercase().contains(term),
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noname_memory_types::{
        NoNameMemoryImportance, NoNameNarrativeNoteType, NoNameNarrativeStatus,
    };

    #[test]
    fn retrieval_filters_and_orders_memory_sections() {
        let result = retrieve_memories(
            &NoNameMemoryQuery {
                role: NoNameRole::Director,
                search_term: Some("山门".to_string()),
                token_budget: 200,
                per_section_limit: 2,
            },
            &[NoNameWorkingMemoryItem {
                memory_id: "work-1".to_string(),
                turn_id: "turn-1".to_string(),
                source: "test".to_string(),
                category: "recent_turn".to_string(),
                summary: "最近玩家回到了山门".to_string(),
                expires_at: None,
                priority: 9,
            }],
            &[NoNameEpisodicMemoryItem {
                memory_id: "episode-1".to_string(),
                event_type: "travel".to_string(),
                timestamp: 2,
                chapter_index: 1,
                location_id: Some("mountain-gate".to_string()),
                actors: vec!["玩家".to_string()],
                summary: "玩家回到山门".to_string(),
                detail_ref: None,
                importance: NoNameMemoryImportance::Medium,
            }],
            &[NoNameSemanticMemoryItem {
                fact_id: "fact-1".to_string(),
                subject: "玩家".to_string(),
                predicate: "位于".to_string(),
                object: "山门".to_string(),
                confidence: 80,
                source: "test".to_string(),
                updated_at: 1,
                tags: vec![],
            }],
            &[NoNameNarrativeMemoryItem {
                note_id: "note-1".to_string(),
                chapter_index: 1,
                arc_id: None,
                note_type: NoNameNarrativeNoteType::Conflict,
                title: "山门危机".to_string(),
                summary: "强敌逼近".to_string(),
                status: NoNameNarrativeStatus::Active,
                related_entities: vec!["玩家".to_string()],
                updated_at: 3,
            }],
        );

        assert_eq!(result.working.len(), 1);
        assert_eq!(result.episodic.len(), 1);
        assert_eq!(result.semantic.len(), 1);
        assert_eq!(result.narrative.len(), 1);
    }
}
