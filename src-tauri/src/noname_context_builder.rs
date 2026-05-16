use crate::entity_store::{EntityQuery, EntityStore};
use crate::entity_types::EntityType;
use crate::noname_context_types::{
    NoNameContextBuildInput, NoNameContextPacket, NoNameContextSourceStat, NoNameRoleContextPacket,
    NoNameRoleContextSliceStat, NoNameRoleNoteEvidenceStat,
};
use crate::noname_memory_manager::NoNameMemoryManager;
use crate::noname_memory_retrieval::NoNameMemoryQuery;
use crate::noname_memory_types::{NoNameNarrativeMemoryItem, NoNameNarrativeNoteType};
use crate::noname_types::NoNameRole;

pub fn build_context_packet(
    store: &EntityStore,
    memory_manager: &NoNameMemoryManager,
    input: &NoNameContextBuildInput,
) -> NoNameContextPacket {
    let query = NoNameMemoryQuery {
        role: input.role,
        search_term: input.player_intent.clone(),
        actor: None,
        location: input.map_node_id.clone(),
        goal: input.player_intent.clone(),
        keyword: input.player_intent.clone(),
        token_budget: input.token_budget,
        per_section_limit: input.per_section_limit,
    };
    let retrieval_report = memory_manager.retrieve_with_active_note_context(&query);
    let note_context_count = retrieval_report.note_contexts.len();
    let retrieved = retrieval_report.memories;

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
    let active_notes = role_ranked_active_notes(memory_manager.active_notes(), input.role);
    let mut narrative_notes = dedupe_ordered_strings(
        active_notes
            .iter()
            .chain(retrieved.narrative.iter())
            .map(note_line)
            .collect(),
    );
    let mut chapter_summaries = active_notes
        .iter()
        .take(input.per_section_limit)
        .map(note_line)
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

    let mut source_stats = vec![
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
    ];
    if note_context_count > 0 {
        source_stats.push(NoNameContextSourceStat {
            source: "noteContext".to_string(),
            count: note_context_count,
        });
    }

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
        source_stats,
    }
}

fn note_line(note: &NoNameNarrativeMemoryItem) -> String {
    format!("{}: {}", note.title, note.summary)
}

fn role_ranked_active_notes(
    mut notes: Vec<NoNameNarrativeMemoryItem>,
    role: NoNameRole,
) -> Vec<NoNameNarrativeMemoryItem> {
    notes.sort_by(|left, right| {
        role_note_priority(role, left.note_type)
            .cmp(&role_note_priority(role, right.note_type))
            .then_with(|| right.updated_at.cmp(&left.updated_at))
            .then_with(|| left.note_id.cmp(&right.note_id))
    });
    notes
}

fn role_note_priority(role: NoNameRole, note_type: NoNameNarrativeNoteType) -> u8 {
    match role {
        NoNameRole::Director => match note_type {
            NoNameNarrativeNoteType::Conflict => 0,
            NoNameNarrativeNoteType::Goal => 1,
            NoNameNarrativeNoteType::UnresolvedThread => 2,
            NoNameNarrativeNoteType::Foreshadowing => 3,
            NoNameNarrativeNoteType::CharacterArc => 4,
        },
        NoNameRole::WorldCurator => match note_type {
            NoNameNarrativeNoteType::Goal => 0,
            NoNameNarrativeNoteType::Foreshadowing => 1,
            NoNameNarrativeNoteType::UnresolvedThread => 2,
            NoNameNarrativeNoteType::Conflict => 3,
            NoNameNarrativeNoteType::CharacterArc => 4,
        },
        NoNameRole::NpcIntent => match note_type {
            NoNameNarrativeNoteType::CharacterArc => 0,
            NoNameNarrativeNoteType::Conflict => 1,
            NoNameNarrativeNoteType::UnresolvedThread => 2,
            NoNameNarrativeNoteType::Foreshadowing => 3,
            NoNameNarrativeNoteType::Goal => 4,
        },
        NoNameRole::CombatNarrator => match note_type {
            NoNameNarrativeNoteType::Conflict => 0,
            NoNameNarrativeNoteType::CharacterArc => 1,
            NoNameNarrativeNoteType::Goal => 2,
            NoNameNarrativeNoteType::UnresolvedThread => 3,
            NoNameNarrativeNoteType::Foreshadowing => 4,
        },
        NoNameRole::System => match note_type {
            NoNameNarrativeNoteType::Goal => 0,
            NoNameNarrativeNoteType::Conflict => 1,
            NoNameNarrativeNoteType::Foreshadowing => 2,
            NoNameNarrativeNoteType::UnresolvedThread => 3,
            NoNameNarrativeNoteType::CharacterArc => 4,
        },
    }
}

fn build_role_note_hits(notes: &[NoNameNarrativeMemoryItem], limit: usize) -> Vec<String> {
    notes
        .iter()
        .take(limit)
        .map(|note| format!("{}: {}", note_type_label(note.note_type), note.title))
        .collect()
}

fn note_type_label(note_type: NoNameNarrativeNoteType) -> &'static str {
    match note_type {
        NoNameNarrativeNoteType::Goal => "goal",
        NoNameNarrativeNoteType::Conflict => "conflict",
        NoNameNarrativeNoteType::Foreshadowing => "foreshadowing",
        NoNameNarrativeNoteType::UnresolvedThread => "unresolvedThread",
        NoNameNarrativeNoteType::CharacterArc => "characterArc",
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
    let ranked_notes = role_ranked_active_notes(memory_manager.active_notes(), role);
    let mut role_packet = specialize_context_packet(&packet);
    role_packet.note_type_hits = build_role_note_hits(&ranked_notes, input.per_section_limit);
    role_packet.note_evidence_stats = build_role_note_evidence_stats(&ranked_notes);
    role_packet
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

pub fn flatten_role_context_packet(packet: &NoNameRoleContextPacket) -> NoNameContextPacket {
    NoNameContextPacket {
        role: packet.role,
        hard_facts: packet.world_facts.clone(),
        working_memory: packet.recent_signals.clone(),
        episodic_memory: packet.recent_signals.clone(),
        narrative_notes: packet.narrative_priorities.clone(),
        chapter_summaries: packet.narrative_priorities.clone(),
        recent_context: packet.recent_signals.clone(),
        referenced_entities: packet.character_relationships.clone(),
        compressed_summary: Some(format!(
            "roleGoal: {}; sceneFocus: {}; noteTypeHits: {}; visibleConstraints: {}; forbiddenScopes: {}",
            packet.role_goal,
            packet.scene_focus,
            packet.note_type_hits.join(" | "),
            packet.visible_constraints.join(" | "),
            packet.forbidden_scopes.join(" | ")
        )),
        token_budget_used: packet.token_budget_used,
        source_stats: packet.source_stats.clone(),
    }
}

fn director_context(packet: &NoNameContextPacket) -> NoNameRoleContextPacket {
    let world_facts = take_lines(&packet.hard_facts, 2);
    let character_relationships = take_lines(&packet.referenced_entities, 4);
    let narrative_priorities =
        take_joined(&[&packet.narrative_notes, &packet.chapter_summaries], 6);
    let recent_signals = take_joined(&[&packet.working_memory, &packet.recent_context], 4);
    NoNameRoleContextPacket {
        role: NoNameRole::Director,
        role_goal: "Select the safest narrative focus for the next beat.".to_string(),
        scene_focus: first_of(&[
            &packet.narrative_notes,
            &packet.episodic_memory,
            &packet.recent_context,
            &packet.hard_facts,
        ]),
        note_type_hits: Vec::new(),
        note_evidence_stats: Vec::new(),
        world_facts: world_facts.clone(),
        character_relationships: character_relationships.clone(),
        narrative_priorities: narrative_priorities.clone(),
        recent_signals: recent_signals.clone(),
        visible_constraints: vec![
            "May propose low-risk narrative direction.".to_string(),
            "Should keep unresolved threads visible for later turns.".to_string(),
            "Primary context bias: narrative notes, episodic memory, then recent signals."
                .to_string(),
        ],
        forbidden_scopes: vec![
            "Must not directly rewrite final plot state.".to_string(),
            "Must not invent hard world canon without WorldCurator support.".to_string(),
        ],
        context_slice_stats: vec![
            build_slice_stat("worldFacts", packet.hard_facts.len(), world_facts.len()),
            build_slice_stat(
                "characterRelationships",
                packet.referenced_entities.len(),
                character_relationships.len(),
            ),
            build_slice_stat(
                "narrativePriorities",
                packet.narrative_notes.len() + packet.chapter_summaries.len(),
                narrative_priorities.len(),
            ),
            build_slice_stat(
                "recentSignals",
                packet.working_memory.len() + packet.recent_context.len(),
                recent_signals.len(),
            ),
        ],
        source_stats: role_source_stats(
            packet,
            &[
                ("rolePriority:director:narrative", 3),
                ("rolePriority:director:episodic", 2),
            ],
        ),
        token_budget_used: packet.token_budget_used,
    }
}

fn world_curator_context(packet: &NoNameContextPacket) -> NoNameRoleContextPacket {
    let world_facts = take_joined(&[&packet.hard_facts, &packet.chapter_summaries], 8);
    let character_relationships = take_lines(&packet.referenced_entities, 3);
    let narrative_priorities = take_lines(&packet.chapter_summaries, 3);
    let recent_signals = take_lines(&packet.recent_context, 2);
    NoNameRoleContextPacket {
        role: NoNameRole::WorldCurator,
        role_goal: "Maintain world facts, scene constraints, and canon anchors.".to_string(),
        scene_focus: first_of(&[
            &packet.hard_facts,
            &packet.chapter_summaries,
            &packet.referenced_entities,
            &packet.recent_context,
        ]),
        note_type_hits: Vec::new(),
        note_evidence_stats: Vec::new(),
        world_facts: world_facts.clone(),
        character_relationships: character_relationships.clone(),
        narrative_priorities: narrative_priorities.clone(),
        recent_signals: recent_signals.clone(),
        visible_constraints: vec![
            "May clarify setting rules and location constraints.".to_string(),
            "Should preserve established facts over dramatic convenience.".to_string(),
            "Primary context bias: semantic facts, chapter summaries, then map entities."
                .to_string(),
        ],
        forbidden_scopes: vec![
            "Must not decide NPC private intent.".to_string(),
            "Must not choose the main plot beat alone.".to_string(),
        ],
        context_slice_stats: vec![
            build_slice_stat(
                "worldFacts",
                packet.hard_facts.len() + packet.chapter_summaries.len(),
                world_facts.len(),
            ),
            build_slice_stat(
                "characterRelationships",
                packet.referenced_entities.len(),
                character_relationships.len(),
            ),
            build_slice_stat(
                "narrativePriorities",
                packet.chapter_summaries.len(),
                narrative_priorities.len(),
            ),
            build_slice_stat(
                "recentSignals",
                packet.recent_context.len(),
                recent_signals.len(),
            ),
        ],
        source_stats: role_source_stats(
            packet,
            &[
                ("rolePriority:worldCurator:semantic", 4),
                ("rolePriority:worldCurator:chapterSummary", 2),
            ],
        ),
        token_budget_used: packet.token_budget_used,
    }
}

fn npc_intent_context(packet: &NoNameContextPacket) -> NoNameRoleContextPacket {
    let world_facts = take_lines(&packet.hard_facts, 2);
    let character_relationships =
        take_joined(&[&packet.referenced_entities, &packet.narrative_notes], 6);
    let narrative_priorities = take_lines(&packet.narrative_notes, 4);
    let recent_signals = take_joined(&[&packet.episodic_memory, &packet.recent_context], 5);
    NoNameRoleContextPacket {
        role: NoNameRole::NpcIntent,
        role_goal: "Infer NPC motivation, stance changes, and relationship pressure.".to_string(),
        scene_focus: first_of(&[
            &packet.referenced_entities,
            &packet.episodic_memory,
            &packet.narrative_notes,
            &packet.recent_context,
        ]),
        note_type_hits: Vec::new(),
        note_evidence_stats: Vec::new(),
        world_facts: world_facts.clone(),
        character_relationships: character_relationships.clone(),
        narrative_priorities: narrative_priorities.clone(),
        recent_signals: recent_signals.clone(),
        visible_constraints: vec![
            "May infer motivation only from visible context.".to_string(),
            "Should keep uncertainty explicit when evidence is thin.".to_string(),
            "Primary context bias: referenced entities, episodic memory, then narrative notes."
                .to_string(),
        ],
        forbidden_scopes: vec![
            "Must not reveal hidden knowledge not present in context.".to_string(),
            "Must not override world facts or combat outcomes.".to_string(),
        ],
        context_slice_stats: vec![
            build_slice_stat("worldFacts", packet.hard_facts.len(), world_facts.len()),
            build_slice_stat(
                "characterRelationships",
                packet.referenced_entities.len() + packet.narrative_notes.len(),
                character_relationships.len(),
            ),
            build_slice_stat(
                "narrativePriorities",
                packet.narrative_notes.len(),
                narrative_priorities.len(),
            ),
            build_slice_stat(
                "recentSignals",
                packet.episodic_memory.len() + packet.recent_context.len(),
                recent_signals.len(),
            ),
        ],
        source_stats: role_source_stats(
            packet,
            &[
                ("rolePriority:npcIntent:referencedEntities", 4),
                ("rolePriority:npcIntent:episodic", 3),
            ],
        ),
        token_budget_used: packet.token_budget_used,
    }
}

fn combat_narrator_context(packet: &NoNameContextPacket) -> NoNameRoleContextPacket {
    let world_facts = take_lines(&packet.hard_facts, 3);
    let character_relationships = take_lines(&packet.referenced_entities, 3);
    let narrative_priorities = take_lines(&packet.narrative_notes, 3);
    let recent_signals = take_joined(&[&packet.recent_context, &packet.episodic_memory], 6);
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
        note_type_hits: Vec::new(),
        note_evidence_stats: Vec::new(),
        world_facts: world_facts.clone(),
        character_relationships: character_relationships.clone(),
        narrative_priorities: narrative_priorities.clone(),
        recent_signals: recent_signals.clone(),
        visible_constraints: vec![
            "May describe action consequences as low-risk narration anchors.".to_string(),
            "Should preserve current combat constraints.".to_string(),
            "Primary context bias: recent context, episodic combat memory, then working memory."
                .to_string(),
        ],
        forbidden_scopes: vec![
            "Must not determine final damage or victory state.".to_string(),
            "Must not invent new combat rules.".to_string(),
        ],
        context_slice_stats: vec![
            build_slice_stat("worldFacts", packet.hard_facts.len(), world_facts.len()),
            build_slice_stat(
                "characterRelationships",
                packet.referenced_entities.len(),
                character_relationships.len(),
            ),
            build_slice_stat(
                "narrativePriorities",
                packet.narrative_notes.len(),
                narrative_priorities.len(),
            ),
            build_slice_stat(
                "recentSignals",
                packet.recent_context.len() + packet.episodic_memory.len(),
                recent_signals.len(),
            ),
        ],
        source_stats: role_source_stats(
            packet,
            &[
                ("rolePriority:combatNarrator:recentContext", 4),
                ("rolePriority:combatNarrator:episodic", 3),
            ],
        ),
        token_budget_used: packet.token_budget_used,
    }
}

fn system_context(packet: &NoNameContextPacket) -> NoNameRoleContextPacket {
    let recent_signals = take_joined(&[&packet.working_memory, &packet.recent_context], 4);
    NoNameRoleContextPacket {
        role: NoNameRole::System,
        role_goal: "Provide diagnostics without narrative authority.".to_string(),
        scene_focus: first_of(&[&packet.recent_context, &packet.working_memory]),
        note_type_hits: Vec::new(),
        note_evidence_stats: Vec::new(),
        world_facts: Vec::new(),
        character_relationships: Vec::new(),
        narrative_priorities: Vec::new(),
        recent_signals: recent_signals.clone(),
        visible_constraints: vec!["May inspect available context shape.".to_string()],
        forbidden_scopes: vec!["Must not author narrative content.".to_string()],
        context_slice_stats: vec![build_slice_stat(
            "recentSignals",
            packet.working_memory.len() + packet.recent_context.len(),
            recent_signals.len(),
        )],
        source_stats: role_source_stats(packet, &[("rolePriority:system:diagnostics", 1)]),
        token_budget_used: packet.token_budget_used,
    }
}

fn build_slice_stat(
    section: &str,
    source_count: usize,
    visible_count: usize,
) -> NoNameRoleContextSliceStat {
    NoNameRoleContextSliceStat {
        section: section.to_string(),
        source_count,
        visible_count,
    }
}

fn build_role_note_evidence_stats(
    ranked_notes: &[NoNameNarrativeMemoryItem],
) -> Vec<NoNameRoleNoteEvidenceStat> {
    let mut stats: Vec<NoNameRoleNoteEvidenceStat> = Vec::new();
    for note in ranked_notes {
        let note_type = note_type_label(note.note_type).to_string();
        if let Some(existing) = stats.iter_mut().find(|item| item.note_type == note_type) {
            existing.count += 1;
        } else {
            stats.push(NoNameRoleNoteEvidenceStat {
                note_type,
                count: 1,
            });
        }
    }
    stats.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.note_type.cmp(&right.note_type))
    });
    stats
}

fn role_source_stats(
    packet: &NoNameContextPacket,
    role_priorities: &[(&str, usize)],
) -> Vec<NoNameContextSourceStat> {
    let mut stats = packet.source_stats.clone();
    for (source, count) in role_priorities {
        stats.push(NoNameContextSourceStat {
            source: (*source).to_string(),
            count: *count,
        });
    }
    stats
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

fn dedupe_ordered_strings(values: Vec<String>) -> Vec<String> {
    let mut deduped = Vec::new();
    for value in values {
        if !deduped.iter().any(|existing| existing == &value) {
            deduped.push(value);
        }
    }
    deduped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_layers::{ChapterSummary, MemoryEntry, MemoryLayers, WorldFact};
    use crate::noname_memory_manager::NoNameMemoryManager;
    use crate::noname_memory_types::{
        NoNameEpisodicMemoryItem, NoNameMemoryImportance, NoNameNarrativeMemoryItem,
        NoNameNarrativeNoteType, NoNameNarrativeStatus, NoNameWorkingMemoryItem,
    };
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
    fn context_builder_uses_active_notes_to_expand_retrieved_memory() {
        let store = EntityStore::new();
        let mut manager = NoNameMemoryManager::new();
        manager.push_episodic_memory(NoNameEpisodicMemoryItem {
            memory_id: "event-qinghe".to_string(),
            event_type: "dialogue".to_string(),
            timestamp: 4,
            chapter_index: 1,
            location_id: None,
            actors: vec!["Elder Qinghe".to_string()],
            summary: "Elder Qinghe hesitated before naming the saboteur.".to_string(),
            detail_ref: None,
            importance: NoNameMemoryImportance::High,
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
                player_intent: Some("ward".to_string()),
                recent_context_lines: vec![],
                token_budget: 200,
                per_section_limit: 4,
            },
        );

        assert!(packet
            .episodic_memory
            .iter()
            .any(|item| item.contains("Elder Qinghe hesitated")));
        assert!(packet
            .source_stats
            .iter()
            .any(|item| item.source == "noteContext" && item.count == 1));
    }

    #[test]
    fn context_builder_ignores_archived_notes_when_expanding_retrieval() {
        let store = EntityStore::new();
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
            importance: NoNameMemoryImportance::High,
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
                player_intent: Some("ward".to_string()),
                recent_context_lines: vec![],
                token_budget: 200,
                per_section_limit: 4,
            },
        );

        assert!(packet.episodic_memory.is_empty());
        assert!(!packet
            .source_stats
            .iter()
            .any(|item| item.source == "noteContext"));
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
        assert!(packets[0]
            .context_slice_stats
            .iter()
            .any(|item| item.section == "narrativePriorities"
                && item.source_count >= item.visible_count));
        assert!(packets[1]
            .context_slice_stats
            .iter()
            .any(|item| item.section == "worldFacts" && item.source_count >= item.visible_count));
        assert_ne!(packets[0].source_stats, packets[1].source_stats);
        assert!(packets[0]
            .source_stats
            .iter()
            .any(|item| item.source == "rolePriority:director:narrative"));
        assert!(packets[1]
            .source_stats
            .iter()
            .any(|item| item.source == "rolePriority:worldCurator:semantic"));
        assert!(packets[2]
            .source_stats
            .iter()
            .any(|item| item.source == "rolePriority:npcIntent:referencedEntities"));
        assert!(packets[2]
            .visible_constraints
            .iter()
            .any(|item| item.contains("Primary context bias")));
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
    fn role_context_prioritizes_structured_note_types_per_role() {
        let store = EntityStore::new();
        let mut manager = NoNameMemoryManager::new();
        manager.upsert_narrative_memory(test_note(
            "goal-1",
            crate::noname_memory_types::NoNameNarrativeNoteType::Goal,
            "Hold Gate",
            "Keep the mountain gate secure.",
            1,
        ));
        manager.upsert_narrative_memory(test_note(
            "conflict-1",
            crate::noname_memory_types::NoNameNarrativeNoteType::Conflict,
            "Gate Assault",
            "Enemy cultivators are pressing the ward line.",
            2,
        ));
        manager.upsert_narrative_memory(test_note(
            "arc-1",
            crate::noname_memory_types::NoNameNarrativeNoteType::CharacterArc,
            "Elder Qinghe Doubt",
            "Elder Qinghe is hesitating under pressure.",
            3,
        ));

        let input = NoNameContextBuildInput {
            role: NoNameRole::Director,
            world_id: "w1".to_string(),
            run_id: "r1".to_string(),
            scene_id: "s1".to_string(),
            character_ids: vec!["player".to_string()],
            map_node_id: None,
            player_intent: None,
            recent_context_lines: vec!["The ward line flickers.".to_string()],
            token_budget: 240,
            per_section_limit: 3,
        };

        let director = build_role_context_packet(&store, &manager, &input, NoNameRole::Director);
        let npc = build_role_context_packet(&store, &manager, &input, NoNameRole::NpcIntent);
        let world = build_role_context_packet(&store, &manager, &input, NoNameRole::WorldCurator);

        assert!(director.narrative_priorities[0].contains("Gate Assault"));
        assert!(npc.narrative_priorities[0].contains("Elder Qinghe Doubt"));
        assert!(world.narrative_priorities[0].contains("Hold Gate"));
        assert_eq!(director.note_type_hits[0], "conflict: Gate Assault");
        assert_eq!(npc.note_type_hits[0], "characterArc: Elder Qinghe Doubt");
        assert_eq!(world.note_type_hits[0], "goal: Hold Gate");
        assert!(director
            .note_evidence_stats
            .iter()
            .any(|item| item.note_type == "conflict" && item.count == 1));
        assert!(npc
            .note_evidence_stats
            .iter()
            .any(|item| item.note_type == "characterArc" && item.count == 1));
        assert!(world
            .note_evidence_stats
            .iter()
            .any(|item| item.note_type == "goal" && item.count == 1));
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

    #[test]
    fn role_context_can_flatten_back_to_agent_context_packet() {
        let packet = NoNameContextPacket {
            role: NoNameRole::NpcIntent,
            hard_facts: vec!["Gate has a ward".to_string()],
            working_memory: vec!["Player suspects Elder Qinghe".to_string()],
            episodic_memory: vec!["Elder Qinghe hesitated".to_string()],
            narrative_notes: vec!["Broken Ward: saboteur unknown".to_string()],
            chapter_summaries: vec!["Gate Crisis: ward damaged".to_string()],
            recent_context: vec!["Ward flickers".to_string()],
            referenced_entities: vec![
                "Character:player".to_string(),
                "Character:ElderQinghe".to_string(),
            ],
            compressed_summary: None,
            token_budget_used: 42,
            source_stats: vec![NoNameContextSourceStat {
                source: "test".to_string(),
                count: 1,
            }],
        };

        let role_packet = specialize_context_packet(&packet);
        let flattened = flatten_role_context_packet(&role_packet);

        assert_eq!(flattened.role, NoNameRole::NpcIntent);
        assert_eq!(flattened.hard_facts, role_packet.world_facts);
        assert_eq!(flattened.narrative_notes, role_packet.narrative_priorities);
        assert_eq!(flattened.recent_context, role_packet.recent_signals);
        assert!(flattened
            .referenced_entities
            .iter()
            .any(|item| item.contains("ElderQinghe")));
        assert!(flattened
            .compressed_summary
            .as_deref()
            .unwrap_or_default()
            .contains("roleGoal"));
    }

    #[test]
    fn role_context_source_stats_are_specialized_per_role() {
        let packet = NoNameContextPacket {
            role: NoNameRole::Director,
            hard_facts: vec!["Gate has a ward".to_string()],
            working_memory: vec!["Player suspects Elder Qinghe".to_string()],
            episodic_memory: vec!["Elder Qinghe hesitated".to_string()],
            narrative_notes: vec!["Broken Ward: saboteur unknown".to_string()],
            chapter_summaries: vec!["Gate Crisis: ward damaged".to_string()],
            recent_context: vec!["Ward flickers".to_string()],
            referenced_entities: vec!["Character:ElderQinghe".to_string()],
            compressed_summary: None,
            token_budget_used: 42,
            source_stats: vec![
                NoNameContextSourceStat {
                    source: "semantic".to_string(),
                    count: 1,
                },
                NoNameContextSourceStat {
                    source: "episodic".to_string(),
                    count: 1,
                },
            ],
        };

        let director = specialize_context_packet(&packet);
        let mut world_packet = packet.clone();
        world_packet.role = NoNameRole::WorldCurator;
        let world = specialize_context_packet(&world_packet);
        let mut npc_packet = packet.clone();
        npc_packet.role = NoNameRole::NpcIntent;
        let npc = specialize_context_packet(&npc_packet);

        assert!(director
            .source_stats
            .iter()
            .any(|item| item.source == "rolePriority:director:narrative" && item.count == 3));
        assert!(world
            .source_stats
            .iter()
            .any(|item| item.source == "rolePriority:worldCurator:semantic" && item.count == 4));
        assert!(npc.source_stats.iter().any(|item| {
            item.source == "rolePriority:npcIntent:referencedEntities" && item.count == 4
        }));
        assert_ne!(director.source_stats, world.source_stats);
        assert_ne!(world.source_stats, npc.source_stats);
    }

    fn test_note(
        note_id: &str,
        note_type: crate::noname_memory_types::NoNameNarrativeNoteType,
        title: &str,
        summary: &str,
        updated_at: u64,
    ) -> crate::noname_memory_types::NoNameNarrativeMemoryItem {
        crate::noname_memory_types::NoNameNarrativeMemoryItem {
            note_id: note_id.to_string(),
            chapter_index: 1,
            arc_id: None,
            note_type,
            title: title.to_string(),
            summary: summary.to_string(),
            status: crate::noname_memory_types::NoNameNarrativeStatus::Active,
            related_entities: vec![],
            updated_at,
        }
    }
}
