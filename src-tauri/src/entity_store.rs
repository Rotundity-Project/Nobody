use crate::entity_types::{EntityType, StoredEntity};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityQuery {
    pub world_id: String,
    pub run_id: String,
    pub entity_type: Option<EntityType>,
    pub keyword: Option<String>,
}

#[derive(Default)]
pub struct EntityStore {
    entities: HashMap<String, StoredEntity>,
}

impl EntityStore {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
        }
    }

    pub fn upsert(&mut self, mut entity: StoredEntity) {
        entity.updated_at = now_secs();
        self.entities.insert(
            key(&entity.world_id, &entity.run_id, &entity.entity_id),
            entity,
        );
    }

    pub fn get_by_id(&self, world_id: &str, run_id: &str, entity_id: &str) -> Option<StoredEntity> {
        self.entities
            .get(&key(world_id, run_id, entity_id))
            .cloned()
    }

    pub fn list_by_query(&self, query: &EntityQuery) -> Vec<StoredEntity> {
        let mut out = self
            .entities
            .values()
            .filter(|e| e.world_id == query.world_id && e.run_id == query.run_id)
            .cloned()
            .collect::<Vec<_>>();

        if let Some(entity_type) = query.entity_type {
            out.retain(|e| e.entity_type == entity_type);
        }
        if let Some(keyword) = &query.keyword {
            let lower = keyword.to_lowercase();
            out.retain(|e| e.payload.to_string().to_lowercase().contains(&lower));
        }
        out.sort_by_key(|entity| std::cmp::Reverse(entity.updated_at));
        out
    }
}

fn key(world_id: &str, run_id: &str, entity_id: &str) -> String {
    format!("{}::{}::{}", world_id, run_id, entity_id)
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn upsert_and_query_roundtrip() {
        let mut store = EntityStore::new();
        store.upsert(StoredEntity {
            world_id: "w1".to_string(),
            run_id: "r1".to_string(),
            entity_id: "e1".to_string(),
            entity_type: EntityType::Technique,
            payload: json!({"name": "Fire Burst"}),
            updated_at: 0,
        });

        let res = store.get_by_id("w1", "r1", "e1");
        assert!(res.is_some());
    }
}
