use crate::entity_store::{EntityQuery, EntityStore};
use crate::entity_types::EntityType;
use crate::memory_layers::MemoryLayers;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextBuildInput {
    pub world_id: String,
    pub run_id: String,
    pub scene_id: String,
    pub character_ids: Vec<String>,
    pub map_node_id: Option<String>,
    pub player_intent: Option<String>,
    #[serde(default)]
    pub recent_context_lines: Vec<String>,
    pub token_budget: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextBundle {
    pub hard_facts: Vec<String>,
    pub recent_context: Vec<String>,
    pub chapter_summaries: Vec<String>,
    pub recent_events: Vec<String>,
    pub referenced_entities: Vec<String>,
    pub token_budget_used: usize,
}

pub fn build_context_bundle(
    store: &EntityStore,
    memory: &MemoryLayers,
    input: &ContextBuildInput,
) -> ContextBundle {
    let mut hard_facts = memory
        .world_facts
        .iter()
        .map(|f| format!("{} {} {}", f.subject, f.predicate, f.object))
        .collect::<Vec<_>>();

    let mut recent_events = memory
        .recent_events
        .iter()
        .rev()
        .take(12)
        .map(|e| e.summary.clone())
        .collect::<Vec<_>>();

    let mut chapter_summaries = memory
        .chapter_summaries
        .iter()
        .rev()
        .take(8)
        .map(|c| format!("{}: {}", c.title, c.summary))
        .collect::<Vec<_>>();
    let mut recent_context = input
        .recent_context_lines
        .iter()
        .rev()
        .take(10)
        .cloned()
        .collect::<Vec<_>>();

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
        for e in store.list_by_query(&query).into_iter().take(5) {
            referenced_entities.push(format!("{:?}:{}", e.entity_type, e.entity_id));
            hard_facts.push(e.payload.to_string());
        }
    }

    let mut used = 0usize;
    trim_to_budget(&mut hard_facts, input.token_budget, &mut used);
    trim_to_budget(
        &mut recent_context,
        input.token_budget.saturating_sub(used),
        &mut used,
    );
    trim_to_budget(
        &mut chapter_summaries,
        input.token_budget.saturating_sub(used),
        &mut used,
    );
    trim_to_budget(
        &mut recent_events,
        input.token_budget.saturating_sub(used),
        &mut used,
    );

    ContextBundle {
        hard_facts,
        recent_context,
        chapter_summaries,
        recent_events,
        referenced_entities,
        token_budget_used: used,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity_store::EntityStore;
    use crate::memory_layers::MemoryLayers;

    #[test]
    fn context_builder_keeps_recent_context_lines() {
        let store = EntityStore::new();
        let memory = MemoryLayers::new();
        let input = ContextBuildInput {
            world_id: "w1".to_string(),
            run_id: "r1".to_string(),
            scene_id: "s1".to_string(),
            character_ids: vec![],
            map_node_id: None,
            player_intent: None,
            recent_context_lines: vec![
                "你望向山门，风雪渐起。".to_string(),
                "远处钟声三响，师兄神色凝重。".to_string(),
            ],
            token_budget: 200,
        };

        let bundle = build_context_bundle(&store, &memory, &input);
        assert!(!bundle.recent_context.is_empty());
    }
}
