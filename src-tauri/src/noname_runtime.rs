use crate::noname_config::NoNameConfig;
use crate::noname_context_types::NoNameContextPacket;
use crate::noname_errors::{NoNameError, NoNameErrorKind};
use crate::noname_graph::NoNameGraphExecutor;
use crate::noname_guardrails::{
    validate_director_observation, validate_director_proposal_for_apply,
    NoNameDirectorGuardrailInput, NoNameGuardrailResult,
};
use crate::noname_roles::{DirectorAgent, NoNameDirectorObservation};
use crate::noname_tools::build_director_registry;
use crate::noname_trace::NoNameTrace;
use crate::noname_types::{
    NoNameApplyScope, NoNameMode, NoNameProposal, NoNameProposalStatus, NoNameRole,
    NoNameTraceStage,
};
use std::collections::VecDeque;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoNameTurnInput {
    pub trace_id: String,
    pub session_id: String,
    pub turn_id: String,
    pub caller_role: NoNameRole,
}

impl Default for NoNameTurnInput {
    fn default() -> Self {
        Self {
            trace_id: "noname-trace-0".to_string(),
            session_id: "default-session".to_string(),
            turn_id: "turn-0".to_string(),
            caller_role: NoNameRole::System,
        }
    }
}

#[derive(Debug)]
pub struct NoNameRuntime {
    config: NoNameConfig,
    graph_executor: NoNameGraphExecutor,
    recent_traces: VecDeque<NoNameTrace>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoNameDirectorRunResult {
    pub trace: NoNameTrace,
    pub observation: NoNameDirectorObservation,
    pub proposal: NoNameProposal,
    pub guardrail_result: Option<NoNameGuardrailResult>,
}

impl NoNameRuntime {
    pub fn new(config: NoNameConfig) -> Self {
        Self {
            config,
            graph_executor: NoNameGraphExecutor::new(),
            recent_traces: VecDeque::new(),
        }
    }

    pub fn mode(&self) -> NoNameMode {
        self.config.mode
    }

    pub fn config_snapshot(&self) -> NoNameConfig {
        self.config.clone()
    }

    pub fn set_mode(&mut self, mode: NoNameMode) {
        self.config = NoNameConfig::from_mode(mode);
    }

    pub fn run_turn(&mut self, input: NoNameTurnInput) -> Result<NoNameTrace, NoNameError> {
        let start = Instant::now();
        let mut trace = self.execute_turn_skeleton(&input)?;
        trace.elapsed_ms = start.elapsed().as_millis() as u64;

        if self.config.trace_policy.enabled {
            self.store_trace(trace.clone())?;
        }

        Ok(trace)
    }

    pub fn run_director_observe_turn(
        &mut self,
        input: NoNameTurnInput,
        action_summary: &str,
        context_packet: &NoNameContextPacket,
        guardrail_input: Option<&NoNameDirectorGuardrailInput>,
    ) -> Result<NoNameDirectorRunResult, NoNameError> {
        self.config.validate().map_err(NoNameError::from)?;

        let start = Instant::now();
        let mut trace = self.execute_turn_skeleton(&input)?;
        let registry = build_director_registry(context_packet);
        let observation = DirectorAgent::new().observe_turn(
            &mut trace,
            &registry,
            context_packet,
            action_summary,
        )?;
        let mut observation = observation;
        let guardrail_result = guardrail_input.map(|payload| {
            trace.push_stage(NoNameTraceStage::ValidateProposal);
            let result = validate_director_observation(&observation, payload);
            trace.set_guardrail_result(result.outcome.as_str(), result.reason.clone());
            if result.is_rejected() {
                trace.fallback_used = true;
                trace.push_stage(NoNameTraceStage::Fallback);
            }
            result
        });
        let proposal = self.finalize_director_proposal(&mut observation, guardrail_result.as_ref());
        let proposal = self.record_assisted_apply_preflight(&mut trace, proposal, guardrail_input);
        observation.proposal = proposal.clone();
        trace.replace_last_proposal(proposal.clone());
        trace.elapsed_ms = start.elapsed().as_millis() as u64;

        if self.config.trace_policy.enabled {
            self.store_trace(trace.clone())?;
        }

        Ok(NoNameDirectorRunResult {
            trace,
            observation,
            proposal,
            guardrail_result,
        })
    }

    pub fn store_trace(&mut self, trace: NoNameTrace) -> Result<(), NoNameError> {
        let max_recent_traces = self.config.trace_policy.max_recent_traces;
        if max_recent_traces == 0 {
            return Err(NoNameError::new(
                NoNameErrorKind::Trace,
                "trace retention size must be greater than zero",
                "noname.trace.invalid_retention",
                true,
            ));
        }

        self.recent_traces.push_back(trace);
        while self.recent_traces.len() > max_recent_traces {
            self.recent_traces.pop_front();
        }

        Ok(())
    }

    pub fn get_recent_traces(&self) -> Vec<NoNameTrace> {
        self.recent_traces.iter().cloned().collect()
    }

    pub fn replace_trace(&mut self, trace: NoNameTrace) -> bool {
        if let Some(existing) = self
            .recent_traces
            .iter_mut()
            .rev()
            .find(|item| item.trace_id == trace.trace_id)
        {
            *existing = trace;
            return true;
        }
        false
    }

    pub fn clear_traces(&mut self) {
        self.recent_traces.clear();
    }

    fn execute_turn_skeleton(&self, input: &NoNameTurnInput) -> Result<NoNameTrace, NoNameError> {
        self.config.validate().map_err(NoNameError::from)?;
        let mut trace = NoNameTrace::empty(
            input.trace_id.clone(),
            input.session_id.clone(),
            input.turn_id.clone(),
            self.config.mode,
        );
        self.graph_executor.execute_empty_turn(&mut trace)?;
        Ok(trace)
    }

    fn finalize_director_proposal(
        &self,
        observation: &mut NoNameDirectorObservation,
        guardrail_result: Option<&NoNameGuardrailResult>,
    ) -> NoNameProposal {
        let mut proposal = observation.proposal.clone();
        proposal.status = NoNameProposalStatus::Observed;
        proposal.applyable = proposal.status.is_applyable();

        if self.config.mode.allows_apply() {
            match guardrail_result {
                Some(result) if result.is_rejected() => {
                    proposal.status = NoNameProposalStatus::Blocked;
                    proposal.labels.push("assisted_blocked".to_string());
                    proposal.suggested_action =
                        Some("护栏拒绝，保持 observe-only 回退".to_string());
                }
                Some(result) => {
                    proposal.status = NoNameProposalStatus::Ready;
                    proposal.labels.push("assisted_ready".to_string());
                    proposal
                        .labels
                        .push(format!("guardrail_{}", result.outcome.as_str()));
                    proposal.suggested_action = Some("可进入 assisted 辅助应用分支".to_string());
                }
                None => {
                    proposal.status = NoNameProposalStatus::Ready;
                    proposal.labels.push("assisted_ready".to_string());
                    proposal.suggested_action =
                        Some("未配置护栏输入，但允许进入 assisted 预备分支".to_string());
                }
            }
        } else {
            proposal.status = NoNameProposalStatus::Observed;
            proposal.labels.push("observe_only".to_string());
            proposal.suggested_action = Some("保持 observe-only，不直接应用".to_string());
        }

        proposal.applyable = proposal.status.is_applyable();

        observation.proposal = proposal.clone();
        proposal
    }

    fn record_assisted_apply_preflight(
        &self,
        trace: &mut NoNameTrace,
        mut proposal: NoNameProposal,
        guardrail_input: Option<&NoNameDirectorGuardrailInput>,
    ) -> NoNameProposal {
        trace.record_proposal_transition(format!(
            "{}:{}",
            proposal.proposal_id,
            proposal.status.as_str()
        ));

        match self.config.mode {
            NoNameMode::Disabled => {
                trace.record_apply_execution(
                    "runtime.apply",
                    "skipped_disabled",
                    Some("当前模式为 disabled，未进入 apply 预检".to_string()),
                );
                trace.set_apply_result(
                    false,
                    "skipped_disabled",
                    Some("当前模式为 disabled，未进入 apply 预检".to_string()),
                );
            }
            NoNameMode::ObserveOnly => {
                trace.record_apply_execution(
                    "runtime.apply",
                    "skipped_observe_only",
                    Some("当前模式仅观察，不尝试 assisted apply".to_string()),
                );
                trace.set_apply_result(
                    false,
                    "skipped_observe_only",
                    Some("当前模式仅观察，不尝试 assisted apply".to_string()),
                );
            }
            NoNameMode::Assisted => {
                trace.push_stage(NoNameTraceStage::ApplyProposal);
                let apply_guardrail = validate_director_proposal_for_apply(
                    self.config.mode,
                    &proposal,
                    guardrail_input,
                );
                trace.record_proposal_transition(format!(
                    "{}:apply_preflight:{}",
                    proposal.proposal_id,
                    apply_guardrail.outcome.as_str()
                ));
                match apply_guardrail.outcome {
                    outcome if outcome.allows_apply() => {
                        proposal.status = NoNameProposalStatus::Applied;
                        proposal.applyable = true;
                        if !proposal
                            .labels
                            .iter()
                            .any(|item| item == "apply_preflight_ready")
                        {
                            proposal.labels.push("apply_preflight_ready".to_string());
                        }
                        if proposal.apply_scopes.is_empty()
                            || proposal.apply_scopes.contains(&NoNameApplyScope::Diagnostics)
                        {
                            if !proposal
                                .labels
                                .iter()
                                .any(|item| item == "apply_scope_diagnostics")
                            {
                                proposal.labels.push("apply_scope_diagnostics".to_string());
                            }
                            trace.record_proposal_transition(format!(
                                "{}:applied:diagnostics",
                                proposal.proposal_id
                            ));
                        } else {
                            trace.record_proposal_transition(format!(
                                "{}:preflight_ready:no_diagnostics_scope",
                                proposal.proposal_id
                            ));
                        }
                        trace.record_apply_execution(
                            "runtime.apply_preflight",
                            "ready",
                            Some("已通过 apply guardrail，允许进入低风险输出层".to_string()),
                        );
                        trace.set_apply_result(
                            true,
                            "applied_diagnostics_note",
                            Some(format!(
                                "已将提案应用到诊断层，聚焦“{}”，不改写主剧情结果",
                                proposal.focus
                            )),
                        );
                    }
                    crate::noname_guardrails::NoNameApplyGuardrailOutcome::ProposalBlocked => {
                        trace.push_stage(NoNameTraceStage::ApplyFallback);
                        proposal.status = NoNameProposalStatus::Blocked;
                        proposal.applyable = false;
                        if !proposal
                            .labels
                            .iter()
                            .any(|item| item == "apply_preflight_blocked")
                        {
                            proposal.labels.push("apply_preflight_blocked".to_string());
                        }
                        trace.record_proposal_transition(format!(
                            "{}:blocked",
                            proposal.proposal_id
                        ));
                        trace.record_apply_execution(
                            "runtime.apply_preflight",
                            "blocked",
                            apply_guardrail.reason.clone(),
                        );
                        trace.set_apply_result(
                            true,
                            "preflight_blocked",
                            apply_guardrail.reason.clone().or_else(|| {
                                proposal.suggested_action.clone().or_else(|| {
                                    Some("提案未通过 apply 预检，已回退经典链路".to_string())
                                })
                            }),
                        );
                    }
                    _ => {
                        trace.push_stage(NoNameTraceStage::ApplyFallback);
                        proposal.status = NoNameProposalStatus::Fallback;
                        proposal.applyable = false;
                        if !proposal.labels.iter().any(|item| item == "apply_fallback") {
                            proposal.labels.push("apply_fallback".to_string());
                        }
                        trace.record_proposal_transition(format!(
                            "{}:fallback",
                            proposal.proposal_id
                        ));
                        trace.record_apply_execution(
                            "runtime.apply_preflight",
                            "fallback",
                            apply_guardrail.reason.clone(),
                        );
                        proposal.suggested_action = apply_guardrail
                            .reason
                            .clone()
                            .or_else(|| Some("apply 预检未通过，继续走经典链路".to_string()));
                        trace.set_apply_result(
                            true,
                            format!("preflight_{}", apply_guardrail.outcome.as_str()),
                            apply_guardrail.reason.clone().or_else(|| {
                                Some("提案进入 fallback 路径，未修改主剧情结果".to_string())
                            }),
                        );
                    }
                }
            }
        }

        proposal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noname_types::NoNameTraceStage;

    #[test]
    fn observe_only_runtime_runs_empty_turn_and_stores_trace() {
        let mut runtime = NoNameRuntime::new(NoNameConfig::observe_only());

        let trace = runtime
            .run_turn(NoNameTurnInput {
                trace_id: "trace-1".to_string(),
                session_id: "session-1".to_string(),
                turn_id: "turn-1".to_string(),
                caller_role: NoNameRole::Director,
            })
            .expect("runtime turn should succeed");

        assert_eq!(trace.mode, NoNameMode::ObserveOnly);
        assert_eq!(
            trace.graph_path,
            vec![
                NoNameTraceStage::CollectTurnInput,
                NoNameTraceStage::BuildContextBundle,
                NoNameTraceStage::PlanTurn,
                NoNameTraceStage::PersistTrace,
            ]
        );
        assert_eq!(runtime.get_recent_traces().len(), 1);
    }

    #[test]
    fn disabled_runtime_returns_trace_without_storing_it() {
        let mut runtime = NoNameRuntime::new(NoNameConfig::disabled());

        let trace = runtime
            .run_turn(NoNameTurnInput::default())
            .expect("disabled runtime should still run skeleton");

        assert_eq!(trace.mode, NoNameMode::Disabled);
        assert!(runtime.get_recent_traces().is_empty());
    }

    #[test]
    fn clear_traces_empties_runtime_storage() {
        let mut runtime = NoNameRuntime::new(NoNameConfig::observe_only());
        runtime
            .run_turn(NoNameTurnInput::default())
            .expect("trace should be stored");

        runtime.clear_traces();
        assert!(runtime.get_recent_traces().is_empty());
    }

    #[test]
    fn replace_trace_updates_existing_entry() {
        let mut runtime = NoNameRuntime::new(NoNameConfig::observe_only());
        let trace = runtime
            .run_turn(NoNameTurnInput::default())
            .expect("trace should be stored");
        let mut replaced = trace.clone();
        replaced.elapsed_ms = 77;

        let updated = runtime.replace_trace(replaced);

        assert!(updated);
        assert_eq!(runtime.get_recent_traces()[0].elapsed_ms, 77);
    }

    #[test]
    fn observe_only_runtime_can_run_director_observation() {
        use crate::noname_context_types::{NoNameContextPacket, NoNameContextSourceStat};
        use crate::noname_guardrails::NoNameGuardrailOutcome;
        use crate::numerical_system::ActionResult;

        let mut runtime = NoNameRuntime::new(NoNameConfig::observe_only());
        let result = runtime
            .run_director_observe_turn(
                NoNameTurnInput {
                    trace_id: "trace-1".to_string(),
                    session_id: "session-1".to_string(),
                    turn_id: "turn-1".to_string(),
                    caller_role: NoNameRole::Director,
                },
                "玩家选择返回山门",
                &NoNameContextPacket {
                    role: NoNameRole::Director,
                    hard_facts: vec!["玩家 位于 山门".to_string()],
                    working_memory: vec!["最近一次行动：返回山门".to_string()],
                    episodic_memory: vec!["玩家回到山门".to_string()],
                    narrative_notes: vec!["山门危机: 强敌逼近".to_string()],
                    chapter_summaries: vec!["第一章: 山门风云".to_string()],
                    recent_context: vec![],
                    referenced_entities: vec!["Character:player".to_string()],
                    compressed_summary: None,
                    token_budget_used: 32,
                    source_stats: vec![NoNameContextSourceStat {
                        source: "narrative".to_string(),
                        count: 1,
                    }],
                },
                Some(&NoNameDirectorGuardrailInput {
                    plot_state: crate::plot_engine::PlotState {
                        current_scene: crate::plot_engine::Scene {
                            id: "scene-1".to_string(),
                            name: "山门".to_string(),
                            description: "山门风声渐紧".to_string(),
                            location: "山门".to_string(),
                            available_options: vec![],
                        },
                        plot_history: vec![],
                        is_waiting_for_input: true,
                        interaction_state:
                            crate::plot_engine::PlotInteractionState::WaitingForChoice,
                        last_action_result: None,
                        settings: crate::plot_engine::PlotSettings::default(),
                        current_chapter: crate::plot_engine::ChapterState::new(
                            1,
                            "第一章".to_string(),
                        ),
                        chapters: vec![],
                        segment_count: 0,
                        last_generation_diagnostics: None,
                        last_option_generation_source: None,
                        last_consistency_risk_score: None,
                    },
                    action_result: ActionResult {
                        success: true,
                        description: "玩家返回山门".to_string(),
                        stat_changes: vec![],
                        events: vec![],
                    },
                    player_name: "无名弟子".to_string(),
                    player_realm_level: 1,
                    player_combat_power: 150,
                }),
            )
            .expect("director observe should succeed");

        assert_eq!(result.observation.role, NoNameRole::Director);
        assert_eq!(
            result.proposal.kind,
            crate::noname_types::NoNameProposalKind::PlotCandidate
        );
        assert_eq!(result.trace.capability_calls.len(), 3);
        assert_eq!(result.trace.proposals.len(), 1);
        assert!(!result.proposal.applyable);
        assert_eq!(result.proposal.status, NoNameProposalStatus::Observed);
        assert_eq!(
            result
                .trace
                .apply_result
                .as_ref()
                .map(|item| item.outcome.as_str()),
            Some("skipped_observe_only")
        );
        assert!(matches!(
            result.guardrail_result.as_ref().map(|item| item.outcome),
            Some(NoNameGuardrailOutcome::Accept | NoNameGuardrailOutcome::Repair)
        ));
        assert_eq!(runtime.get_recent_traces().len(), 1);
    }

    #[test]
    fn assisted_runtime_marks_proposal_applyable_when_guardrail_passes() {
        use crate::noname_context_types::{NoNameContextPacket, NoNameContextSourceStat};
        use crate::numerical_system::ActionResult;

        let mut runtime = NoNameRuntime::new(NoNameConfig::assisted());
        let result = runtime
            .run_director_observe_turn(
                NoNameTurnInput {
                    trace_id: "trace-assisted".to_string(),
                    session_id: "session-assisted".to_string(),
                    turn_id: "turn-assisted".to_string(),
                    caller_role: NoNameRole::Director,
                },
                "玩家准备查看山门异动",
                &NoNameContextPacket {
                    role: NoNameRole::Director,
                    hard_facts: vec!["玩家 位于 山门".to_string()],
                    working_memory: vec!["玩家留意山门异动".to_string()],
                    episodic_memory: vec!["山门异动加剧".to_string()],
                    narrative_notes: vec!["山门危机: 强敌逼近".to_string()],
                    chapter_summaries: vec!["第一章: 山门风云".to_string()],
                    recent_context: vec![],
                    referenced_entities: vec!["Character:player".to_string()],
                    compressed_summary: None,
                    token_budget_used: 32,
                    source_stats: vec![NoNameContextSourceStat {
                        source: "narrative".to_string(),
                        count: 1,
                    }],
                },
                Some(&NoNameDirectorGuardrailInput {
                    plot_state: crate::plot_engine::PlotState {
                        current_scene: crate::plot_engine::Scene {
                            id: "scene-1".to_string(),
                            name: "山门".to_string(),
                            description: "山门风声渐紧".to_string(),
                            location: "山门".to_string(),
                            available_options: vec![],
                        },
                        plot_history: vec![],
                        is_waiting_for_input: true,
                        interaction_state:
                            crate::plot_engine::PlotInteractionState::WaitingForChoice,
                        last_action_result: None,
                        settings: crate::plot_engine::PlotSettings::default(),
                        current_chapter: crate::plot_engine::ChapterState::new(
                            1,
                            "第一章".to_string(),
                        ),
                        chapters: vec![],
                        segment_count: 0,
                        last_generation_diagnostics: None,
                        last_option_generation_source: None,
                        last_consistency_risk_score: None,
                    },
                    action_result: ActionResult {
                        success: true,
                        description: "玩家查看山门异动".to_string(),
                        stat_changes: vec![],
                        events: vec![],
                    },
                    player_name: "无名弟子".to_string(),
                    player_realm_level: 1,
                    player_combat_power: 150,
                }),
            )
            .expect("assisted director run should succeed");

        assert!(result.proposal.applyable);
        assert_eq!(result.proposal.status, NoNameProposalStatus::Applied);
        assert!(result
            .proposal
            .labels
            .iter()
            .any(|item| item == "assisted_ready"));
        assert!(result
            .proposal
            .labels
            .iter()
            .any(|item| item == "apply_scope_diagnostics"));
        assert_eq!(result.trace.mode, NoNameMode::Assisted);
        assert_eq!(
            result
                .trace
                .apply_result
                .as_ref()
                .map(|item| item.outcome.as_str()),
            Some("applied_diagnostics_note")
        );
        assert!(result
            .trace
            .graph_path
            .contains(&NoNameTraceStage::ApplyProposal));
        assert!(result
            .trace
            .proposal_transition_log
            .iter()
            .any(|item| item.contains("applied:diagnostics")));
    }

    #[test]
    fn assisted_runtime_blocks_proposal_when_guardrail_rejects() {
        use crate::noname_context_types::{NoNameContextPacket, NoNameContextSourceStat};
        use crate::numerical_system::ActionResult;

        let mut runtime = NoNameRuntime::new(NoNameConfig::assisted());
        let result = runtime
            .run_director_observe_turn(
                NoNameTurnInput {
                    trace_id: "trace-assisted-reject".to_string(),
                    session_id: "session-assisted".to_string(),
                    turn_id: "turn-assisted-reject".to_string(),
                    caller_role: NoNameRole::Director,
                },
                "玩家继续观察山门异动",
                &NoNameContextPacket {
                    role: NoNameRole::Director,
                    hard_facts: vec!["玩家 位于 山门".to_string()],
                    working_memory: vec!["玩家继续观察山门异动".to_string()],
                    episodic_memory: vec!["山门异动加剧".to_string()],
                    narrative_notes: vec!["山门危机: 强敌逼近".to_string()],
                    chapter_summaries: vec!["第一章: 山门风云".to_string()],
                    recent_context: vec![],
                    referenced_entities: vec!["Character:player".to_string()],
                    compressed_summary: None,
                    token_budget_used: 32,
                    source_stats: vec![NoNameContextSourceStat {
                        source: "narrative".to_string(),
                        count: 1,
                    }],
                },
                Some(&NoNameDirectorGuardrailInput {
                    plot_state: crate::plot_engine::PlotState {
                        current_scene: crate::plot_engine::Scene {
                            id: "scene-1".to_string(),
                            name: "山门".to_string(),
                            description: "山门风声渐紧".to_string(),
                            location: "山门".to_string(),
                            available_options: vec![],
                        },
                        plot_history: vec![],
                        is_waiting_for_input: true,
                        interaction_state:
                            crate::plot_engine::PlotInteractionState::WaitingForChoice,
                        last_action_result: None,
                        settings: crate::plot_engine::PlotSettings::default(),
                        current_chapter: crate::plot_engine::ChapterState::new(
                            1,
                            "第一章".to_string(),
                        ),
                        chapters: vec![],
                        segment_count: 0,
                        last_generation_diagnostics: None,
                        last_option_generation_source: None,
                        last_consistency_risk_score: None,
                    },
                    action_result: ActionResult {
                        success: true,
                        description: "玩家继续观察山门异动".to_string(),
                        stat_changes: vec![],
                        events: vec![],
                    },
                    player_name: "无名弟子".to_string(),
                    player_realm_level: 999,
                    player_combat_power: 150,
                }),
            )
            .expect("assisted director run should succeed even when blocked");

        assert!(!result.proposal.applyable);
        assert_eq!(result.proposal.status, NoNameProposalStatus::Blocked);
        assert!(result.trace.fallback_used);
        assert_eq!(
            result
                .trace
                .apply_result
                .as_ref()
                .map(|item| item.outcome.as_str()),
            Some("preflight_blocked")
        );
        assert!(result
            .trace
            .graph_path
            .contains(&NoNameTraceStage::ApplyFallback));
    }

    #[test]
    fn assisted_runtime_falls_back_when_apply_guardrail_input_is_missing() {
        use crate::noname_context_types::{NoNameContextPacket, NoNameContextSourceStat};

        let mut runtime = NoNameRuntime::new(NoNameConfig::assisted());
        let result = runtime
            .run_director_observe_turn(
                NoNameTurnInput {
                    trace_id: "trace-assisted-missing-input".to_string(),
                    session_id: "session-assisted".to_string(),
                    turn_id: "turn-assisted-missing-input".to_string(),
                    caller_role: NoNameRole::Director,
                },
                "玩家继续观察山门异动",
                &NoNameContextPacket {
                    role: NoNameRole::Director,
                    hard_facts: vec!["玩家 位于 山门".to_string()],
                    working_memory: vec!["玩家继续观察山门异动".to_string()],
                    episodic_memory: vec!["山门异动加剧".to_string()],
                    narrative_notes: vec!["山门危机: 强敌逼近".to_string()],
                    chapter_summaries: vec!["第一章: 山门风云".to_string()],
                    recent_context: vec![],
                    referenced_entities: vec!["Character:player".to_string()],
                    compressed_summary: None,
                    token_budget_used: 32,
                    source_stats: vec![NoNameContextSourceStat {
                        source: "narrative".to_string(),
                        count: 1,
                    }],
                },
                None,
            )
            .expect("assisted director run should succeed without guardrail input");

        assert!(!result.proposal.applyable);
        assert_eq!(result.proposal.status, NoNameProposalStatus::Fallback);
        assert_eq!(
            result
                .trace
                .apply_result
                .as_ref()
                .map(|item| item.outcome.as_str()),
            Some("preflight_fallback_required")
        );
        assert!(result
            .trace
            .graph_path
            .contains(&NoNameTraceStage::ApplyFallback));
    }
}
