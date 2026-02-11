use crate::llm_service::{LLMRequest, LLMService};
use crate::memory_manager::MemoryManager;
use crate::npc::{InteractionRecord, MemoryEntry, NPC, Relationship};
use crate::prompt_builder::{PromptBuilder, PromptConstraints, PromptContext, PromptTemplate};
use crate::response_validator::{ResponseValidator, ValidationConstraints};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NPCEvent {
    pub timestamp: u64,
    pub description: String,
    pub involved_npc_ids: Vec<String>,
    pub importance: f32,
    pub emotional_impact: f32,
    pub affinity_impact: i32,
    pub trust_impact: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NPCDecision {
    pub npc_id: String,
    pub action: String,
    pub reason: String,
}

pub struct NPCEngine {
    npcs: HashMap<String, NPC>,
    memory_manager: MemoryManager,
    llm_service: Option<LLMService>,
    prompt_builder: PromptBuilder,
    response_validator: ResponseValidator,
}

impl NPCEngine {
    pub fn new() -> Self {
        Self {
            npcs: HashMap::new(),
            memory_manager: MemoryManager::default(),
            llm_service: None,
            prompt_builder: PromptBuilder::default(),
            response_validator: ResponseValidator::default(),
        }
    }

    pub fn with_npcs(npcs: HashMap<String, NPC>) -> Self {
        Self {
            npcs,
            memory_manager: MemoryManager::default(),
            llm_service: None,
            prompt_builder: PromptBuilder::default(),
            response_validator: ResponseValidator::default(),
        }
    }

    pub fn with_llm_service(mut self, llm_service: LLMService) -> Self {
        self.llm_service = Some(llm_service);
        self
    }

    pub fn process_event(&mut self, event: &NPCEvent) -> Vec<NPCDecision> {
        let mut decisions = Vec::new();

        let affected_npc_ids = if event.involved_npc_ids.is_empty() {
            self.npcs.keys().cloned().collect::<Vec<String>>()
        } else {
            event.involved_npc_ids.clone()
        };

        for npc_id in affected_npc_ids {
            if !self.npcs.contains_key(&npc_id) {
                continue;
            }

            self.update_npc_memory(&npc_id, event);
            decisions.push(self.generate_reaction_decision(&npc_id, event));
        }

        self.apply_pairwise_relationship_updates(event);

        decisions
    }

    pub fn update_npc_memory(&mut self, npc_id: &str, event: &NPCEvent) {
        let Some(npc) = self.npcs.get_mut(npc_id) else {
            return;
        };

        let entry = MemoryEntry {
            timestamp: event.timestamp,
            event: event.description.clone(),
            importance: event.importance.clamp(0.0, 1.0),
            emotional_impact: event.emotional_impact.clamp(-1.0, 1.0),
        };

        self.memory_manager.add_memory(&mut npc.memory, entry);
    }

    pub fn update_relationship(
        &mut self,
        npc_id: &str,
        target_id: &str,
        affinity_delta: i32,
        trust_delta: i32,
        event_desc: &str,
        timestamp: u64,
    ) {
        let Some(npc) = self.npcs.get_mut(npc_id) else {
            return;
        };

        let relationship = npc
            .relationships
            .entry(target_id.to_string())
            .or_insert_with(|| Relationship {
                target_id: target_id.to_string(),
                affinity: 0,
                trust: 0,
                history: Vec::new(),
            });

        relationship.affinity = clamp_i32(relationship.affinity + affinity_delta, -100, 100);
        relationship.trust = clamp_i32(relationship.trust + trust_delta, -100, 100);
        relationship.history.push(InteractionRecord {
            timestamp,
            event: event_desc.to_string(),
            affinity_change: affinity_delta,
            trust_change: trust_delta,
        });
    }

    pub fn get_npc(&self, npc_id: &str) -> Option<&NPC> {
        self.npcs.get(npc_id)
    }

    pub fn insert_npc(&mut self, npc: NPC) {
        self.npcs.insert(npc.id.clone(), npc);
    }

    pub async fn autonomous_npc_actions(&self) -> Vec<NPCDecision> {
        let mut decisions = Vec::new();
        for npc_id in self.npcs.keys() {
            let fallback = self.generate_fallback_decision(npc_id, "no player input");
            let decision = if self.llm_service.is_some() {
                self.generate_npc_decision(npc_id, "no player input")
                    .await
                    .unwrap_or(fallback)
            } else {
                fallback
            };
            decisions.push(decision);
        }
        decisions
    }

    pub async fn generate_npc_decision(
        &self,
        npc_id: &str,
        situation: &str,
    ) -> Result<NPCDecision, String> {
        let npc = self
            .npcs
            .get(npc_id)
            .ok_or_else(|| format!("NPC not found: {npc_id}"))?;

        let decision = if let Some(llm_service) = &self.llm_service {
            self.generate_npc_decision_with_llm(llm_service, npc, situation)
                .await
                .unwrap_or_else(|_| self.generate_fallback_decision(npc_id, situation))
        } else {
            self.generate_fallback_decision(npc_id, situation)
        };

        self.validate_decision_against_personality(npc, decision)
    }

    async fn generate_npc_decision_with_llm(
        &self,
        llm_service: &LLMService,
        npc: &NPC,
        situation: &str,
    ) -> Result<NPCDecision, String> {
        let context = PromptContext {
            scene: Some(situation.to_string()),
            location: None,
            actor_name: Some(npc.name.clone()),
            actor_realm: Some(npc.stats.cultivation_realm.name.clone()),
            actor_combat_power: Some(npc.stats.combat_power),
            history_events: npc
                .memory
                .short_term
                .iter()
                .rev()
                .take(5)
                .map(|m| m.event.clone())
                .collect(),
            world_setting_summary: Some("Cultivation world with strict numerical rules".to_string()),
        };
        let constraints = PromptConstraints {
            numerical_rules: vec![
                "action must not violate combat power realism".to_string(),
                "decision should be executable in current realm".to_string(),
            ],
            world_rules: vec![
                "respond in strict JSON only".to_string(),
                "json keys: action, reason".to_string(),
            ],
            output_schema_hint: Some("{\"action\":\"string\",\"reason\":\"string\"}".to_string()),
        };

        let prompt = self.prompt_builder.build_prompt_with_token_limit(
            PromptTemplate::NpcDecision,
            &context,
            &constraints,
            400,
        );
        let response = llm_service
            .generate(LLMRequest {
                prompt,
                max_tokens: Some(200),
                temperature: Some(0.6),
            })
            .await
            .map_err(|e| e.to_string())?;

        self.response_validator
            .validate_response(
                &response,
                &ValidationConstraints {
                    require_json: true,
                    max_realm_level: None,
                    min_combat_power: None,
                    max_combat_power: None,
                    max_current_age: None,
                },
            )
            .map_err(|e| e.to_string())?;

        let parsed: Value = serde_json::from_str(&response.text).map_err(|e| e.to_string())?;
        let action = parsed
            .get("action")
            .and_then(Value::as_str)
            .ok_or_else(|| "LLM decision missing action".to_string())?;
        let reason = parsed
            .get("reason")
            .and_then(Value::as_str)
            .ok_or_else(|| "LLM decision missing reason".to_string())?;

        Ok(NPCDecision {
            npc_id: npc.id.clone(),
            action: action.to_string(),
            reason: reason.to_string(),
        })
    }

    fn generate_fallback_decision(&self, npc_id: &str, situation: &str) -> NPCDecision {
        let lower = situation.to_lowercase();
        let action = if lower.contains("combat") || lower.contains("battle") {
            "prepare_defense"
        } else if lower.contains("resource") || lower.contains("treasure") {
            "secure_resource"
        } else {
            "observe_and_plan"
        };

        NPCDecision {
            npc_id: npc_id.to_string(),
            action: action.to_string(),
            reason: format!("Fallback decision for situation: {}", situation),
        }
    }

    fn validate_decision_against_personality(
        &self,
        npc: &NPC,
        mut decision: NPCDecision,
    ) -> Result<NPCDecision, String> {
        let is_cautious = npc
            .personality
            .traits
            .iter()
            .any(|t| matches!(t, crate::npc::PersonalityTrait::Cautious));
        let is_aggressive = npc
            .personality
            .traits
            .iter()
            .any(|t| matches!(t, crate::npc::PersonalityTrait::Aggressive));

        if is_cautious && decision.action.contains("reckless") {
            decision.action = "observe_and_plan".to_string();
            decision.reason.push_str(" | adjusted for cautious personality");
        }

        if is_aggressive && decision.action == "observe_and_plan" {
            decision.action = "intervene".to_string();
            decision.reason.push_str(" | adjusted for aggressive personality");
        }

        Ok(decision)
    }

    fn generate_reaction_decision(&self, npc_id: &str, event: &NPCEvent) -> NPCDecision {
        let Some(npc) = self.npcs.get(npc_id) else {
            return NPCDecision {
                npc_id: npc_id.to_string(),
                action: "ignore".to_string(),
                reason: "npc not found".to_string(),
            };
        };

        let has_aggressive = npc
            .personality
            .traits
            .iter()
            .any(|t| matches!(t, crate::npc::PersonalityTrait::Aggressive));
        let has_cautious = npc
            .personality
            .traits
            .iter()
            .any(|t| matches!(t, crate::npc::PersonalityTrait::Cautious));

        let action = if event.importance >= 0.8 {
            if has_aggressive {
                "intervene"
            } else if has_cautious {
                "observe_carefully"
            } else {
                "respond"
            }
        } else if has_cautious {
            "observe"
        } else {
            "acknowledge"
        };

        NPCDecision {
            npc_id: npc_id.to_string(),
            action: action.to_string(),
            reason: format!("Reaction to event: {}", event.description),
        }
    }

    fn apply_pairwise_relationship_updates(&mut self, event: &NPCEvent) {
        let involved = &event.involved_npc_ids;
        for i in 0..involved.len() {
            for j in 0..involved.len() {
                if i == j {
                    continue;
                }
                self.update_relationship(
                    &involved[i],
                    &involved[j],
                    event.affinity_impact,
                    event.trust_impact,
                    &event.description,
                    event.timestamp,
                );
            }
        }
    }
}

impl Default for NPCEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn clamp_i32(value: i32, min: i32, max: i32) -> i32 {
    value.max(min).min(max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CultivationRealm, Element, Grade, Lifespan, SpiritualRoot};
    use crate::npc::{CoreValue, Goal, NPCMemory, Personality, PersonalityTrait};

    fn test_npc(id: &str, aggressive: bool) -> NPC {
        let traits = if aggressive {
            vec![PersonalityTrait::Aggressive]
        } else {
            vec![PersonalityTrait::Calm, PersonalityTrait::Cautious]
        };

        NPC {
            id: id.to_string(),
            name: format!("NPC {}", id),
            stats: crate::models::CharacterStats::new(
                SpiritualRoot {
                    element: Element::Fire,
                    grade: Grade::Double,
                    affinity: 0.7,
                },
                CultivationRealm::new("Qi Condensation".to_string(), 1, 0, 1.0),
                Lifespan::new(20, 120, 20),
            ),
            personality: Personality {
                traits,
                goals: vec![Goal {
                    description: "survive".to_string(),
                    priority: 7,
                }],
                values: vec![CoreValue {
                    name: "self-preservation".to_string(),
                    weight: 0.9,
                }],
            },
            memory: NPCMemory::default(),
            relationships: HashMap::new(),
        }
    }

    #[test]
    fn test_process_event_generates_decisions() {
        let mut npcs = HashMap::new();
        npcs.insert("a".to_string(), test_npc("a", true));
        npcs.insert("b".to_string(), test_npc("b", false));

        let mut engine = NPCEngine::with_npcs(npcs);
        let event = NPCEvent {
            timestamp: 1,
            description: "A demonic beast appears".to_string(),
            involved_npc_ids: vec!["a".to_string(), "b".to_string()],
            importance: 0.9,
            emotional_impact: 0.7,
            affinity_impact: 5,
            trust_impact: 3,
        };

        let decisions = engine.process_event(&event);
        assert_eq!(decisions.len(), 2);
        assert!(decisions.iter().any(|d| d.npc_id == "a"));
        assert!(decisions.iter().any(|d| d.npc_id == "b"));
    }

    #[test]
    fn test_update_relationship_clamps_values() {
        let mut engine = NPCEngine::new();
        engine.insert_npc(test_npc("a", false));

        engine.update_relationship("a", "b", 200, -200, "conflict", 1);
        let npc = engine.get_npc("a").unwrap();
        let rel = npc.relationships.get("b").unwrap();

        assert_eq!(rel.affinity, 100);
        assert_eq!(rel.trust, -100);
        assert_eq!(rel.history.len(), 1);
    }

    #[tokio::test]
    async fn test_generate_npc_decision_fallback() {
        let mut engine = NPCEngine::new();
        engine.insert_npc(test_npc("a", false));

        let decision = engine
            .generate_npc_decision("a", "combat in the outer sect")
            .await
            .unwrap();

        assert_eq!(decision.npc_id, "a");
        assert!(!decision.action.is_empty());
        assert!(!decision.reason.is_empty());
    }

    #[tokio::test]
    async fn test_generate_npc_decision_respects_personality() {
        let mut engine = NPCEngine::new();
        let mut npc = test_npc("a", false);
        npc.personality.traits = vec![PersonalityTrait::Cautious];
        engine.insert_npc(npc);

        let decision = engine
            .generate_npc_decision("a", "reckless battle opportunity")
            .await
            .unwrap();

        assert!(decision.action.contains("observe") || decision.action.contains("prepare"));
    }

    #[tokio::test]
    async fn test_autonomous_actions_without_player_input() {
        let mut engine = NPCEngine::new();
        engine.insert_npc(test_npc("a", false));
        engine.insert_npc(test_npc("b", true));

        let actions = engine.autonomous_npc_actions().await;
        assert_eq!(actions.len(), 2);
        assert!(actions.iter().all(|a| !a.action.is_empty()));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::models::{CultivationRealm, Element, Grade, Lifespan, SpiritualRoot};
    use crate::npc::{CoreValue, Goal, NPCMemory, Personality, PersonalityTrait};
    use proptest::prelude::*;

    fn basic_npc(id: &str) -> NPC {
        NPC {
            id: id.to_string(),
            name: id.to_string(),
            stats: crate::models::CharacterStats::new(
                SpiritualRoot {
                    element: Element::Water,
                    grade: Grade::Double,
                    affinity: 0.6,
                },
                CultivationRealm::new("Qi Condensation".to_string(), 1, 0, 1.0),
                Lifespan::new(20, 100, 10),
            ),
            personality: Personality {
                traits: vec![PersonalityTrait::Calm],
                goals: vec![Goal {
                    description: "cultivate".to_string(),
                    priority: 5,
                }],
                values: vec![CoreValue {
                    name: "discipline".to_string(),
                    weight: 0.8,
                }],
            },
            memory: NPCMemory::default(),
            relationships: HashMap::new(),
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        // Feature: Nobody, Property 13: NPC event response generation
        #[test]
        fn prop_process_event_generates_decisions_for_involved_npcs(
            importance in 0.0f32..=1.0f32,
            impact in -1.0f32..=1.0f32
        ) {
            let mut npcs = HashMap::new();
            npcs.insert("n1".to_string(), basic_npc("n1"));
            npcs.insert("n2".to_string(), basic_npc("n2"));
            let mut engine = NPCEngine::with_npcs(npcs);

            let event = NPCEvent {
                timestamp: 1,
                description: "event".to_string(),
                involved_npc_ids: vec!["n1".to_string(), "n2".to_string()],
                importance,
                emotional_impact: impact,
                affinity_impact: 1,
                trust_impact: 1,
            };

            let decisions = engine.process_event(&event);
            prop_assert_eq!(decisions.len(), 2);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        // Feature: Nobody, Property 14: interaction updates relationships
        #[test]
        fn prop_update_relationship_updates_affinity_and_trust(
            affinity_delta in -20i32..=20,
            trust_delta in -20i32..=20
        ) {
            let mut engine = NPCEngine::new();
            engine.insert_npc(basic_npc("n1"));

            engine.update_relationship(
                "n1",
                "n2",
                affinity_delta,
                trust_delta,
                "interaction",
                1,
            );

            let npc = engine.get_npc("n1").unwrap();
            let rel = npc.relationships.get("n2").unwrap();
            prop_assert_eq!(rel.affinity, affinity_delta.max(-100).min(100));
            prop_assert_eq!(rel.trust, trust_delta.max(-100).min(100));
            prop_assert_eq!(rel.history.len(), 1);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        // Feature: Nobody, Property 15: NPC autonomous action capability
        #[test]
        fn prop_npcs_can_act_without_player_input(
            npc_count in 1usize..=6
        ) {
            let mut npcs = HashMap::new();
            for i in 0..npc_count {
                let id = format!("n{}", i);
                npcs.insert(id.clone(), basic_npc(&id));
            }

            let rt = tokio::runtime::Runtime::new().unwrap();
            let engine = NPCEngine::with_npcs(npcs);
            let actions = rt.block_on(engine.autonomous_npc_actions());

            prop_assert_eq!(actions.len(), npc_count);
            prop_assert!(actions.iter().all(|a| !a.action.is_empty()));
        }
    }
}
