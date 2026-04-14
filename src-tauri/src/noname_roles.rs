use crate::noname_capability_registry::NoNameCapabilityRegistry;
use crate::noname_context_types::NoNameContextPacket;
use crate::noname_errors::NoNameError;
use crate::noname_prompts::DIRECTOR_OBSERVE_PROMPT_ID;
use crate::noname_protocol_tool::{NoNamePromptResolve, NoNameResourceRead, NoNameToolCall};
use crate::noname_protocol_types::{NoNameProtocolHeader, NoNameTraceWritable};
use crate::noname_tools::{DIRECTOR_CONTEXT_RESOURCE_ID, GENERATE_PLOT_CANDIDATE_TOOL_ID};
use crate::noname_trace::NoNameTrace;
use crate::noname_types::{
    NoNameApplyScope, NoNameProposal, NoNameProposalKind, NoNameProposalStatus, NoNameRole,
    NoNameTargetSegment, NoNameTraceStage,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameDirectorObservation {
    pub role: NoNameRole,
    pub action_summary: String,
    pub focus: String,
    pub rationale: String,
    pub prompt_preview: String,
    pub proposal: NoNameProposal,
}

#[derive(Debug, Default, Clone)]
pub struct DirectorAgent;

impl DirectorAgent {
    pub fn new() -> Self {
        Self
    }

    pub fn observe_turn(
        &self,
        trace: &mut NoNameTrace,
        registry: &NoNameCapabilityRegistry,
        context_packet: &NoNameContextPacket,
        action_summary: &str,
    ) -> Result<NoNameDirectorObservation, NoNameError> {
        let header = NoNameProtocolHeader::new(trace.trace_id.clone(), trace.session_id.clone());
        let resource_read = NoNameResourceRead {
            header: header.clone(),
            resource_id: DIRECTOR_CONTEXT_RESOURCE_ID.to_string(),
        };
        let resource_result = registry.read_resource(&resource_read)?;
        resource_read.record_on_trace(trace, "ok");

        let mut variables = BTreeMap::new();
        variables.insert("goal".to_string(), self.goal_from_context(context_packet));
        variables.insert("action".to_string(), action_summary.to_string());
        variables.insert(
            "scene".to_string(),
            context_packet
                .referenced_entities
                .first()
                .cloned()
                .unwrap_or_else(|| "当前场景".to_string()),
        );
        let prompt_resolve = NoNamePromptResolve {
            header: header.clone(),
            prompt_id: DIRECTOR_OBSERVE_PROMPT_ID.to_string(),
            variables,
        };
        let prompt_result = registry.resolve_prompt(&prompt_resolve)?;
        prompt_resolve.record_on_trace(trace, "ok");

        let tool_call = NoNameToolCall {
            header,
            capability_id: GENERATE_PLOT_CANDIDATE_TOOL_ID.to_string(),
            args: json!({
                "actionSummary": action_summary,
                "contextTokenBudgetUsed": context_packet.token_budget_used,
            }),
        };
        let tool_result = registry.invoke_tool(&tool_call)?;
        tool_call.record_on_trace(trace, "ok");

        let focus = self.pick_focus(context_packet);
        let rationale = format!(
            "基于{}条叙事线索、{}条近期事件与工具模式{}，下一步建议优先观察“{}”",
            context_packet.narrative_notes.len(),
            context_packet.episodic_memory.len(),
            tool_result.content["mode"]
                .as_str()
                .unwrap_or("observe_only"),
            focus
        );
        let proposal = NoNameProposal {
            proposal_id: format!("proposal-{}-director", trace.turn_id),
            kind: NoNameProposalKind::PlotCandidate,
            producer_role: NoNameRole::Director,
            title: format!("Director提案：{}", focus),
            summary: format!("围绕“{}”继续推进当前剧情观察与编排。", focus),
            focus: focus.clone(),
            target_segment: NoNameTargetSegment::CurrentTurnTail,
            intended_effect: "为下一轮低风险输出提供稳定导向".to_string(),
            rationale: rationale.clone(),
            suggested_action: Some("保持 observe-only，等待后续 assisted 落地".to_string()),
            labels: vec!["director".to_string(), "observe_only".to_string()],
            apply_scopes: vec![
                NoNameApplyScope::Diagnostics,
                NoNameApplyScope::ChapterSummaryHint,
                NoNameApplyScope::OptionBiasHint,
                NoNameApplyScope::PlotTextHint,
            ],
            status: NoNameProposalStatus::Observed,
            applyable: false,
        };
        trace.push_stage(NoNameTraceStage::AssembleProposal);
        trace.record_proposal(proposal.clone());
        let _ = resource_result;

        Ok(NoNameDirectorObservation {
            role: NoNameRole::Director,
            action_summary: action_summary.to_string(),
            focus,
            rationale,
            prompt_preview: prompt_result.resolved_prompt,
            proposal,
        })
    }

    fn goal_from_context(&self, context_packet: &NoNameContextPacket) -> String {
        context_packet
            .narrative_notes
            .first()
            .cloned()
            .or_else(|| context_packet.chapter_summaries.first().cloned())
            .unwrap_or_else(|| "保持剧情推进与状态一致".to_string())
    }

    fn pick_focus(&self, context_packet: &NoNameContextPacket) -> String {
        context_packet
            .narrative_notes
            .first()
            .cloned()
            .or_else(|| context_packet.episodic_memory.first().cloned())
            .or_else(|| context_packet.working_memory.first().cloned())
            .or_else(|| context_packet.hard_facts.first().cloned())
            .unwrap_or_else(|| "当前回合缺少足够线索，建议维持稳定推进".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noname_context_types::{NoNameContextPacket, NoNameContextSourceStat};
    use crate::noname_tools::build_director_registry;
    use crate::noname_types::NoNameMode;

    #[test]
    fn director_agent_generates_observation_and_records_trace() {
        let packet = NoNameContextPacket {
            role: NoNameRole::Director,
            hard_facts: vec!["玩家 位于 山门".to_string()],
            working_memory: vec!["玩家刚刚选择回到山门".to_string()],
            episodic_memory: vec!["玩家返回山门".to_string()],
            narrative_notes: vec!["山门危机: 强敌逼近".to_string()],
            chapter_summaries: vec!["第一章: 危机渐近".to_string()],
            recent_context: vec![],
            referenced_entities: vec!["Character:player".to_string()],
            compressed_summary: None,
            token_budget_used: 42,
            source_stats: vec![NoNameContextSourceStat {
                source: "narrative".to_string(),
                count: 1,
            }],
        };
        let registry = build_director_registry(&packet);
        let mut trace =
            NoNameTrace::empty("trace-1", "session-1", "turn-1", NoNameMode::ObserveOnly);

        let observation = DirectorAgent::new()
            .observe_turn(&mut trace, &registry, &packet, "选择返回山门")
            .expect("director observe should succeed");

        assert!(observation.rationale.contains("山门危机"));
        assert_eq!(observation.proposal.kind, NoNameProposalKind::PlotCandidate);
        assert_eq!(trace.capability_calls.len(), 3);
        assert_eq!(trace.proposals.len(), 1);
    }
}
