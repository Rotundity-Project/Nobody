use crate::noname_capability_base::{
    NoNameCapabilityDescriptor, NoNameCapabilityKind, NoNamePromptCapability,
    NoNameResourceCapability, NoNameToolCapability,
};
use crate::noname_capability_registry::NoNameCapabilityRegistry;
use crate::noname_context_types::NoNameContextPacket;
use crate::noname_prompts::{director_observe_prompt_template, DIRECTOR_OBSERVE_PROMPT_ID};
use crate::noname_resources::NoNameResourceDocument;
use serde_json::json;

pub const DIRECTOR_CONTEXT_RESOURCE_ID: &str = "resource.director.context_packet";
pub const GENERATE_PLOT_CANDIDATE_TOOL_ID: &str = "tool.generate_plot_candidate";

pub fn build_director_registry(context_packet: &NoNameContextPacket) -> NoNameCapabilityRegistry {
    let mut registry = NoNameCapabilityRegistry::new();
    registry.register_tool(NoNameToolCapability {
        descriptor: NoNameCapabilityDescriptor::new(
            GENERATE_PLOT_CANDIDATE_TOOL_ID,
            "Generate Plot Candidate",
            NoNameCapabilityKind::Tool,
            "Generate a plot candidate in observe-only mode",
        ),
        canned_result: json!({
            "planner": "director",
            "mode": "observe_only"
        }),
    });
    registry.register_resource(
        NoNameResourceCapability {
            descriptor: NoNameCapabilityDescriptor::new(
                DIRECTOR_CONTEXT_RESOURCE_ID,
                "Director Context Packet",
                NoNameCapabilityKind::Resource,
                "Read the current context packet used by DirectorAgent",
            ),
            resource_id: DIRECTOR_CONTEXT_RESOURCE_ID.to_string(),
        },
        NoNameResourceDocument::new(
            DIRECTOR_CONTEXT_RESOURCE_ID,
            "application/json",
            serde_json::to_value(context_packet).unwrap_or_else(|_| json!({})),
        ),
    );
    registry.register_prompt(
        NoNamePromptCapability {
            descriptor: NoNameCapabilityDescriptor::new(
                DIRECTOR_OBSERVE_PROMPT_ID,
                "Director Observe Prompt",
                NoNameCapabilityKind::Prompt,
                "Prompt template for DirectorAgent observe-only planning",
            ),
            prompt_id: DIRECTOR_OBSERVE_PROMPT_ID.to_string(),
        },
        director_observe_prompt_template(),
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
}
