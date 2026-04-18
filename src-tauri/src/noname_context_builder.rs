use crate::entity_store::{EntityQuery, EntityStore};
use crate::entity_types::EntityType;
use crate::noname_context_types::{
    NoNameContextBuildInput, NoNameContextPacket, NoNameContextSourceStat, NoNameRoleContextPacket,
};
use crate::noname_memory_manager::NoNameMemoryManager;
use crate::noname_memory_retrieval::NoNameMemoryQuery;
use crate::noname_types::NoNameRole;

pub fn build_context_packet(
    store: &EntityStore,
    memory_manager: &NoNameMemoryManager,
    input: &NoNameContextBuildInput,
) -> NoNameContextPacket {
    let retrieved = memory_manager.retrieve(&NoNameMemoryQuery {
        role: input.role,
        search_term: input.player_intent.clone(),
        actor: None,
        location: input.map_node_id.clone(),
        goal: input.player_intent.clone(),
        keyword: input.player_intent.clone(),
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

pub fn build_role_context_packet(
    store: &EntityStore,
    memory_manager: &NoNameMemoryManager,
    input: &NoNameContextBuildInput,
    role: NoNameRole,
) -> NoNameRoleContextPacket {
    let mut role_input = input.clone();
    role_input.role = role;
    let packet = build_context_packet(store, memory_manager, &role_input);
    specialize_context_packet(&packet)
}

pub fn build_role_context_packets(
    store: &EntityStore,
    memory_manager: &NoNameMemoryManager,
    input: &NoNameContextBuildInput,
    roles: &[NoNameRole],
) -> Vec<NoNameRoleContextPacket> {
    roles
        .iter()
        .copied()
        .map(|role| build_role_context_packet(store, memory_manager, input, role))
        .collect()
}

pub fn specialize_context_packet(packet: &NoNameContextPacket) -> NoNameRoleContextPacket {
    match packet.role {
        NoNameRole::Director => director_context(packet),
        NoNameRole::WorldCurator => world_curator_context(packet),
        NoNameRole::NpcIntent => npc_intent_context(packet),
        NoNameRole::CombatNarrator => combat_narrator_context(packet),
        NoNameRole::System => system_context(packet),
    }
}

fn director_context(packet: &NoNameContextPacket) -> NoNameRoleContextPacket {
    NoNameRoleContextPacket {
        role: NoNameRole::Director,
        role_goal: "Select the safest narrative focus for the next beat.".to_string(),
        scene_focus: first_of(&[
            &packet.narrative_notes,
            &packet.episodic_memory,
            &packet.recent_context,
            &packet.hard_facts,
        ]),
        world_facts: take_lines(&packet.hard_facts, 2),
        character_relationships: take_lines(&packet.referenced_entities, 4),
        narrative_priorities: take_joined(&[&packet.narrative_notes, &packet.chapter_summaries], 6),
        recent_signals: take_joined(&[&packet.working_memory, &packet.recent_context], 4),
        visible_constraints: vec![
            "May propose low-risk narrative direction.".to_string(),
            "Should keep unresolved threads visible for later turns.".to_string(),
        ],
        forbidden_scopes: vec![
            "Must not directly rewrite final plot state.".to_string(),
            "Must not invent hard world canon without WorldCurator support.".to_string(),
        ],
        source_stats: packet.source_stats.clone(),
        token_budget_used: packet.token_budget_used,
    }
}

fn world_curator_context(packet: &NoNameContextPacket) -> NoNameRoleContextPacket {
    NoNameRoleContextPacket {
        role: NoNameRole::WorldCurator,
        role_goal: "Maintain world facts, scene constraints, and canon anchors.".to_string(),
        scene_focus: first_of(&[
            &packet.hard_facts,
            &packet.chapter_summaries,
            &packet.referenced_entities,
            &packet.recent_context,
        ]),
        world_facts: take_joined(&[&packet.hard_facts, &packet.chapter_summaries], 8),
        character_relationships: take_lines(&packet.referenced_entities, 3),
        narrative_priorities: take_lines(&packet.chapter_summaries, 3),
        recent_signals: take_lines(&packet.recent_context, 2),
        visible_constraints: vec![
            "May clarify setting rules and location constraints.".to_string(),
            "Should preserve established facts over dramatic convenience.".to_string(),
        ],
        forbidden_scopes: vec![
            "Must not decide NPC private intent.".to_string(),
            "Must not choose the main plot beat alone.".to_string(),
        ],
        source_stats: packet.source_stats.clone(),
        token_budget_used: packet.token_budget_used,
    }
}

fn npc_intent_context(packet: &NoNameContextPacket) -> NoNameRoleContextPacket {
    NoNameRoleContextPacket {
        role: NoNameRole::NpcIntent,
        role_goal: "Infer NPC motivation, stance changes, and relationship pressure.".to_string(),
        scene_focus: first_of(&[
            &packet.referenced_entities,
            &packet.episodic_memory,
            &packet.narrative_notes,
            &packet.recent_context,
        ]),
        world_facts: take_lines(&packet.hard_facts, 2),
        character_relationships: take_joined(
            &[&packet.referenced_entities, &packet.narrative_notes],
            6,
        ),
        narrative_priorities: take_lines(&packet.narrative_notes, 4),
        recent_signals: take_joined(&[&packet.episodic_memory, &packet.recent_context], 5),
        visible_constraints: vec![
            "May infer motivation only from visible context.".to_string(),
            "Should keep uncertainty explicit when evidence is thin.".to_string(),
        ],
        forbidden_scopes: vec![
            "Must not reveal hidden knowledge not present in context.".to_string(),
            "Must not override world facts or combat outcomes.".to_string(),
        ],
        source_stats: packet.source_stats.clone(),
        token_budget_used: packet.token_budget_used,
    }
}

fn combat_narrator_context(packet: &NoNameContextPacket) -> NoNameRoleContextPacket {
    NoNameRoleContextPacket {
        role: NoNameRole::CombatNarrator,
        role_goal: "Track conflict rhythm, action feedback, and combat narration anchors."
            .to_string(),
        scene_focus: first_of(&[
            &packet.recent_context,
            &packet.episodic_memory,
            &packet.working_memory,
            &packet.narrative_notes,
        ]),
        world_facts: take_lines(&packet.hard_facts, 3),
        character_relationships: take_lines(&packet.referenced_entities, 3),
        narrative_priorities: take_lines(&packet.narrative_notes, 3),
        recent_signals: take_joined(&[&packet.recent_context, &packet.episodic_memory], 6),
        visible_constraints: vec![
            "May describe action consequences as low-risk narration anchors.".to_string(),
            "Should preserve current combat constraints.".to_string(),
        ],
        forbidden_scopes: vec![
            "Must not determine final damage or victory state.".to_string(),
            "Must not invent new combat rules.".to_string(),
        ],
        source_stats: packet.source_stats.clone(),
        token_budget_used: packet.token_budget_used,
    }
}

fn system_context(packet: &NoNameContextPacket) -> NoNameRoleContextPacket {
    NoNameRoleContextPacket {
        role: NoNameRole::System,
        role_goal: "Provide diagnostics without narrative authority.".to_string(),
        scene_focus: first_of(&[&packet.recent_context, &packet.working_memory]),
        world_facts: Vec::new(),
        character_relationships: Vec::new(),
        narrative_priorities: Vec::new(),
        recent_signals: take_joined(&[&packet.working_memory, &packet.recent_context], 4),
        visible_constraints: vec!["May inspect available context shape.".to_string()],
        forbidden_scopes: vec!["Must not author narrative content.".to_string()],
        source_stats: packet.source_stats.clone(),
        token_budget_used: packet.token_budget_used,
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

fn first_of(groups: &[&Vec<String>]) -> String {
    groups
        .iter()
        .find_map(|items| items.first().cloned())
        .unwrap_or_else(|| "No focused context available.".to_string())
}

fn take_lines(items: &[String], limit: usize) -> Vec<String> {
    items.iter().take(limit).cloned().collect()
}

fn take_joined(groups: &[&Vec<String>], limit: usize) -> Vec<String> {
    let mut values = Vec::new();
    for group in groups {
        for item in group.iter() {
            if values.len() >= limit {
                return values;
            }
            if !values.iter().any(|existing| existing == item) {
                values.push(item.clone());
            }
        }
    }
    values
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

    #[test]
    fn role_context_builder_returns_distinct_packets_for_core_roles() {
        let store = EntityStore::new();
        let mut manager = NoNameMemoryManager::new();
        manager.push_working_memory(
            NoNameWorkingMemoryItem {
                memory_id: "work-1".to_string(),
                turn_id: "turn-1".to_string(),
                source: "runtime".to_string(),
                category: "recent_turn".to_string(),
                summary: "Player asks Elder Qinghe about the broken ward.".to_string(),
                expires_at: None,
                priority: 10,
            },
            8,
        );
        manager.push_episodic_memory(crate::noname_memory_types::NoNameEpisodicMemoryItem {
            memory_id: "event-1".to_string(),
            event_type: "dialogue".to_string(),
            timestamp: 1,
            chapter_index: 1,
            location_id: Some("mountain_gate".to_string()),
            actors: vec!["Player".to_string(), "Elder Qinghe".to_string()],
            summary: "Elder Qinghe hesitates before naming the ward saboteur.".to_string(),
            detail_ref: None,
            importance: crate::noname_memory_types::NoNameMemoryImportance::High,
        });
        manager.upsert_semantic_memory(crate::noname_memory_types::NoNameSemanticMemoryItem {
            fact_id: "fact-1".to_string(),
            subject: "Mountain Gate".to_string(),
            predicate: "is protected by".to_string(),
            object: "ward formation".to_string(),
            confidence: 95,
            source: "test".to_string(),
            updated_at: 1,
            tags: vec!["mountain_gate".to_string()],
        });
        manager.upsert_narrative_memory(crate::noname_memory_types::NoNameNarrativeMemoryItem {
            note_id: "note-1".to_string(),
            chapter_index: 1,
            arc_id: None,
            note_type: crate::noname_memory_types::NoNameNarrativeNoteType::UnresolvedThread,
            title: "Broken Ward".to_string(),
            summary: "The saboteur behind the ward damage remains unknown.".to_string(),
            status: crate::noname_memory_types::NoNameNarrativeStatus::Active,
            related_entities: vec!["Elder Qinghe".to_string()],
            updated_at: 1,
        });

        let input = NoNameContextBuildInput {
            role: NoNameRole::Director,
            world_id: "w1".to_string(),
            run_id: "r1".to_string(),
            scene_id: "s1".to_string(),
            character_ids: vec!["player".to_string()],
            map_node_id: Some("mountain_gate".to_string()),
            player_intent: Some("ward".to_string()),
            recent_context_lines: vec![
                "The ward light flickers.".to_string(),
                "Elder Qinghe avoids the player's gaze.".to_string(),
            ],
            token_budget: 240,
            per_section_limit: 4,
        };

        let packets = build_role_context_packets(
            &store,
            &manager,
            &input,
            &[
                NoNameRole::Director,
                NoNameRole::WorldCurator,
                NoNameRole::NpcIntent,
            ],
        );

        assert_eq!(packets.len(), 3);
        assert_eq!(packets[0].role, NoNameRole::Director);
        assert_eq!(packets[1].role, NoNameRole::WorldCurator);
        assert_eq!(packets[2].role, NoNameRole::NpcIntent);
        assert_ne!(packets[0].role_goal, packets[1].role_goal);
        assert_ne!(packets[1].forbidden_scopes, packets[2].forbidden_scopes);
        assert!(packets[1]
            .world_facts
            .iter()
            .any(|item| item.contains("Mountain Gate")));
        assert!(packets[2]
            .recent_signals
            .iter()
            .any(|item| item.contains("Elder Qinghe")));
    }

    #[test]
    fn role_context_specialization_limits_world_curator_npc_visibility_differently() {
        let packet = NoNameContextPacket {
            role: NoNameRole::WorldCurator,
            hard_facts: vec!["Gate has a ward".to_string()],
            working_memory: vec!["Player suspects Elder Qinghe".to_string()],
            episodic_memory: vec!["Elder Qinghe hesitated".to_string()],
            narrative_notes: vec!["Broken Ward: saboteur unknown".to_string()],
            chapter_summaries: vec!["Gate Crisis: ward damaged".to_string()],
            recent_context: vec!["Ward flickers".to_string()],
            referenced_entities: vec!["Character:ElderQinghe".to_string()],
            compressed_summary: None,
            token_budget_used: 42,
            source_stats: vec![NoNameContextSourceStat {
                source: "test".to_string(),
                count: 1,
            }],
        };

        let world = specialize_context_packet(&packet);
        let mut npc_packet = packet.clone();
        npc_packet.role = NoNameRole::NpcIntent;
        let npc = specialize_context_packet(&npc_packet);

        assert_eq!(world.scene_focus, "Gate has a ward");
        assert_eq!(npc.scene_focus, "Character:ElderQinghe");
        assert!(world
            .forbidden_scopes
            .iter()
            .any(|item| item.contains("NPC")));
        assert!(npc
            .forbidden_scopes
            .iter()
            .any(|item| item.contains("hidden knowledge")));
    }
}
