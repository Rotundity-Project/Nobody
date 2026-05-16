use crate::noname_agent_registry::NoNameAgentRegistry;
use crate::noname_config::NoNameConfig;
use crate::noname_context_builder::specialize_context_packet;
use crate::noname_context_types::NoNameContextPacket;
use crate::noname_errors::{NoNameError, NoNameErrorKind};
use crate::noname_graph::NoNameGraphExecutor;
use crate::noname_guardrails::{
    validate_director_observation, validate_director_proposal_for_apply,
    NoNameDirectorGuardrailInput, NoNameGuardrailResult,
};
use crate::noname_output_interface::{
    controlled_output_policy_from_role_context, NoNameControlledOutputDecision,
    NoNameControlledOutputInterface, NoNameControlledOutputKind,
};
use crate::noname_protocol_agent::{NoNameAgentMessage, NoNameAgentMessageKind};
use crate::noname_protocol_runtime::NoNameProtocolRuntime;
use crate::noname_protocol_types::{NoNameAgentAddress, NoNameProtocolHeader, NoNameTraceWritable};
use crate::noname_roles::{DirectorAgent, NoNameDirectorObservation, NoNameRoleObservation};
use crate::noname_tools::build_director_registry;
use crate::noname_trace::{NoNameRelatedObservationRecord, NoNameTrace};
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
    agent_registry: NoNameAgentRegistry,
    protocol_runtime: NoNameProtocolRuntime,
    recent_traces: VecDeque<NoNameTrace>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoNameDirectorRunResult {
    pub trace: NoNameTrace,
    pub observation: NoNameDirectorObservation,
    pub proposal: NoNameProposal,
    pub guardrail_result: Option<NoNameGuardrailResult>,
    pub related_observations: Vec<NoNameRoleObservation>,
}

impl NoNameRuntime {
    pub fn new(config: NoNameConfig) -> Self {
        Self {
            config,
            graph_executor: NoNameGraphExecutor::new(),
            agent_registry: NoNameAgentRegistry::new_default(),
            protocol_runtime: NoNameProtocolRuntime::new(),
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
        let proposal = self.record_assisted_apply_preflight(
            &mut trace,
            proposal,
            guardrail_input,
            context_packet,
        );
        observation.proposal = proposal.clone();
        trace.replace_last_proposal(proposal.clone());
        let related_observations =
            self.run_protocol_observe_fan_out(&input, &mut trace, context_packet, action_summary)?;
        trace.replace_related_observations(
            related_observations
                .iter()
                .map(|observation| NoNameRelatedObservationRecord {
                    role: observation.role,
                    action_summary: observation.action_summary.clone(),
                    focus: observation.focus.clone(),
                    rationale: observation.rationale.clone(),
                    role_goal: observation.role_goal.clone(),
                    scene_focus: observation.scene_focus.clone(),
                    forbidden_scopes: observation.forbidden_scopes.clone(),
                    note_type_hits: observation.note_type_hits.clone(),
                    source_stats: observation.source_stats.clone(),
                    context_token_budget_used: observation.context_token_budget_used,
                    context_slice_stats: observation.context_slice_stats.clone(),
                    proposal: observation.proposal.clone(),
                })
                .collect(),
        );
        trace.elapsed_ms = start.elapsed().as_millis() as u64;

        if self.config.trace_policy.enabled {
            self.store_trace(trace.clone())?;
        }

        Ok(NoNameDirectorRunResult {
            trace,
            observation,
            proposal,
            guardrail_result,
            related_observations,
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

    pub fn get_trace_by_id(&self, trace_id: &str) -> Option<NoNameTrace> {
        self.recent_traces
            .iter()
            .rev()
            .find(|trace| trace.trace_id == trace_id)
            .cloned()
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

    pub fn update_trace_by_id<F>(
        &mut self,
        trace_id: &str,
        update: F,
    ) -> Result<NoNameTrace, String>
    where
        F: FnOnce(&mut NoNameTrace) -> Result<(), String>,
    {
        let trace = self
            .recent_traces
            .iter_mut()
            .rev()
            .find(|trace| trace.trace_id == trace_id)
            .ok_or_else(|| format!("NoName trace not found: {}", trace_id))?;
        let mut updated = trace.clone();
        update(&mut updated)?;
        *trace = updated.clone();
        Ok(updated)
    }

    pub fn clear_traces(&mut self) {
        self.recent_traces.clear();
        self.protocol_runtime.clear();
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
        context_packet: &NoNameContextPacket,
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
                            || proposal
                                .apply_scopes
                                .contains(&NoNameApplyScope::Diagnostics)
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
                        let (review_count, needs_review_count) =
                            self.record_controlled_output_reviews(trace, &proposal, context_packet);
                        if review_count > 0 {
                            trace.record_apply_execution(
                                "runtime.controlled_output_review",
                                "recorded",
                                Some(format!(
                                    "已记录{}条受控输出 review，其中{}条需要人工复核",
                                    review_count, needs_review_count
                                )),
                            );
                        }
                        if needs_review_count > 0
                            && !proposal
                                .labels
                                .iter()
                                .any(|item| item == "controlled_output_needs_review")
                        {
                            proposal
                                .labels
                                .push("controlled_output_needs_review".to_string());
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

    fn record_controlled_output_reviews(
        &self,
        trace: &mut NoNameTrace,
        proposal: &NoNameProposal,
        context_packet: &NoNameContextPacket,
    ) -> (usize, usize) {
        let role_context = specialize_context_packet(context_packet);
        let interface = NoNameControlledOutputInterface::new(
            controlled_output_policy_from_role_context(&role_context),
        );
        let policy_forbidden_scopes = interface.policy().forbidden_scopes.clone();
        let scopes = if proposal.apply_scopes.is_empty() {
            vec![NoNameApplyScope::Diagnostics]
        } else {
            proposal.apply_scopes.clone()
        };
        let mut review_count = 0;
        let mut needs_review_count = 0;

        for scope in scopes {
            let kind = controlled_output_kind_for_scope(scope);
            let mut request = interface.draft_stub(
                kind,
                proposal.producer_role,
                proposal.title.clone(),
                proposal.summary.clone(),
            );
            request.request_id = format!(
                "controlled-output-{}-{}",
                proposal.proposal_id,
                scope.as_str()
            );
            request.proposal_ref = Some(proposal.to_ref());
            request.target_scope = scope;
            request
                .labels
                .push("runtime_assisted_preflight".to_string());

            let review = interface.review(&request);
            if review.decision == NoNameControlledOutputDecision::NeedsReview {
                needs_review_count += 1;
            }
            trace.record_proposal_transition(format!(
                "{}:controlled_output:{}:{}",
                proposal.proposal_id,
                scope.as_str(),
                controlled_output_decision_key(review.decision)
            ));
            trace.record_controlled_output_review(
                Some(proposal.proposal_id.clone()),
                kind,
                policy_forbidden_scopes.clone(),
                review,
            );
            review_count += 1;
        }

        (review_count, needs_review_count)
    }

    fn run_protocol_observe_fan_out(
        &mut self,
        input: &NoNameTurnInput,
        trace: &mut NoNameTrace,
        context_packet: &NoNameContextPacket,
        action_summary: &str,
    ) -> Result<Vec<NoNameRoleObservation>, NoNameError> {
        if !self.config.mode.is_enabled() {
            return Ok(Vec::new());
        }

        self.protocol_runtime.clear();

        let orchestrator = NoNameAgentAddress {
            agent_id: NoNameRole::Director.as_str().to_string(),
            role: NoNameRole::Director,
            runtime: "local".to_string(),
        };
        let parent_task_id = format!("{}-director-observe", input.turn_id);
        let mut observations = Vec::new();

        for role in NoNameGraphExecutor::default_role_dispatch_order()
            .iter()
            .copied()
            .filter(|role| *role != NoNameRole::Director)
        {
            let header =
                NoNameProtocolHeader::new(trace.trace_id.clone(), trace.session_id.clone());
            let callee = NoNameAgentAddress {
                agent_id: role.as_str().to_string(),
                role,
                runtime: "local".to_string(),
            };
            let base_lifecycle = crate::noname_protocol_types::NoNameTaskLifecycle::new(format!(
                "{}-{}-observe",
                input.turn_id,
                role.as_str()
            ))
            .with_parent(parent_task_id.clone())
            .with_timeout(self.config.timeout_policy.planning_timeout_ms);

            let task_request = NoNameAgentMessage::new(
                header.clone(),
                orchestrator.clone(),
                callee.clone(),
                NoNameAgentMessageKind::TaskRequest,
                base_lifecycle.clone(),
                serde_json::json!({
                    "actionSummary": action_summary,
                    "callerRole": input.caller_role.as_str(),
                    "targetRole": role.as_str(),
                }),
            );
            let queued = self
                .protocol_runtime
                .submit_agent_message(task_request.clone())?;
            task_request.record_on_trace(trace, queued.status.as_str());

            let mut base_context = context_packet.clone();
            base_context.role = role;
            let role_context_packet = specialize_context_packet(&base_context);

            let delegation = NoNameAgentMessage::new(
                header.clone(),
                orchestrator.clone(),
                callee.clone(),
                NoNameAgentMessageKind::Delegation,
                queued.clone(),
                serde_json::json!({
                    "goal": action_summary,
                    "targetRole": role.as_str(),
                    "roleGoal": &role_context_packet.role_goal,
                    "sceneFocus": &role_context_packet.scene_focus,
                    "forbiddenScopes": &role_context_packet.forbidden_scopes,
                }),
            );
            let running = self
                .protocol_runtime
                .submit_agent_message(delegation.clone())?;
            delegation.record_on_trace(trace, running.status.as_str());

            let mut role_trace = NoNameTrace::empty(
                trace.trace_id.clone(),
                trace.session_id.clone(),
                trace.turn_id.clone(),
                trace.mode,
            );

            match self.agent_registry.dispatch_role_context_observe_turn(
                &mut role_trace,
                &role_context_packet,
                action_summary,
            ) {
                Ok(observation) => {
                    trace.capability_calls.extend(role_trace.capability_calls);
                    trace.record_proposal_transition(format!(
                        "{}:fanout:{}:{}",
                        observation.proposal.proposal_id,
                        role.as_str(),
                        observation.proposal.status.as_str()
                    ));

                    let result_message = NoNameAgentMessage::new(
                        header,
                        callee,
                        orchestrator.clone(),
                        NoNameAgentMessageKind::Result,
                        running,
                        serde_json::json!({
                            "proposalId": observation.proposal.proposal_id,
                            "proposalKind": format!("{:?}", observation.proposal.kind),
                            "focus": observation.focus,
                        }),
                    );
                    let completed = self
                        .protocol_runtime
                        .submit_agent_message(result_message.clone())?;
                    result_message.record_on_trace(trace, completed.status.as_str());
                    observations.push(observation);
                }
                Err(error) => {
                    let error_message = NoNameAgentMessage::new(
                        header,
                        callee,
                        orchestrator.clone(),
                        NoNameAgentMessageKind::Error,
                        running,
                        serde_json::json!({
                            "code": error.code,
                            "message": error.message,
                        }),
                    );
                    let failed = self
                        .protocol_runtime
                        .submit_agent_message(error_message.clone())?;
                    error_message.record_on_trace(trace, failed.status.as_str());
                    trace.record_proposal_transition(format!(
                        "{}:fanout:{}:error:{}",
                        input.turn_id,
                        role.as_str(),
                        error_message.payload["code"].as_str().unwrap_or("unknown")
                    ));
                }
            }
        }

        Ok(observations)
    }
}

fn controlled_output_kind_for_scope(scope: NoNameApplyScope) -> NoNameControlledOutputKind {
    match scope {
        NoNameApplyScope::Diagnostics => NoNameControlledOutputKind::NarrativeNote,
        NoNameApplyScope::ChapterSummaryHint => NoNameControlledOutputKind::RecapNote,
        NoNameApplyScope::OptionBiasHint => NoNameControlledOutputKind::IntermediateNarrativeHint,
        NoNameApplyScope::PlotAugmentationHint => {
            NoNameControlledOutputKind::NonFinalPlotAugmentation
        }
        NoNameApplyScope::PlotTextHint => NoNameControlledOutputKind::SceneAugmentation,
    }
}

fn controlled_output_decision_key(decision: NoNameControlledOutputDecision) -> &'static str {
    match decision {
        NoNameControlledOutputDecision::Allow => "allow",
        NoNameControlledOutputDecision::Reject => "reject",
        NoNameControlledOutputDecision::NeedsReview => "needs_review",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::noname_output_interface::NoNameForbiddenOutputScope;
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
    fn update_trace_by_id_mutates_existing_trace() {
        let mut runtime = NoNameRuntime::new(NoNameConfig::observe_only());
        runtime
            .run_turn(NoNameTurnInput::default())
            .expect("trace should be stored");

        let updated = runtime
            .update_trace_by_id("noname-trace-0", |trace| {
                trace.elapsed_ms = 88;
                Ok(())
            })
            .expect("trace update should succeed");

        assert_eq!(updated.elapsed_ms, 88);
        assert_eq!(runtime.get_recent_traces()[0].elapsed_ms, 88);
    }

    #[test]
    fn update_trace_by_id_reports_missing_trace() {
        let mut runtime = NoNameRuntime::new(NoNameConfig::observe_only());

        let error = runtime
            .update_trace_by_id("missing-trace", |_| Ok(()))
            .expect_err("missing trace should fail");

        assert_eq!(error, "NoName trace not found: missing-trace");
    }

    #[test]
    fn update_trace_by_id_keeps_failed_update_mutation_local_to_closure() {
        let mut runtime = NoNameRuntime::new(NoNameConfig::observe_only());
        runtime
            .run_turn(NoNameTurnInput::default())
            .expect("trace should be stored");

        let error = runtime
            .update_trace_by_id("noname-trace-0", |trace| {
                trace.elapsed_ms = 99;
                Err("review decision rejected".to_string())
            })
            .expect_err("failed update should bubble up");

        assert_eq!(error, "review decision rejected");
        assert_eq!(runtime.get_recent_traces()[0].elapsed_ms, 0);
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
                        pending_plot_augmentation_hints: Vec::new(),
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
        assert_eq!(result.related_observations.len(), 3);
        assert_eq!(result.trace.related_observations.len(), 3);
        assert!(result.trace.related_observations.iter().any(|item| {
            item.role == NoNameRole::WorldCurator
                && item
                    .role_goal
                    .as_deref()
                    .unwrap_or_default()
                    .contains("Maintain world facts")
                && item
                    .scene_focus
                    .as_deref()
                    .unwrap_or_default()
                    .contains("玩家 位于 山门")
                && item
                    .forbidden_scopes
                    .iter()
                    .any(|scope| scope.contains("NPC private intent"))
                && item.source_stats.iter().any(|source| source.count > 0)
                && item.context_token_budget_used.unwrap_or_default() > 0
        }));
        assert_eq!(result.trace.protocol_events.len(), 9);
        assert_eq!(result.trace.capability_calls.len(), 21);
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
        assert!(result.trace.controlled_output_reviews.is_empty());
        assert!(matches!(
            result.guardrail_result.as_ref().map(|item| item.outcome),
            Some(NoNameGuardrailOutcome::Accept | NoNameGuardrailOutcome::Repair)
        ));
        assert!(result
            .trace
            .proposal_transition_log
            .iter()
            .any(|item| item.contains(":fanout:world_curator:observed")));
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
                        pending_plot_augmentation_hints: Vec::new(),
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
        assert_eq!(result.trace.controlled_output_reviews.len(), 5);
        assert!(result.trace.controlled_output_reviews.iter().any(|item| {
            item.decision == NoNameControlledOutputDecision::NeedsReview
                && item.safe_apply_scope == Some(NoNameApplyScope::PlotTextHint)
                && item
                    .policy_forbidden_scopes
                    .contains(&NoNameForbiddenOutputScope::FinalPlotState)
                && item
                    .policy_forbidden_scopes
                    .contains(&NoNameForbiddenOutputScope::CanonWorldFact)
                && item.requires_human_review
        }));
        assert!(result
            .trace
            .proposal_transition_log
            .iter()
            .any(|item| item.contains("controlled_output:plot_text_hint:needs_review")));
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
                        pending_plot_augmentation_hints: Vec::new(),
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
        assert!(result.trace.controlled_output_reviews.is_empty());
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
        assert!(result.trace.controlled_output_reviews.is_empty());
        assert!(result
            .trace
            .graph_path
            .contains(&NoNameTraceStage::ApplyFallback));
    }
}
