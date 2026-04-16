use crate::noname_capability_base::{
    NoNameCapabilityDescriptor, NoNameCapabilityKind, NoNamePromptCapability,
    NoNameResourceCapability, NoNameToolCapability,
};
use crate::noname_capability_registry::NoNameCapabilityRegistry;
use crate::noname_context_types::NoNameContextPacket;
use crate::noname_prompts::{
    combat_narrator_observe_prompt_template, director_observe_prompt_template,
    npc_intent_observe_prompt_template, world_curator_observe_prompt_template,
    COMBAT_NARRATOR_OBSERVE_PROMPT_ID, DIRECTOR_OBSERVE_PROMPT_ID,
    NPC_INTENT_OBSERVE_PROMPT_ID, WORLD_CURATOR_OBSERVE_PROMPT_ID,
};
use crate::noname_prompt_catalog::NoNamePromptTemplate;
use crate::noname_resources::NoNameResourceDocument;
use serde_json::json;

pub const DIRECTOR_CONTEXT_RESOURCE_ID: &str = "resource.director.context_packet";
pub const GENERATE_PLOT_CANDIDATE_TOOL_ID: &str = "tool.generate_plot_candidate";
pub const WORLD_CURATOR_CONTEXT_RESOURCE_ID: &str = "resource.world_curator.context_packet";
pub const NPC_INTENT_CONTEXT_RESOURCE_ID: &str = "resource.npc_intent.context_packet";
pub const COMBAT_NARRATOR_CONTEXT_RESOURCE_ID: &str = "resource.combat_narrator.context_packet";
pub const GENERATE_WORLD_PATCH_TOOL_ID: &str = "tool.generate_world_patch";
pub const GENERATE_NPC_INTENT_TOOL_ID: &str = "tool.generate_npc_intent";
pub const GENERATE_COMBAT_BEAT_TOOL_ID: &str = "tool.generate_combat_beat";

pub fn build_director_registry(context_packet: &NoNameContextPacket) -> NoNameCapabilityRegistry {
    build_role_registry(
        context_packet,
        DIRECTOR_CONTEXT_RESOURCE_ID,
        GENERATE_PLOT_CANDIDATE_TOOL_ID,
        "Generate Plot Candidate",
        "Generate a plot candidate in observe-only mode",
        json!({
            "planner": "director",
            "mode": "observe_only"
        }),
        DIRECTOR_OBSERVE_PROMPT_ID,
        "Director Observe Prompt",
        "Prompt template for DirectorAgent observe-only planning",
        director_observe_prompt_template(),
    )
}

pub fn build_world_curator_registry(
    context_packet: &NoNameContextPacket,
) -> NoNameCapabilityRegistry {
    build_role_registry(
        context_packet,
        WORLD_CURATOR_CONTEXT_RESOURCE_ID,
        GENERATE_WORLD_PATCH_TOOL_ID,
        "Generate World Patch",
        "Generate a world-state patch proposal in observe-only mode",
        json!({
            "planner": "world_curator",
            "mode": "observe_only",
            "target": "world_state",
        }),
        WORLD_CURATOR_OBSERVE_PROMPT_ID,
        "World Curator Observe Prompt",
        "Prompt template for WorldCuratorAgent observe-only planning",
        world_curator_observe_prompt_template(),
    )
}

pub fn build_npc_intent_registry(context_packet: &NoNameContextPacket) -> NoNameCapabilityRegistry {
    build_role_registry(
        context_packet,
        NPC_INTENT_CONTEXT_RESOURCE_ID,
        GENERATE_NPC_INTENT_TOOL_ID,
        "Generate NPC Intent",
        "Generate an NPC intent proposal in observe-only mode",
        json!({
            "planner": "npc_intent",
            "mode": "observe_only",
            "target": "npc_reaction",
        }),
        NPC_INTENT_OBSERVE_PROMPT_ID,
        "NPC Intent Observe Prompt",
        "Prompt template for NpcIntentAgent observe-only planning",
        npc_intent_observe_prompt_template(),
    )
}

pub fn build_combat_narrator_registry(
    context_packet: &NoNameContextPacket,
) -> NoNameCapabilityRegistry {
    build_role_registry(
        context_packet,
        COMBAT_NARRATOR_CONTEXT_RESOURCE_ID,
        GENERATE_COMBAT_BEAT_TOOL_ID,
        "Generate Combat Beat",
        "Generate a combat narration proposal in observe-only mode",
        json!({
            "planner": "combat_narrator",
            "mode": "observe_only",
            "target": "combat_pacing",
        }),
        COMBAT_NARRATOR_OBSERVE_PROMPT_ID,
        "Combat Narrator Observe Prompt",
        "Prompt template for CombatNarratorAgent observe-only planning",
        combat_narrator_observe_prompt_template(),
    )
}

fn build_role_registry(
    context_packet: &NoNameContextPacket,
    resource_id: &str,
    tool_id: &str,
    tool_name: &str,
    tool_description: &str,
    tool_result: serde_json::Value,
    prompt_id: &str,
    prompt_name: &str,
    prompt_description: &str,
    prompt_template: NoNamePromptTemplate,
) -> NoNameCapabilityRegistry {
    let mut registry = NoNameCapabilityRegistry::new();
    registry.register_tool(NoNameToolCapability {
        descriptor: NoNameCapabilityDescriptor::new(
            tool_id,
            tool_name,
            NoNameCapabilityKind::Tool,
            tool_description,
        ),
        canned_result: tool_result,
    });
    registry.register_resource(
        NoNameResourceCapability {
            descriptor: NoNameCapabilityDescriptor::new(
                resource_id,
                "Role Context Packet",
                NoNameCapabilityKind::Resource,
                "Read the current context packet used by a NoName role agent",
            ),
            resource_id: resource_id.to_string(),
        },
        NoNameResourceDocument::new(
            resource_id,
            "application/json",
            serde_json::to_value(context_packet).unwrap_or_else(|_| json!({})),
        ),
    );
    registry.register_prompt(
        NoNamePromptCapability {
            descriptor: NoNameCapabilityDescriptor::new(
                prompt_id,
                prompt_name,
                NoNameCapabilityKind::Prompt,
                prompt_description,
            ),
            prompt_id: prompt_id.to_string(),
        },
        prompt_template,
    );
    registry
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noname_context_types::NoNameContextPacket;
    use crate::noname_types::NoNameRole;

    #[test]
    fn director_registry_contains_tool_resource_and_prompt() {
        let registry = build_director_registry(&NoNameContextPacket {
            role: NoNameRole::Director,
            hard_facts: vec![],
            working_memory: vec![],
            episodic_memory: vec![],
            narrative_notes: vec![],
            chapter_summaries: vec![],
            recent_context: vec![],
            referenced_entities: vec![],
            compressed_summary: None,
            token_budget_used: 0,
            source_stats: vec![],
        });

        assert_eq!(registry.list_descriptors().len(), 3);
    }

    #[test]
    fn multi_role_registries_are_constructible() {
        let packet = NoNameContextPacket {
            role: NoNameRole::WorldCurator,
            hard_facts: vec!["山门位于青云岭".to_string()],
            working_memory: vec![],
            episodic_memory: vec![],
            narrative_notes: vec![],
            chapter_summaries: vec![],
            recent_context: vec![],
            referenced_entities: vec!["Location:qingyun_gate".to_string()],
            compressed_summary: None,
            token_budget_used: 12,
            source_stats: vec![],
        };

        let registries = [
            build_world_curator_registry(&packet),
            build_npc_intent_registry(&packet),
            build_combat_narrator_registry(&packet),
        ];

        for registry in registries {
            assert_eq!(registry.list_descriptors().len(), 3);
        }
    }
}
