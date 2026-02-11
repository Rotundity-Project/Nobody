use crate::models::CharacterStats;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NPC {
    pub id: String,
    pub name: String,
    pub stats: CharacterStats,
    pub personality: Personality,
    pub memory: NPCMemory,
    pub relationships: HashMap<String, Relationship>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Personality {
    pub traits: Vec<PersonalityTrait>,
    pub goals: Vec<Goal>,
    pub values: Vec<CoreValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PersonalityTrait {
    Calm,
    Aggressive,
    Cautious,
    Ambitious,
    Righteous,
    Scheming,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Goal {
    pub description: String,
    pub priority: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoreValue {
    pub name: String,
    pub weight: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NPCMemory {
    pub short_term: Vec<MemoryEntry>,
    pub long_term: Vec<MemoryEntry>,
    pub important_events: Vec<MemoryEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub timestamp: u64,
    pub event: String,
    pub importance: f32,
    pub emotional_impact: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Relationship {
    pub target_id: String,
    pub affinity: i32,
    pub trust: i32,
    pub history: Vec<InteractionRecord>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InteractionRecord {
    pub timestamp: u64,
    pub event: String,
    pub affinity_change: i32,
    pub trust_change: i32,
}

impl NPCMemory {
    pub fn new() -> Self {
        Self {
            short_term: Vec::new(),
            long_term: Vec::new(),
            important_events: Vec::new(),
        }
    }
}

impl Default for NPCMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CultivationRealm, Element, Grade, Lifespan, SpiritualRoot};

    fn test_stats() -> CharacterStats {
        CharacterStats::new(
            SpiritualRoot {
                element: Element::Fire,
                grade: Grade::Double,
                affinity: 0.7,
            },
            CultivationRealm::new("Qi Condensation".to_string(), 1, 1, 1.2),
            Lifespan::new(20, 120, 20),
        )
    }

    #[test]
    fn test_npc_memory_default() {
        let memory = NPCMemory::default();
        assert!(memory.short_term.is_empty());
        assert!(memory.long_term.is_empty());
        assert!(memory.important_events.is_empty());
    }

    #[test]
    fn test_npc_serialization_roundtrip() {
        let npc = NPC {
            id: "npc_1".to_string(),
            name: "Han Yue".to_string(),
            stats: test_stats(),
            personality: Personality {
                traits: vec![PersonalityTrait::Calm, PersonalityTrait::Ambitious],
                goals: vec![Goal {
                    description: "Reach Foundation Establishment".to_string(),
                    priority: 8,
                }],
                values: vec![CoreValue {
                    name: "Sect Loyalty".to_string(),
                    weight: 0.9,
                }],
            },
            memory: NPCMemory::default(),
            relationships: HashMap::new(),
        };

        let json = serde_json::to_string(&npc).unwrap();
        let restored: NPC = serde_json::from_str(&json).unwrap();
        assert_eq!(npc, restored);
    }
}
