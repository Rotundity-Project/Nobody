use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Technique,
    Character,
    MapNode,
    Item,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityCandidateRequest {
    pub entity_type: EntityType,
    pub payload: Value,
    pub source_trace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    Accepted,
    Normalized,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationReport {
    pub status: ValidationStatus,
    pub reasons: Vec<String>,
    pub normalized_payload: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedEntity {
    pub entity_id: String,
    pub entity_type: EntityType,
    pub payload: Value,
    pub validation_report: ValidationReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredEntity {
    pub world_id: String,
    pub run_id: String,
    pub entity_id: String,
    pub entity_type: EntityType,
    pub payload: Value,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TechniqueDef {
    pub technique_id: String,
    pub name: String,
    pub tags: Vec<String>,
    pub realm_requirement: u32,
    pub root_affinity: Vec<String>,
    pub base_power: f64,
    pub risk_tags: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterProfile {
    pub character_id: String,
    pub name: String,
    pub realm: String,
    pub personality_tags: Vec<String>,
    pub relationship_edges: Vec<String>,
    pub known_techniques: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapNodeDef {
    pub node_id: String,
    pub name: String,
    pub node_type: String,
    pub danger_tier: u8,
    pub aura_density: f64,
    pub faction_control: String,
    pub connected_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemDef {
    pub item_id: String,
    pub name: String,
    pub item_type: String,
    pub quality_tier: u8,
    pub description: String,
}
