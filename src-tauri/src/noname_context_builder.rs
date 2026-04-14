use crate::entity_store::{EntityQuery, EntityStore};
use crate::entity_types::EntityType;
use crate::noname_context_types::{
    NoNameContextBuildInput, NoNameContextPacket, NoNameContextSourceStat,
};
use crate::noname_memory_manager::NoNameMemoryManager;
use crate::noname_memory_retrieval::NoNameMemoryQuery;

pub fn build_context_packet(
    store: &EntityStore,
    memory_manager: &NoNameMemoryManager,
    input: &NoNameContextBuildInput,
) -> NoNameContextPacket {
    let retrieved = memory_manager.retrieve(&NoNameMemoryQuery {
        role: input.role,
        search_term: input.player_intent.clone(),
        token_budget: input.token_budget,
        per_section_limit: input.per_section_limit,
    });

    let mut hard_facts = retrieved
        .semantic
        .iter()
        .map(|item| format!("{} {} {}", item.subject, item.predicate, item.object))
        .collect::<Vec<_>>();
    let mut working_memory = retrieved
        .working
        .iter()
        .map(|item| item.summary.clone())
        .collect::<Vec<_>>();
    let mut episodic_memory = retrieved
        .episodic
        .iter()
        .map(|item| item.summary.clone())
        .collect::<Vec<_>>();
    let mut narrative_notes = retrieved
        .narrative
        .iter()
        .map(|item| format!("{}: {}", item.title, item.summary))
        .collect::<Vec<_>>();

    let mut chapter_summaries = memory_manager
        .active_notes()
        .into_iter()
        .take(input.per_section_limit)
        .map(|note| format!("{}: {}", note.title, note.summary))
        .collect::<Vec<_>>();

    let mut recent_context = input
        .recent_context_lines
        .iter()
        .rev()
        .take(input.per_section_limit)
        .cloned()
        .collect::<Vec<_>>();

    let total_estimated_before_trim = estimate_tokens(&hard_facts)
        + estimate_tokens(&working_memory)
        + estimate_tokens(&episodic_memory)
        + estimate_tokens(&narrative_notes)
        + estimate_tokens(&chapter_summaries)
        + estimate_tokens(&recent_context);

    let mut referenced_entities = Vec::new();
    for entity_type in [
        EntityType::Character,
        EntityType::Technique,
        EntityType::MapNode,
    ] {
        let query = EntityQuery {
            world_id: input.world_id.clone(),
            run_id: input.run_id.clone(),
            entity_type: Some(entity_type),
            keyword: input.player_intent.clone(),
        };
        for entity in store.list_by_query(&query).into_iter().take(3) {
            referenced_entities.push(format!("{:?}:{}", entity.entity_type, entity.entity_id));
            hard_facts.push(entity.payload.to_string());
        }
    }

    let mut used = 0usize;
    trim_to_budget(&mut hard_facts, input.token_budget, &mut used);
    trim_to_budget(
        &mut working_memory,
        input.token_budget.saturating_sub(used),
        &mut used,
    );
    trim_to_budget(
        &mut episodic_memory,
        input.token_budget.saturating_sub(used),
        &mut used,
    );
    trim_to_budget(
        &mut narrative_notes,
        input.token_budget.saturating_sub(used),
        &mut used,
    );
    trim_to_budget(
        &mut chapter_summaries,
        input.token_budget.saturating_sub(used),
        &mut used,
    );
    trim_to_budget(
        &mut recent_context,
        input.token_budget.saturating_sub(used),
        &mut used,
    );

    let compressed_summary = if total_estimated_before_trim > used {
        Some(build_compressed_summary(
            &hard_facts,
            &episodic_memory,
            &narrative_notes,
        ))
    } else {
        None
    };

    NoNameContextPacket {
        role: input.role,
        hard_facts,
        working_memory,
        episodic_memory,
        narrative_notes,
        chapter_summaries,
        recent_context,
        referenced_entities,
        compressed_summary,
        token_budget_used: used,
        source_stats: vec![
            NoNameContextSourceStat {
                source: "semantic".to_string(),
                count: retrieved.semantic.len(),
            },
            NoNameContextSourceStat {
                source: "working".to_string(),
                count: retrieved.working.len(),
            },
            NoNameContextSourceStat {
                source: "episodic".to_string(),
                count: retrieved.episodic.len(),
            },
            NoNameContextSourceStat {
                source: "narrative".to_string(),
                count: retrieved.narrative.len(),
            },
        ],
    }
}

fn trim_to_budget(items: &mut Vec<String>, remain_budget: usize, used: &mut usize) {
    let mut current = 0usize;
    items.retain(|line| {
        let estimate = (line.len() / 4).max(1);
        if current + estimate <= remain_budget {
            current += estimate;
            true
        } else {
            false
        }
    });
    *used += current;
}

fn estimate_tokens(items: &[String]) -> usize {
    items.iter().map(|line| (line.len() / 4).max(1)).sum()
}

fn build_compressed_summary(
    hard_facts: &[String],
    episodic_memory: &[String],
    narrative_notes: &[String],
) -> String {
    let fact = hard_facts
        .first()
        .cloned()
        .unwrap_or_else(|| "暂无硬事实".to_string());
    let event = episodic_memory
        .first()
        .cloned()
        .unwrap_or_else(|| "暂无关键事件".to_string());
    let note = narrative_notes
        .first()
        .cloned()
        .unwrap_or_else(|| "暂无叙事线索".to_string());
    format!("硬事实: {}; 近期事件: {}; 当前叙事: {}", fact, event, note)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_layers::{ChapterSummary, MemoryEntry, MemoryLayers, WorldFact};
    use crate::noname_memory_manager::NoNameMemoryManager;
    use crate::noname_memory_types::NoNameWorkingMemoryItem;
    use crate::noname_types::NoNameRole;

    #[test]
    fn context_builder_produces_context_packet_from_memory_manager() {
        let store = EntityStore::new();
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
        let mut manager = NoNameMemoryManager::new();
        manager.ingest_legacy_layers(&layers);
        manager.push_working_memory(
            NoNameWorkingMemoryItem {
                memory_id: "work-1".to_string(),
                turn_id: "turn-1".to_string(),
                source: "runtime".to_string(),
                category: "recent_turn".to_string(),
                summary: "最近一次行动是回山门".to_string(),
                expires_at: None,
                priority: 10,
            },
            8,
        );

        let packet = build_context_packet(
            &store,
            &manager,
            &NoNameContextBuildInput {
                role: NoNameRole::Director,
                world_id: "w1".to_string(),
                run_id: "r1".to_string(),
                scene_id: "s1".to_string(),
                character_ids: vec![],
                map_node_id: None,
                player_intent: Some("山门".to_string()),
                recent_context_lines: vec![
                    "远处钟声响起。".to_string(),
                    "弟子们回望山门。".to_string(),
                ],
                token_budget: 200,
                per_section_limit: 4,
            },
        );

        assert!(!packet.hard_facts.is_empty());
        assert!(!packet.working_memory.is_empty());
        assert!(!packet.episodic_memory.is_empty());
        assert!(!packet.narrative_notes.is_empty());
        assert!(!packet.source_stats.is_empty());
    }

    #[test]
    fn context_builder_can_emit_compressed_summary_under_tight_budget() {
        let store = EntityStore::new();
        let mut manager = NoNameMemoryManager::new();
        manager.push_working_memory(
            NoNameWorkingMemoryItem {
                memory_id: "work-1".to_string(),
                turn_id: "turn-1".to_string(),
                source: "runtime".to_string(),
                category: "recent_turn".to_string(),
                summary: "一段很长很长的上下文描述，用来触发压缩逻辑".repeat(4),
                expires_at: None,
                priority: 10,
            },
            8,
        );

        let packet = build_context_packet(
            &store,
            &manager,
            &NoNameContextBuildInput {
                role: NoNameRole::Director,
                world_id: "w1".to_string(),
                run_id: "r1".to_string(),
                scene_id: "s1".to_string(),
                character_ids: vec![],
                map_node_id: None,
                player_intent: None,
                recent_context_lines: vec!["又一段很长很长的上下文描述".repeat(5)],
                token_budget: 20,
                per_section_limit: 4,
            },
        );

        assert!(packet.compressed_summary.is_some());
    }
}
