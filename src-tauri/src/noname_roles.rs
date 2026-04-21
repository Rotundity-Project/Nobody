use crate::noname_capability_registry::NoNameCapabilityRegistry;
use crate::noname_context_types::{
    NoNameContextPacket, NoNameContextSourceStat, NoNameRoleContextSliceStat,
};
use crate::noname_errors::{NoNameError, NoNameErrorKind};
use crate::noname_prompts::{
    COMBAT_NARRATOR_OBSERVE_PROMPT_ID, DIRECTOR_OBSERVE_PROMPT_ID, NPC_INTENT_OBSERVE_PROMPT_ID,
    WORLD_CURATOR_OBSERVE_PROMPT_ID,
};
use crate::noname_protocol_tool::{NoNamePromptResolve, NoNameResourceRead, NoNameToolCall};
use crate::noname_protocol_types::{NoNameProtocolHeader, NoNameTraceWritable};
use crate::noname_tools::{
    COMBAT_NARRATOR_CONTEXT_RESOURCE_ID, DIRECTOR_CONTEXT_RESOURCE_ID,
    GENERATE_COMBAT_BEAT_TOOL_ID, GENERATE_NPC_INTENT_TOOL_ID, GENERATE_PLOT_CANDIDATE_TOOL_ID,
    GENERATE_WORLD_PATCH_TOOL_ID, NPC_INTENT_CONTEXT_RESOURCE_ID,
    WORLD_CURATOR_CONTEXT_RESOURCE_ID,
};
use crate::noname_trace::NoNameTrace;
use crate::noname_types::{
    NoNameApplyScope, NoNameProposal, NoNameProposalKind, NoNameProposalStatus, NoNameRole,
    NoNameTargetSegment, NoNameTraceStage,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoNameRoleObservation {
    pub role: NoNameRole,
    pub action_summary: String,
    pub focus: String,
    pub rationale: String,
    pub prompt_preview: String,
    #[serde(default)]
    pub role_goal: Option<String>,
    #[serde(default)]
    pub scene_focus: Option<String>,
    #[serde(default)]
    pub forbidden_scopes: Vec<String>,
    #[serde(default)]
    pub note_type_hits: Vec<String>,
    #[serde(default)]
    pub source_stats: Vec<NoNameContextSourceStat>,
    #[serde(default)]
    pub context_token_budget_used: Option<usize>,
    #[serde(default)]
    pub context_slice_stats: Vec<NoNameRoleContextSliceStat>,
    pub proposal: NoNameProposal,
}

pub type NoNameDirectorObservation = NoNameRoleObservation;

#[derive(Debug, Clone, PartialEq)]
struct NoNameObserveArtifacts {
    prompt_preview: String,
    tool_content: Value,
}

#[derive(Debug, Default, Clone)]
pub struct DirectorAgent;

#[derive(Debug, Default, Clone)]
pub struct WorldCuratorAgent;

#[derive(Debug, Default, Clone)]
pub struct NpcIntentAgent;

#[derive(Debug, Default, Clone)]
pub struct CombatNarratorAgent;

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
        let artifacts = run_observe_capability_pipeline(
            trace,
            registry,
            context_packet,
            action_summary,
            DIRECTOR_CONTEXT_RESOURCE_ID,
            DIRECTOR_OBSERVE_PROMPT_ID,
            GENERATE_PLOT_CANDIDATE_TOOL_ID,
            self.goal_from_context(context_packet),
        )?;

        let focus = self.pick_focus(context_packet);
        let rationale = format!(
            "基于{}条叙事线索、{}条近期事件与工具模式{}，下一步建议优先观察“{}”",
            context_packet.narrative_notes.len(),
            context_packet.episodic_memory.len(),
            read_tool_field(&artifacts.tool_content, "mode", "observe_only"),
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
            labels: vec![
                NoNameRole::Director.as_str().to_string(),
                "observe_only".to_string(),
            ],
            apply_scopes: vec![
                NoNameApplyScope::Diagnostics,
                NoNameApplyScope::ChapterSummaryHint,
                NoNameApplyScope::OptionBiasHint,
                NoNameApplyScope::PlotAugmentationHint,
                NoNameApplyScope::PlotTextHint,
            ],
            status: NoNameProposalStatus::Observed,
            applyable: false,
        };
        trace.push_stage(NoNameTraceStage::AssembleProposal);
        trace.record_proposal(proposal.clone());

        Ok(NoNameRoleObservation {
            role: NoNameRole::Director,
            action_summary: action_summary.to_string(),
            focus,
            rationale,
            prompt_preview: artifacts.prompt_preview,
            role_goal: None,
            scene_focus: None,
            forbidden_scopes: Vec::new(),
            note_type_hits: Vec::new(),
            source_stats: Vec::new(),
            context_token_budget_used: None,
            context_slice_stats: Vec::new(),
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

impl WorldCuratorAgent {
    pub fn new() -> Self {
        Self
    }

    pub fn observe_turn(
        &self,
        trace: &mut NoNameTrace,
        registry: &NoNameCapabilityRegistry,
        context_packet: &NoNameContextPacket,
        action_summary: &str,
    ) -> Result<NoNameRoleObservation, NoNameError> {
        let artifacts = run_observe_capability_pipeline(
            trace,
            registry,
            context_packet,
            action_summary,
            WORLD_CURATOR_CONTEXT_RESOURCE_ID,
            WORLD_CURATOR_OBSERVE_PROMPT_ID,
            GENERATE_WORLD_PATCH_TOOL_ID,
            self.goal_from_context(context_packet),
        )?;

        let focus = self.pick_focus(context_packet);
        let rationale = format!(
            "基于{}条硬事实、{}条章节摘要与工具目标{}，建议优先校准“{}”",
            context_packet.hard_facts.len(),
            context_packet.chapter_summaries.len(),
            read_tool_field(&artifacts.tool_content, "target", "world_state"),
            focus
        );
        let proposal = NoNameProposal {
            proposal_id: format!("proposal-{}-world-curator", trace.turn_id),
            kind: NoNameProposalKind::WorldPatchProposal,
            producer_role: NoNameRole::WorldCurator,
            title: format!("WorldCurator提案：{}", focus),
            summary: format!("围绕“{}”补足世界事实、场景约束或设定锚点。", focus),
            focus: focus.clone(),
            target_segment: NoNameTargetSegment::ChapterSummaryTail,
            intended_effect: "为后续剧情与状态一致性提供世界事实锚点".to_string(),
            rationale: rationale.clone(),
            suggested_action: Some("保持 observe-only，先产出世界补丁候选".to_string()),
            labels: vec![
                NoNameRole::WorldCurator.as_str().to_string(),
                "observe_only".to_string(),
                "world_state".to_string(),
            ],
            apply_scopes: vec![
                NoNameApplyScope::Diagnostics,
                NoNameApplyScope::ChapterSummaryHint,
            ],
            status: NoNameProposalStatus::Observed,
            applyable: false,
        };
        trace.push_stage(NoNameTraceStage::AssembleProposal);
        trace.record_proposal(proposal.clone());

        Ok(NoNameRoleObservation {
            role: NoNameRole::WorldCurator,
            action_summary: action_summary.to_string(),
            focus,
            rationale,
            prompt_preview: artifacts.prompt_preview,
            role_goal: None,
            scene_focus: None,
            forbidden_scopes: Vec::new(),
            note_type_hits: Vec::new(),
            source_stats: Vec::new(),
            context_token_budget_used: None,
            context_slice_stats: Vec::new(),
            proposal,
        })
    }

    fn goal_from_context(&self, context_packet: &NoNameContextPacket) -> String {
        context_packet
            .hard_facts
            .first()
            .cloned()
            .or_else(|| context_packet.chapter_summaries.first().cloned())
            .unwrap_or_else(|| "维持世界事实与场景约束一致".to_string())
    }

    fn pick_focus(&self, context_packet: &NoNameContextPacket) -> String {
        context_packet
            .hard_facts
            .first()
            .cloned()
            .or_else(|| context_packet.chapter_summaries.first().cloned())
            .or_else(|| context_packet.referenced_entities.first().cloned())
            .unwrap_or_else(|| "当前场景缺少稳定的世界事实锚点".to_string())
    }
}

impl NpcIntentAgent {
    pub fn new() -> Self {
        Self
    }

    pub fn observe_turn(
        &self,
        trace: &mut NoNameTrace,
        registry: &NoNameCapabilityRegistry,
        context_packet: &NoNameContextPacket,
        action_summary: &str,
    ) -> Result<NoNameRoleObservation, NoNameError> {
        let artifacts = run_observe_capability_pipeline(
            trace,
            registry,
            context_packet,
            action_summary,
            NPC_INTENT_CONTEXT_RESOURCE_ID,
            NPC_INTENT_OBSERVE_PROMPT_ID,
            GENERATE_NPC_INTENT_TOOL_ID,
            self.goal_from_context(context_packet),
        )?;

        let focus = self.pick_focus(context_packet);
        let rationale = format!(
            "基于{}个引用实体、{}条近期上下文与工具目标{}，建议优先跟踪“{}”的动机变化",
            context_packet.referenced_entities.len(),
            context_packet.recent_context.len(),
            read_tool_field(&artifacts.tool_content, "target", "npc_reaction"),
            focus
        );
        let proposal = NoNameProposal {
            proposal_id: format!("proposal-{}-npc-intent", trace.turn_id),
            kind: NoNameProposalKind::NpcIntentProposal,
            producer_role: NoNameRole::NpcIntent,
            title: format!("NpcIntent提案：{}", focus),
            summary: format!("围绕“{}”补充NPC意图、立场变化或反应预期。", focus),
            focus: focus.clone(),
            target_segment: NoNameTargetSegment::CurrentTurnTail,
            intended_effect: "为选项偏置和对话反应提供稳定的人物动机参考".to_string(),
            rationale: rationale.clone(),
            suggested_action: Some("保持 observe-only，先确认 NPC 意图走向".to_string()),
            labels: vec![
                NoNameRole::NpcIntent.as_str().to_string(),
                "observe_only".to_string(),
                "npc_reaction".to_string(),
            ],
            apply_scopes: vec![
                NoNameApplyScope::Diagnostics,
                NoNameApplyScope::OptionBiasHint,
                NoNameApplyScope::PlotAugmentationHint,
                NoNameApplyScope::PlotTextHint,
            ],
            status: NoNameProposalStatus::Observed,
            applyable: false,
        };
        trace.push_stage(NoNameTraceStage::AssembleProposal);
        trace.record_proposal(proposal.clone());

        Ok(NoNameRoleObservation {
            role: NoNameRole::NpcIntent,
            action_summary: action_summary.to_string(),
            focus,
            rationale,
            prompt_preview: artifacts.prompt_preview,
            role_goal: None,
            scene_focus: None,
            forbidden_scopes: Vec::new(),
            note_type_hits: Vec::new(),
            source_stats: Vec::new(),
            context_token_budget_used: None,
            context_slice_stats: Vec::new(),
            proposal,
        })
    }

    fn goal_from_context(&self, context_packet: &NoNameContextPacket) -> String {
        first_non_player_character(&context_packet.referenced_entities)
            .or_else(|| context_packet.narrative_notes.first().cloned())
            .unwrap_or_else(|| "补足当前场景的 NPC 动机与反应".to_string())
    }

    fn pick_focus(&self, context_packet: &NoNameContextPacket) -> String {
        first_non_player_character(&context_packet.referenced_entities)
            .or_else(|| context_packet.episodic_memory.first().cloned())
            .or_else(|| context_packet.narrative_notes.first().cloned())
            .unwrap_or_else(|| "当前场景缺少明确的 NPC 反应锚点".to_string())
    }
}

impl CombatNarratorAgent {
    pub fn new() -> Self {
        Self
    }

    pub fn observe_turn(
        &self,
        trace: &mut NoNameTrace,
        registry: &NoNameCapabilityRegistry,
        context_packet: &NoNameContextPacket,
        action_summary: &str,
    ) -> Result<NoNameRoleObservation, NoNameError> {
        let artifacts = run_observe_capability_pipeline(
            trace,
            registry,
            context_packet,
            action_summary,
            COMBAT_NARRATOR_CONTEXT_RESOURCE_ID,
            COMBAT_NARRATOR_OBSERVE_PROMPT_ID,
            GENERATE_COMBAT_BEAT_TOOL_ID,
            self.goal_from_context(context_packet, action_summary),
        )?;

        let focus = self.pick_focus(context_packet, action_summary);
        let rationale = format!(
            "基于{}条近期上下文、{}条事件记忆与工具目标{}，建议优先强化“{}”的冲突节奏",
            context_packet.recent_context.len(),
            context_packet.episodic_memory.len(),
            read_tool_field(&artifacts.tool_content, "target", "combat_pacing"),
            focus
        );
        let proposal = NoNameProposal {
            proposal_id: format!("proposal-{}-combat-narrator", trace.turn_id),
            kind: NoNameProposalKind::CombatNarration,
            producer_role: NoNameRole::CombatNarrator,
            title: format!("CombatNarrator提案：{}", focus),
            summary: format!("围绕“{}”强化冲突节奏、动作反馈或战斗描写。", focus),
            focus: focus.clone(),
            target_segment: NoNameTargetSegment::CurrentTurnTail,
            intended_effect: "为战斗或高压冲突场景提供更稳定的表现节奏".to_string(),
            rationale: rationale.clone(),
            suggested_action: Some("保持 observe-only，先输出战斗节奏候选".to_string()),
            labels: vec![
                NoNameRole::CombatNarrator.as_str().to_string(),
                "observe_only".to_string(),
                "combat".to_string(),
            ],
            apply_scopes: vec![
                NoNameApplyScope::Diagnostics,
                NoNameApplyScope::PlotAugmentationHint,
                NoNameApplyScope::PlotTextHint,
            ],
            status: NoNameProposalStatus::Observed,
            applyable: false,
        };
        trace.push_stage(NoNameTraceStage::AssembleProposal);
        trace.record_proposal(proposal.clone());

        Ok(NoNameRoleObservation {
            role: NoNameRole::CombatNarrator,
            action_summary: action_summary.to_string(),
            focus,
            rationale,
            prompt_preview: artifacts.prompt_preview,
            role_goal: None,
            scene_focus: None,
            forbidden_scopes: Vec::new(),
            note_type_hits: Vec::new(),
            source_stats: Vec::new(),
            context_token_budget_used: None,
            context_slice_stats: Vec::new(),
            proposal,
        })
    }

    fn goal_from_context(
        &self,
        context_packet: &NoNameContextPacket,
        action_summary: &str,
    ) -> String {
        self.pick_focus(context_packet, action_summary)
    }

    fn pick_focus(&self, context_packet: &NoNameContextPacket, action_summary: &str) -> String {
        context_packet
            .recent_context
            .iter()
            .find(|line| contains_combat_signal(line))
            .cloned()
            .or_else(|| {
                if contains_combat_signal(action_summary) {
                    Some(action_summary.to_string())
                } else {
                    None
                }
            })
            .or_else(|| context_packet.episodic_memory.first().cloned())
            .unwrap_or_else(|| "当前冲突仍缺少明确的节奏锚点".to_string())
    }
}

#[allow(clippy::too_many_arguments)]
fn run_observe_capability_pipeline(
    trace: &mut NoNameTrace,
    registry: &NoNameCapabilityRegistry,
    context_packet: &NoNameContextPacket,
    action_summary: &str,
    resource_id: &str,
    prompt_id: &str,
    tool_id: &str,
    goal: String,
) -> Result<NoNameObserveArtifacts, NoNameError> {
    let header = NoNameProtocolHeader::new(trace.trace_id.clone(), trace.session_id.clone());
    let resource_read = NoNameResourceRead {
        header: header.clone(),
        resource_id: resource_id.to_string(),
    };
    let _resource_result = registry.read_resource(&resource_read)?;
    resource_read.record_on_trace(trace, "ok");

    let mut variables = BTreeMap::new();
    variables.insert("goal".to_string(), goal);
    variables.insert(
        "roleGoal".to_string(),
        role_goal_from_context(context_packet),
    );
    variables.insert("action".to_string(), action_summary.to_string());
    variables.insert("scene".to_string(), scene_from_context(context_packet));
    variables.insert(
        "forbiddenScopes".to_string(),
        forbidden_scopes_from_context(context_packet),
    );

    let prompt_resolve = NoNamePromptResolve {
        header: header.clone(),
        prompt_id: prompt_id.to_string(),
        variables,
    };
    let prompt_result = registry.resolve_prompt(&prompt_resolve)?;
    prompt_resolve.record_on_trace(trace, "ok");

    let tool_call = NoNameToolCall {
        header,
        capability_id: tool_id.to_string(),
        args: json!({
            "actionSummary": action_summary,
            "contextTokenBudgetUsed": context_packet.token_budget_used,
            "roleGoal": role_goal_from_context(context_packet),
            "forbiddenScopes": forbidden_scopes_from_context(context_packet),
        }),
    };
    let tool_result = registry.invoke_tool(&tool_call)?;
    tool_call.record_on_trace(trace, "ok");

    Ok(NoNameObserveArtifacts {
        prompt_preview: prompt_result.resolved_prompt,
        tool_content: tool_result.content,
    })
}

fn scene_from_context(context_packet: &NoNameContextPacket) -> String {
    context_packet
        .referenced_entities
        .first()
        .cloned()
        .unwrap_or_else(|| "当前场景".to_string())
}

fn role_goal_from_context(context_packet: &NoNameContextPacket) -> String {
    metadata_value_from_context(context_packet, "roleGoal").unwrap_or_else(|| {
        "Follow the current role responsibility without expanding authority.".to_string()
    })
}

fn forbidden_scopes_from_context(context_packet: &NoNameContextPacket) -> String {
    metadata_value_from_context(context_packet, "forbiddenScopes")
        .unwrap_or_else(|| "Must not directly mutate final plot state.".to_string())
}

fn metadata_value_from_context(context_packet: &NoNameContextPacket, key: &str) -> Option<String> {
    let summary = context_packet.compressed_summary.as_ref()?;
    let marker = format!("{key}: ");
    let start = summary.find(&marker)? + marker.len();
    let tail = &summary[start..];
    let end = tail.find(';').unwrap_or(tail.len());
    let value = tail[..end].trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn read_tool_field<'a>(content: &'a Value, field: &str, fallback: &'a str) -> &'a str {
    content
        .get(field)
        .and_then(Value::as_str)
        .unwrap_or(fallback)
}

fn first_non_player_character(referenced_entities: &[String]) -> Option<String> {
    referenced_entities
        .iter()
        .find(|entity| entity.starts_with("Character:") && !entity.contains("player"))
        .cloned()
}

fn contains_combat_signal(text: &str) -> bool {
    ["战", "斗", "交手", "冲突", "出招", "杀", "攻"]
        .iter()
        .any(|token| text.contains(token))
}

pub fn unsupported_role_error(role: NoNameRole) -> NoNameError {
    NoNameError::new(
        NoNameErrorKind::Config,
        format!(
            "role {} is not registered for observe dispatch",
            role.as_str()
        ),
        "noname.agent.unsupported_role",
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noname_context_builder::{flatten_role_context_packet, specialize_context_packet};
    use crate::noname_context_types::{NoNameContextPacket, NoNameContextSourceStat};
    use crate::noname_tools::{
        build_combat_narrator_registry, build_director_registry, build_npc_intent_registry,
        build_world_curator_registry,
    };
    use crate::noname_types::NoNameMode;

    fn sample_packet(role: NoNameRole) -> NoNameContextPacket {
        NoNameContextPacket {
            role,
            hard_facts: vec![
                "玩家 位于 山门".to_string(),
                "山门受宗门法阵保护".to_string(),
            ],
            working_memory: vec!["玩家刚刚选择回到山门".to_string()],
            episodic_memory: vec![
                "玩家返回山门".to_string(),
                "执事长老抬手示意弟子退后".to_string(),
            ],
            narrative_notes: vec!["山门危机: 强敌逼近".to_string()],
            chapter_summaries: vec!["第一章: 危机渐近".to_string()],
            recent_context: vec!["敌修拔剑逼近，山门弟子列阵应对".to_string()],
            referenced_entities: vec![
                "Character:player".to_string(),
                "Character:elder_qinghe".to_string(),
                "Location:qingyun_gate".to_string(),
            ],
            compressed_summary: None,
            token_budget_used: 42,
            source_stats: vec![NoNameContextSourceStat {
                source: "narrative".to_string(),
                count: 1,
            }],
        }
    }

    #[test]
    fn director_agent_generates_observation_and_records_trace() {
        let packet = sample_packet(NoNameRole::Director);
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

    #[test]
    fn multi_role_agents_generate_distinct_proposal_kinds() {
        let mut trace =
            NoNameTrace::empty("trace-2", "session-2", "turn-2", NoNameMode::ObserveOnly);
        let world_packet = sample_packet(NoNameRole::WorldCurator);
        let npc_packet = sample_packet(NoNameRole::NpcIntent);
        let combat_packet = sample_packet(NoNameRole::CombatNarrator);

        let world = WorldCuratorAgent::new()
            .observe_turn(
                &mut trace,
                &build_world_curator_registry(&world_packet),
                &world_packet,
                "检查山门法阵是否稳定",
            )
            .expect("world curator observe should succeed");
        let npc = NpcIntentAgent::new()
            .observe_turn(
                &mut trace,
                &build_npc_intent_registry(&npc_packet),
                &npc_packet,
                "向青河长老请示是否迎战",
            )
            .expect("npc intent observe should succeed");
        let combat = CombatNarratorAgent::new()
            .observe_turn(
                &mut trace,
                &build_combat_narrator_registry(&combat_packet),
                &combat_packet,
                "敌修拔剑攻来，玩家侧身避让",
            )
            .expect("combat narrator observe should succeed");

        assert_eq!(world.proposal.kind, NoNameProposalKind::WorldPatchProposal);
        assert_eq!(npc.proposal.kind, NoNameProposalKind::NpcIntentProposal);
        assert_eq!(combat.proposal.kind, NoNameProposalKind::CombatNarration);
        assert_eq!(trace.proposals.len(), 3);
        assert_eq!(trace.capability_calls.len(), 9);
    }

    #[test]
    fn role_prompt_reads_role_goal_and_forbidden_scopes_from_context() {
        let mut base_packet = sample_packet(NoNameRole::WorldCurator);
        base_packet.role = NoNameRole::WorldCurator;
        let role_packet = specialize_context_packet(&base_packet);
        let flattened = flatten_role_context_packet(&role_packet);
        let registry = build_world_curator_registry(&flattened);
        let mut trace =
            NoNameTrace::empty("trace-3", "session-3", "turn-3", NoNameMode::ObserveOnly);

        let observation = WorldCuratorAgent::new()
            .observe_turn(&mut trace, &registry, &flattened, "检查山门法阵")
            .expect("world curator observe should resolve role prompt");

        assert!(observation.prompt_preview.contains("角色目标"));
        assert!(observation.prompt_preview.contains(&role_packet.role_goal));
        assert!(observation.prompt_preview.contains("禁止越权"));
        assert!(observation
            .prompt_preview
            .contains(&role_packet.forbidden_scopes[0]));
    }

    #[test]
    fn unsupported_role_error_uses_stable_code() {
        let err = unsupported_role_error(NoNameRole::System);
        assert_eq!(err.code, "noname.agent.unsupported_role");
        assert!(err.message.contains("system"));
    }
}
