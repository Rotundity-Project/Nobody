use crate::noname_context_builder::flatten_role_context_packet;
use crate::noname_context_types::{NoNameContextPacket, NoNameRoleContextPacket};
use crate::noname_errors::NoNameError;
use crate::noname_roles::{
    unsupported_role_error, CombatNarratorAgent, DirectorAgent, NoNameRoleObservation,
    NpcIntentAgent, WorldCuratorAgent,
};
use crate::noname_tools::{
    build_combat_narrator_registry, build_director_registry, build_npc_intent_registry,
    build_world_curator_registry,
};
use crate::noname_trace::NoNameTrace;
use crate::noname_types::{NoNameProposalKind, NoNameRole};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoNameRoleProfile {
    pub role: NoNameRole,
    pub responsibility: &'static str,
    pub primary_inputs: &'static [&'static str],
    pub output_kind: NoNameProposalKind,
    pub boundary_with_director: &'static str,
}

#[derive(Debug, Clone)]
pub struct NoNameAgentRegistry {
    profiles: Vec<NoNameRoleProfile>,
}

impl Default for NoNameAgentRegistry {
    fn default() -> Self {
        Self::new_default()
    }
}

impl NoNameAgentRegistry {
    pub fn new_default() -> Self {
        Self {
            profiles: vec![
                NoNameRoleProfile {
                    role: NoNameRole::Director,
                    responsibility: "统筹当前回合的剧情观察焦点与低风险推进方向",
                    primary_inputs: &["narrative_notes", "episodic_memory", "chapter_summaries"],
                    output_kind: NoNameProposalKind::PlotCandidate,
                    boundary_with_director:
                        "Director 负责统筹，不负责直接补全世界事实或 NPC/战斗细节。",
                },
                NoNameRoleProfile {
                    role: NoNameRole::WorldCurator,
                    responsibility: "补全世界事实、场景约束和设定锚点，维持世界一致性",
                    primary_inputs: &["hard_facts", "chapter_summaries", "referenced_entities"],
                    output_kind: NoNameProposalKind::WorldPatchProposal,
                    boundary_with_director:
                        "WorldCurator 不负责决定剧情主冲突，只负责提供世界层约束与补丁候选。",
                },
                NoNameRoleProfile {
                    role: NoNameRole::NpcIntent,
                    responsibility: "推断 NPC 动机、立场变化和关系反应",
                    primary_inputs: &["referenced_entities", "recent_context", "episodic_memory"],
                    output_kind: NoNameProposalKind::NpcIntentProposal,
                    boundary_with_director:
                        "NpcIntent 不负责编排整段剧情，只补足角色行为动机与反应。",
                },
                NoNameRoleProfile {
                    role: NoNameRole::CombatNarrator,
                    responsibility: "观察冲突节奏、动作反馈与战斗描写锚点",
                    primary_inputs: &["recent_context", "episodic_memory", "action_summary"],
                    output_kind: NoNameProposalKind::CombatNarration,
                    boundary_with_director:
                        "CombatNarrator 不负责世界规则或人物动机，只服务于冲突表现层。",
                },
            ],
        }
    }

    pub fn profiles(&self) -> &[NoNameRoleProfile] {
        &self.profiles
    }

    pub fn supported_roles(&self) -> Vec<NoNameRole> {
        self.profiles.iter().map(|profile| profile.role).collect()
    }

    pub fn get_profile(&self, role: NoNameRole) -> Option<&NoNameRoleProfile> {
        self.profiles.iter().find(|profile| profile.role == role)
    }

    pub fn dispatch_observe_turn(
        &self,
        role: NoNameRole,
        trace: &mut NoNameTrace,
        context_packet: &NoNameContextPacket,
        action_summary: &str,
    ) -> Result<NoNameRoleObservation, NoNameError> {
        match role {
            NoNameRole::Director => DirectorAgent::new().observe_turn(
                trace,
                &build_director_registry(context_packet),
                context_packet,
                action_summary,
            ),
            NoNameRole::WorldCurator => WorldCuratorAgent::new().observe_turn(
                trace,
                &build_world_curator_registry(context_packet),
                context_packet,
                action_summary,
            ),
            NoNameRole::NpcIntent => NpcIntentAgent::new().observe_turn(
                trace,
                &build_npc_intent_registry(context_packet),
                context_packet,
                action_summary,
            ),
            NoNameRole::CombatNarrator => CombatNarratorAgent::new().observe_turn(
                trace,
                &build_combat_narrator_registry(context_packet),
                context_packet,
                action_summary,
            ),
            NoNameRole::System => Err(unsupported_role_error(role)),
        }
    }

    pub fn dispatch_role_context_observe_turn(
        &self,
        trace: &mut NoNameTrace,
        context_packet: &NoNameRoleContextPacket,
        action_summary: &str,
    ) -> Result<NoNameRoleObservation, NoNameError> {
        let flattened_context = flatten_role_context_packet(context_packet);
        let mut observation = self.dispatch_observe_turn(
            context_packet.role,
            trace,
            &flattened_context,
            action_summary,
        )?;
        observation.role_goal = Some(context_packet.role_goal.clone());
        observation.scene_focus = Some(context_packet.scene_focus.clone());
        observation.forbidden_scopes = context_packet.forbidden_scopes.clone();
        observation.note_type_hits = context_packet.note_type_hits.clone();
        observation.source_stats = context_packet.source_stats.clone();
        observation.context_token_budget_used = Some(context_packet.token_budget_used);
        observation.context_slice_stats = context_packet.context_slice_stats.clone();
        Ok(observation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noname_context_builder::specialize_context_packet;
    use crate::noname_context_types::{NoNameContextPacket, NoNameContextSourceStat};
    use crate::noname_types::NoNameMode;

    fn sample_packet(role: NoNameRole) -> NoNameContextPacket {
        NoNameContextPacket {
            role,
            hard_facts: vec!["山门前有护山法阵".to_string()],
            working_memory: vec!["玩家正与长老对话".to_string()],
            episodic_memory: vec!["敌修逼近山门".to_string()],
            narrative_notes: vec!["山门危机".to_string()],
            chapter_summaries: vec!["第一章: 山门受袭".to_string()],
            recent_context: vec!["敌修拔剑向前，弟子布阵迎敌".to_string()],
            referenced_entities: vec![
                "Character:player".to_string(),
                "Character:elder_qinghe".to_string(),
                "Location:qingyun_gate".to_string(),
            ],
            compressed_summary: None,
            token_budget_used: 24,
            source_stats: vec![NoNameContextSourceStat {
                source: "narrative".to_string(),
                count: 1,
            }],
        }
    }

    #[test]
    fn registry_exposes_supported_multi_role_profiles() {
        let registry = NoNameAgentRegistry::new_default();

        assert_eq!(registry.profiles().len(), 4);
        assert!(registry.get_profile(NoNameRole::WorldCurator).is_some());
        assert_eq!(
            registry
                .get_profile(NoNameRole::NpcIntent)
                .map(|profile| profile.output_kind),
            Some(NoNameProposalKind::NpcIntentProposal)
        );
    }

    #[test]
    fn registry_dispatches_multiple_roles() {
        let registry = NoNameAgentRegistry::new_default();
        let mut trace =
            NoNameTrace::empty("trace-1", "session-1", "turn-1", NoNameMode::ObserveOnly);

        let observations = [
            registry
                .dispatch_observe_turn(
                    NoNameRole::Director,
                    &mut trace,
                    &sample_packet(NoNameRole::Director),
                    "回到山门",
                )
                .expect("director dispatch should work"),
            registry
                .dispatch_observe_turn(
                    NoNameRole::WorldCurator,
                    &mut trace,
                    &sample_packet(NoNameRole::WorldCurator),
                    "检查护山法阵",
                )
                .expect("world curator dispatch should work"),
            registry
                .dispatch_observe_turn(
                    NoNameRole::NpcIntent,
                    &mut trace,
                    &sample_packet(NoNameRole::NpcIntent),
                    "询问长老是否迎战",
                )
                .expect("npc intent dispatch should work"),
        ];

        assert_eq!(
            observations[0].proposal.kind,
            NoNameProposalKind::PlotCandidate
        );
        assert_eq!(
            observations[1].proposal.kind,
            NoNameProposalKind::WorldPatchProposal
        );
        assert_eq!(
            observations[2].proposal.kind,
            NoNameProposalKind::NpcIntentProposal
        );
    }

    #[test]
    fn system_role_is_rejected() {
        let registry = NoNameAgentRegistry::new_default();
        let mut trace =
            NoNameTrace::empty("trace-2", "session-2", "turn-2", NoNameMode::ObserveOnly);
        let err = registry
            .dispatch_observe_turn(
                NoNameRole::System,
                &mut trace,
                &sample_packet(NoNameRole::System),
                "system noop",
            )
            .expect_err("system should not dispatch");

        assert_eq!(err.code, "noname.agent.unsupported_role");
    }

    #[test]
    fn registry_can_dispatch_role_context_packet_directly() {
        let registry = NoNameAgentRegistry::new_default();
        let mut trace =
            NoNameTrace::empty("trace-3", "session-3", "turn-3", NoNameMode::ObserveOnly);
        let role_context = specialize_context_packet(&sample_packet(NoNameRole::WorldCurator));

        let observation = registry
            .dispatch_role_context_observe_turn(&mut trace, &role_context, "检查护山法阵")
            .expect("role context dispatch should work");

        assert_eq!(observation.role, NoNameRole::WorldCurator);
        assert_eq!(
            observation.proposal.kind,
            NoNameProposalKind::WorldPatchProposal
        );
        assert!(observation.prompt_preview.contains(&role_context.role_goal));
        assert!(observation
            .prompt_preview
            .contains(&role_context.forbidden_scopes[0]));
        assert_eq!(
            observation.role_goal.as_deref(),
            Some(role_context.role_goal.as_str())
        );
        assert_eq!(
            observation.scene_focus.as_deref(),
            Some(role_context.scene_focus.as_str())
        );
        assert_eq!(observation.forbidden_scopes, role_context.forbidden_scopes);
        assert_eq!(observation.note_type_hits, role_context.note_type_hits);
        assert_eq!(observation.source_stats, role_context.source_stats);
        assert_eq!(
            observation.context_token_budget_used,
            Some(role_context.token_budget_used)
        );
        assert_eq!(
            observation.context_slice_stats,
            role_context.context_slice_stats
        );
    }
}
