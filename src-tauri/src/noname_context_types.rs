use crate::noname_types::NoNameRole;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameContextSourceStat {
    pub source: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameContextPacket {
    pub role: NoNameRole,
    pub hard_facts: Vec<String>,
    pub working_memory: Vec<String>,
    pub episodic_memory: Vec<String>,
    pub narrative_notes: Vec<String>,
    pub chapter_summaries: Vec<String>,
    pub recent_context: Vec<String>,
    pub referenced_entities: Vec<String>,
    pub compressed_summary: Option<String>,
    pub token_budget_used: usize,
    pub source_stats: Vec<NoNameContextSourceStat>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameContextBuildInput {
    pub role: NoNameRole,
    pub world_id: String,
    pub run_id: String,
    pub scene_id: String,
    pub character_ids: Vec<String>,
    pub map_node_id: Option<String>,
    pub player_intent: Option<String>,
    pub recent_context_lines: Vec<String>,
    pub token_budget: usize,
    pub per_section_limit: usize,
}
