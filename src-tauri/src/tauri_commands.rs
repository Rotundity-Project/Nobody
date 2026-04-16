use crate::app_error::AppError;
use crate::context_builder::{build_context_bundle, ContextBuildInput, ContextBundle};
use crate::entity_store::{EntityQuery, EntityStore};
use crate::entity_types::{
    EntityCandidateRequest, EntityType, ResolvedEntity, StoredEntity, ValidationStatus,
};
use crate::entity_validator::resolve_candidate;
use crate::event_log::EventImportance;
use crate::game_engine::GameEngine;
use crate::game_state::GameState;
use crate::llm_runtime_config::{
    clear_runtime_llm_config, get_llm_config_status as runtime_llm_config_status,
    resolve_llm_config, set_runtime_llm_config, LLMConfigStatus,
};
use crate::llm_service::{LLMConfig, LLMRequest, LLMService};
use crate::memory_layers::{ChapterSummary, MemoryEntry, WorldFact};
use crate::noname_config::NoNameConfig;
use crate::noname_context_builder::build_context_packet;
use crate::noname_context_types::NoNameContextBuildInput;
use crate::noname_guardrails::{NoNameDirectorGuardrailInput, NoNameGuardrailResult};
use crate::noname_memory_manager::NoNameMemoryManager;
use crate::noname_roles::NoNameDirectorObservation;
use crate::noname_runtime::{NoNameDirectorRunResult, NoNameRuntime, NoNameTurnInput};
use crate::noname_trace::NoNameTrace;
use crate::noname_types::{NoNameApplyScope, NoNameMode, NoNameRole, NoNameTargetSegment};
use crate::novel_generator::{Novel, NovelGenerator};
use crate::numerical_system::{Action, Context, StatChange};
use crate::plot_consistency::{
    get_runtime_policy, reset_runtime_policy, update_runtime_policy,
    validate_and_repair_plot_update, ConsistencyPolicy, ConsistencyReport,
};
use crate::plot_engine::{
    PlayerAction, PlayerOption, PlotEngine, PlotInteractionState, PlotSettings, PlotState,
};
use crate::runtime_prompt_baseline::{WORLD_REGISTRY_REFERENCE, WORLD_REGISTRY_SUPPLEMENT};
use crate::save_load::{MigrationBatchReport, SaveInfo};
use crate::script::Script;
use crate::world_registry::{apply_registry_to_game_state, WorldRegistry};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::State;

static ENTITY_STORE: OnceLock<Mutex<EntityStore>> = OnceLock::new();
static MEMORY_LAYERS: OnceLock<Mutex<crate::memory_layers::MemoryLayers>> = OnceLock::new();
static NONAME_RUNTIME: OnceLock<Mutex<NoNameRuntime>> = OnceLock::new();

fn entity_store() -> &'static Mutex<EntityStore> {
    ENTITY_STORE.get_or_init(|| Mutex::new(EntityStore::new()))
}

fn memory_layers() -> &'static Mutex<crate::memory_layers::MemoryLayers> {
    MEMORY_LAYERS.get_or_init(|| Mutex::new(crate::memory_layers::MemoryLayers::new()))
}

fn noname_runtime() -> &'static Mutex<NoNameRuntime> {
    NONAME_RUNTIME.get_or_init(|| Mutex::new(NoNameRuntime::new(NoNameConfig::observe_only())))
}

fn noname_mode_label(mode: NoNameMode) -> &'static str {
    match mode {
        NoNameMode::Disabled => "disabled",
        NoNameMode::ObserveOnly => "observe_only",
        NoNameMode::Assisted => "assisted",
    }
}

fn build_noname_action_summary(action: &PlayerAction, plot_state: &PlotState) -> String {
    if let Some(selected_option_id) = action.selected_option_id {
        if let Some(option) = plot_state
            .current_scene
            .available_options
            .get(selected_option_id)
        {
            return format!("选择选项: {}", option.description);
        }
    }

    if !action.content.trim().is_empty() {
        return format!("自由输入: {}", action.content.trim());
    }

    "玩家执行了默认推进动作".to_string()
}

fn append_noname_observation_diagnostics(
    existing: &mut Option<String>,
    mode: NoNameMode,
    observation: &NoNameDirectorObservation,
    guardrail_result: Option<&NoNameGuardrailResult>,
    trace: &NoNameTrace,
) {
    let mut segments = vec![
        format!("focus={}", observation.focus),
        format!("rationale={}", observation.rationale),
        format!("proposal={}", observation.proposal.title),
        format!(
            "target_segment={}",
            observation.proposal.target_segment.as_str()
        ),
        format!("intended_effect={}", observation.proposal.intended_effect),
        format!("proposal_status={}", observation.proposal.status.as_str()),
        format!(
            "apply_scopes={}",
            if observation.proposal.apply_scopes.is_empty() {
                "none".to_string()
            } else {
                observation
                    .proposal
                    .apply_scopes
                    .iter()
                    .map(|scope| scope.as_str())
                    .collect::<Vec<_>>()
                    .join(",")
            }
        ),
        format!(
            "applyable={}",
            if observation.proposal.applyable {
                "yes"
            } else {
                "no"
            }
        ),
    ];
    if let Some(result) = guardrail_result {
        segments.push(format!("guardrail={}", result.outcome.as_str()));
        if let Some(reason) = &result.reason {
            segments.push(format!("reason={}", reason));
        }
    }
    if let Some(apply_result) = &trace.apply_result {
        segments.push(format!("apply={}", apply_result.outcome));
        if let Some(reason) = &apply_result.reason {
            segments.push(format!("apply_reason={}", reason));
        }
    }
    let note = format!(
        "NoName.{}：{}",
        noname_mode_label(mode),
        segments.join("；")
    );
    match existing {
        Some(diag) => {
            diag.push('；');
            diag.push_str(&note);
        }
        None => {
            *existing = Some(note);
        }
    }
}

fn build_noname_summary_hint(proposal: &crate::noname_types::NoNameProposal) -> String {
    format!("NoName提示：后续重点关注{}", proposal.focus.trim())
}

fn build_noname_option_bias_hint(proposal: &crate::noname_types::NoNameProposal) -> String {
    format!(
        "NoName选项偏置：下轮优先围绕{}提供行动切入点",
        proposal.focus.trim()
    )
}

fn build_noname_plot_text_hint(proposal: &crate::noname_types::NoNameProposal) -> String {
    format!("【NoName】重点关注：{}", proposal.focus.trim())
}

fn proposal_allows_apply_scope(
    proposal: &crate::noname_types::NoNameProposal,
    scope: NoNameApplyScope,
) -> bool {
    proposal.apply_scopes.is_empty() || proposal.apply_scopes.contains(&scope)
}

fn target_segment_supports_apply_scope(
    target_segment: NoNameTargetSegment,
    scope: NoNameApplyScope,
) -> bool {
    match scope {
        NoNameApplyScope::PlotTextHint => matches!(
            target_segment,
            NoNameTargetSegment::CurrentTurnHead | NoNameTargetSegment::CurrentTurnTail
        ),
        _ => true,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NoNameApplyTargetDecision {
    Apply,
    Skip {
        outcome: &'static str,
        note: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NoNameApplyTargetPlan {
    target: &'static str,
    priority: u32,
    order: u32,
    decision: NoNameApplyTargetDecision,
}

fn priority_for_apply_scope(scope: NoNameApplyScope) -> u32 {
    match scope {
        NoNameApplyScope::PlotTextHint => 300,
        NoNameApplyScope::ChapterSummaryHint => 200,
        NoNameApplyScope::OptionBiasHint => 100,
        NoNameApplyScope::Diagnostics => 50,
    }
}

fn plan_noname_apply_target(
    proposal: &crate::noname_types::NoNameProposal,
    scope: NoNameApplyScope,
    plot_state: Option<&PlotState>,
    plot_text: Option<&str>,
) -> NoNameApplyTargetPlan {
    let target = scope.as_str();
    let priority = priority_for_apply_scope(scope);
    if !proposal_allows_apply_scope(proposal, scope) {
        return NoNameApplyTargetPlan {
            target,
            priority,
            order: 0,
            decision: NoNameApplyTargetDecision::Skip {
                outcome: "skipped_scope_forbidden",
                note: format!("提案未声明 {} 作用域，跳过对应输出", target),
            },
        };
    }

    if !target_segment_supports_apply_scope(proposal.target_segment, scope) {
        return NoNameApplyTargetPlan {
            target,
            priority,
            order: 0,
            decision: NoNameApplyTargetDecision::Skip {
                outcome: "skipped_target_mismatch",
                note: format!(
                    "目标段 {} 不支持 {}，跳过对应输出",
                    proposal.target_segment.as_str(),
                    target
                ),
            },
        };
    }

    match scope {
        NoNameApplyScope::PlotTextHint => {
            let current_text = plot_text.unwrap_or_default();
            if current_text.trim().is_empty() {
                return NoNameApplyTargetPlan {
                    target,
                    priority,
                    order: 0,
                    decision: NoNameApplyTargetDecision::Skip {
                        outcome: "skipped_empty_plot_text",
                        note: "当前正文为空，跳过受控正文提示".to_string(),
                    },
                };
            }
            if current_text.contains("【NoName】") || current_text.contains("NoName提示") {
                return NoNameApplyTargetPlan {
                    target,
                    priority,
                    order: 0,
                    decision: NoNameApplyTargetDecision::Skip {
                        outcome: "skipped_duplicate",
                        note: "正文已包含 NoName 标记，跳过重复提示".to_string(),
                    },
                };
            }
        }
        NoNameApplyScope::ChapterSummaryHint => {
            if let Some(state) = plot_state {
                let summary = state.current_chapter.summary.trim();
                if !summary.is_empty() && summary.contains(proposal.focus.trim()) {
                    return NoNameApplyTargetPlan {
                        target,
                        priority,
                        order: 0,
                        decision: NoNameApplyTargetDecision::Skip {
                            outcome: "skipped_duplicate",
                            note: format!(
                                "章节摘要已覆盖“{}”，跳过重复提示",
                                proposal.focus.trim()
                            ),
                        },
                    };
                }
            }
        }
        NoNameApplyScope::OptionBiasHint => {
            let Some(state) = plot_state else {
                return NoNameApplyTargetPlan {
                    target,
                    priority,
                    order: 0,
                    decision: NoNameApplyTargetDecision::Skip {
                        outcome: "skipped_missing_plot_state",
                        note: "缺少 plot_state，跳过选项偏置提示".to_string(),
                    },
                };
            };
            if !state.is_waiting_for_input {
                return NoNameApplyTargetPlan {
                    target,
                    priority,
                    order: 0,
                    decision: NoNameApplyTargetDecision::Skip {
                        outcome: "skipped_not_waiting",
                        note: "当前不处于等待输入状态，跳过选项偏置提示".to_string(),
                    },
                };
            }
            let option_hint = build_noname_option_bias_hint(proposal);
            let diagnostics = state.last_generation_diagnostics.as_deref().unwrap_or_default();
            if diagnostics.contains(option_hint.as_str()) {
                return NoNameApplyTargetPlan {
                    target,
                    priority,
                    order: 0,
                    decision: NoNameApplyTargetDecision::Skip {
                        outcome: "skipped_duplicate",
                        note: format!(
                            "诊断中已包含选项偏置提示“{}”，跳过重复写入",
                            proposal.focus.trim()
                        ),
                    },
                };
            }
        }
        NoNameApplyScope::Diagnostics => {}
    }

    NoNameApplyTargetPlan {
        target,
        priority,
        order: 0,
        decision: NoNameApplyTargetDecision::Apply,
    }
}

fn build_noname_apply_plan_set(
    proposal: &crate::noname_types::NoNameProposal,
    plot_state: Option<&PlotState>,
    plot_text: Option<&str>,
) -> Vec<NoNameApplyTargetPlan> {
    let mut plans = vec![
        plan_noname_apply_target(proposal, NoNameApplyScope::PlotTextHint, plot_state, plot_text),
        plan_noname_apply_target(proposal, NoNameApplyScope::ChapterSummaryHint, plot_state, None),
        plan_noname_apply_target(proposal, NoNameApplyScope::OptionBiasHint, plot_state, None),
    ];
    plans.sort_by(|left, right| {
        right
            .priority
            .cmp(&left.priority)
            .then_with(|| left.target.cmp(right.target))
    });
    for (index, plan) in plans.iter_mut().enumerate() {
        plan.order = (index + 1) as u32;
    }
    plans
}

fn record_noname_apply_plan(
    trace: &mut NoNameTrace,
    plan: &NoNameApplyTargetPlan,
) {
    let (decision, note) = match &plan.decision {
        NoNameApplyTargetDecision::Apply => (
            "apply",
            Some(format!(
                "允许执行 {}，优先级 {}，顺位 #{}",
                plan.target, plan.priority, plan.order
            )),
        ),
        NoNameApplyTargetDecision::Skip { note, .. } => ("skip", Some(note.clone())),
    };
    trace.record_apply_plan(plan.order, plan.target, decision, plan.priority, note);
}

fn record_noname_apply_plan_and_skip_execution(
    trace: &mut NoNameTrace,
    plan: &NoNameApplyTargetPlan,
) {
    record_noname_apply_plan(trace, plan);
    if let NoNameApplyTargetDecision::Skip { outcome, note } = &plan.decision {
        trace.record_apply_execution(plan.target, *outcome, Some(note.clone()));
    }
}

fn apply_noname_plot_text_hint(
    plot_text: &mut String,
    noname_result: &mut NoNameDirectorRunResult,
) -> bool {
    if noname_result.proposal.status != crate::noname_types::NoNameProposalStatus::Applied {
        return false;
    }

    let plan = plan_noname_apply_target(
        &noname_result.proposal,
        NoNameApplyScope::PlotTextHint,
        None,
        Some(plot_text.as_str()),
    );
    if !matches!(plan.decision, NoNameApplyTargetDecision::Apply) {
        if let NoNameApplyTargetDecision::Skip { outcome, note } = &plan.decision {
            noname_result
                .trace
                .record_apply_execution(plan.target, *outcome, Some(note.clone()));
        }
        return false;
    }

    let hint = build_noname_plot_text_hint(&noname_result.proposal);
    match noname_result.proposal.target_segment {
        NoNameTargetSegment::CurrentTurnHead => {
            let original = plot_text.trim().to_string();
            *plot_text = format!("{}\n\n{}", hint, original);
        }
        _ => {
            plot_text.push_str("\n\n");
            plot_text.push_str(&hint);
        }
    }
    noname_result.trace.record_proposal_transition(format!(
        "{}:applied:plot_text_hint",
        noname_result.proposal.proposal_id
    ));
    noname_result.trace.record_apply_execution(
        "plot_text_hint",
        "applied",
        Some(format!(
            "已将提案提示插入正文，聚焦“{}”",
            noname_result.proposal.focus
        )),
    );
    true
}

fn apply_noname_low_risk_outputs(
    plot_state: &mut PlotState,
    noname_result: &mut NoNameDirectorRunResult,
    plot_text_applied: bool,
) {
    if noname_result.proposal.status != crate::noname_types::NoNameProposalStatus::Applied {
        return;
    }

    let mut applied_targets: Vec<&str> = Vec::new();

    let summary_plan = plan_noname_apply_target(
        &noname_result.proposal,
        NoNameApplyScope::ChapterSummaryHint,
        Some(plot_state),
        None,
    );
    if matches!(summary_plan.decision, NoNameApplyTargetDecision::Apply) {
        let hint = build_noname_summary_hint(&noname_result.proposal);
        let summary = plot_state.current_chapter.summary.trim();
        if summary.is_empty() {
            plot_state.current_chapter.summary = hint.clone();
        } else {
            plot_state.current_chapter.summary = match noname_result.proposal.target_segment {
                NoNameTargetSegment::ChapterSummaryHead => format!("{}；{}", hint, summary),
                _ => format!("{}；{}", summary, hint),
            };
        }
        noname_result.trace.record_proposal_transition(format!(
            "{}:applied:chapter_summary_hint",
            noname_result.proposal.proposal_id
        ));
        noname_result.trace.record_apply_execution(
            "chapter_summary_hint",
            "applied",
            Some(format!(
                "已写入章节摘要提示，聚焦“{}”",
                noname_result.proposal.focus
            )),
        );
        applied_targets.push("chapter_summary_hint");
    } else {
        record_noname_apply_plan_and_skip_execution(&mut noname_result.trace, &summary_plan);
    }

    let option_bias_plan = plan_noname_apply_target(
        &noname_result.proposal,
        NoNameApplyScope::OptionBiasHint,
        Some(plot_state),
        None,
    );
    if matches!(option_bias_plan.decision, NoNameApplyTargetDecision::Apply) {
        let option_hint = build_noname_option_bias_hint(&noname_result.proposal);
        let diagnostics = plot_state
            .last_generation_diagnostics
            .clone()
            .unwrap_or_default();
        if diagnostics.is_empty() {
            plot_state.last_generation_diagnostics = Some(option_hint.clone());
        } else if !diagnostics.contains(option_hint.as_str()) {
            plot_state.last_generation_diagnostics =
                Some(format!("{}；{}", diagnostics, option_hint));
        }
        noname_result.trace.record_proposal_transition(format!(
            "{}:applied:option_bias_hint",
            noname_result.proposal.proposal_id
        ));
        noname_result.trace.record_apply_execution(
            "option_bias_hint",
            "applied",
            Some(format!(
                "已写入下轮选项偏置提示，聚焦“{}”",
                noname_result.proposal.focus
            )),
        );
        applied_targets.push("option_bias_hint");
    } else {
        record_noname_apply_plan_and_skip_execution(&mut noname_result.trace, &option_bias_plan);
    }

    if plot_text_applied {
        applied_targets.push("plot_text_hint");
    }
    let apply_outcome = if applied_targets.is_empty() {
        "applied_no_scoped_output"
    } else {
        "applied_scoped_outputs"
    };
    noname_result.trace.set_apply_result(
        true,
        apply_outcome,
        Some(format!(
            "已应用作用域：{}；聚焦“{}”",
            if applied_targets.is_empty() {
                "无".to_string()
            } else {
                applied_targets.join(", ")
            },
            noname_result.proposal.focus
        )),
    );

    if let Ok(mut runtime) = noname_runtime().lock() {
        let _ = runtime.replace_trace(noname_result.trace.clone());
    }
}

fn run_noname_observe_only_for_turn(
    action_summary: &str,
    game_state: &GameState,
    plot_state: &PlotState,
    timestamp: u64,
) -> Result<NoNameDirectorRunResult, String> {
    let mut memory_manager = NoNameMemoryManager::new();
    if let Ok(memory) = memory_layers().lock() {
        memory_manager.ingest_legacy_layers(&memory);
    }
    memory_manager.push_working_memory(
        crate::noname_memory_types::NoNameWorkingMemoryItem {
            memory_id: format!("work-{}", timestamp),
            turn_id: format!("turn-{}", timestamp),
            source: "execute_player_action".to_string(),
            category: "recent_turn".to_string(),
            summary: action_summary.to_string(),
            expires_at: None,
            priority: 10,
        },
        8,
    );

    let store = entity_store()
        .lock()
        .map_err(|_| "NoName observe-only: entity store lock poisoned".to_string())?;
    let context_packet = build_context_packet(
        &store,
        &memory_manager,
        &NoNameContextBuildInput {
            role: NoNameRole::Director,
            world_id: game_state.script.id.clone(),
            run_id: "active-run".to_string(),
            scene_id: plot_state.current_scene.id.clone(),
            character_ids: vec![game_state.player.id.clone()],
            map_node_id: Some(game_state.player.location.clone()),
            player_intent: if action_summary.trim().is_empty() {
                None
            } else {
                Some(action_summary.to_string())
            },
            recent_context_lines: plot_state
                .current_chapter
                .content
                .iter()
                .rev()
                .take(6)
                .cloned()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect(),
            token_budget: 320,
            per_section_limit: 4,
        },
    );
    drop(store);

    let mut runtime = noname_runtime()
        .lock()
        .map_err(|_| "NoName observe-only: runtime lock poisoned".to_string())?;
    let result = runtime
        .run_director_observe_turn(
            NoNameTurnInput {
                trace_id: format!("noname-trace-{}", timestamp),
                session_id: "active-run".to_string(),
                turn_id: format!("turn-{}", timestamp),
                caller_role: NoNameRole::Director,
            },
            action_summary,
            &context_packet,
            Some(&NoNameDirectorGuardrailInput {
                plot_state: plot_state.clone(),
                action_result: plot_state.last_action_result.clone().unwrap_or(
                    crate::numerical_system::ActionResult {
                        success: true,
                        description: action_summary.to_string(),
                        stat_changes: Vec::new(),
                        events: Vec::new(),
                    },
                ),
                player_name: game_state.player.name.clone(),
                player_realm_level: game_state.player.stats.cultivation_realm.level,
                player_combat_power: game_state.player.stats.combat_power,
            }),
        )
        .map_err(|e| e.to_string())?;

    Ok(result)
}

fn build_plot_context_for_generation(
    game_state: &GameState,
    plot_state: &PlotState,
    action: &PlayerAction,
) -> Option<ContextBundle> {
    let mut recent_context_lines = plot_state
        .current_chapter
        .content
        .iter()
        .rev()
        .take(6)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>();
    recent_context_lines.push(format!(
        "战斗后状态: injury_level={}, reputation={}, enmity={}",
        game_state.player.combat_status.injury_level,
        game_state.player.combat_status.reputation,
        game_state.player.combat_status.enmity
    ));

    let input = ContextBuildInput {
        world_id: game_state.script.id.clone(),
        run_id: "active-run".to_string(),
        scene_id: plot_state.current_scene.id.clone(),
        character_ids: vec![game_state.player.id.clone()],
        map_node_id: Some(game_state.player.location.clone()),
        player_intent: if action.content.trim().is_empty() {
            None
        } else {
            Some(action.content.clone())
        },
        recent_context_lines,
        token_budget: 260,
    };

    let store = entity_store().lock().ok()?;
    let memory = memory_layers().lock().ok()?;
    Some(build_context_bundle(&store, &memory, &input))
}

fn render_generation_context(bundle: &ContextBundle) -> String {
    let mut lines = Vec::new();
    for line in bundle.hard_facts.iter().take(4) {
        lines.push(format!("事实: {}", line));
    }
    for line in bundle.recent_context.iter().take(3) {
        lines.push(format!("近文: {}", line));
    }
    for line in bundle.chapter_summaries.iter().take(2) {
        lines.push(format!("章节: {}", line));
    }
    for line in bundle.recent_events.iter().take(2) {
        lines.push(format!("事件: {}", line));
    }
    if lines.is_empty() {
        return String::new();
    }
    format!("\n\n[外置记忆+上下文窗口]\n{}", lines.join("\n"))
}

fn has_consistency_issue(report: &ConsistencyReport, code: &str) -> bool {
    report.issues.iter().any(|issue| issue.code == code)
}

fn diagnostics_used_preset_fallback(diag: Option<&str>) -> bool {
    let Some(text) = diag else {
        return false;
    };
    text.contains("已使用预设文本")
}

fn effective_consistency_report_after_option_resolution(
    mut report: ConsistencyReport,
    is_waiting_for_input: bool,
    chapter_end: bool,
    option_count: usize,
) -> ConsistencyReport {
    if is_waiting_for_input && !chapter_end && option_count > 0 {
        report
            .issues
            .retain(|issue| issue.code != "waiting_without_options");
    }
    report
}

fn player_options_from_choice_texts(texts: &[String]) -> Vec<PlayerOption> {
    texts
        .iter()
        .take(4)
        .enumerate()
        .map(|(idx, text)| PlayerOption {
            id: idx,
            description: text.clone(),
            requirements: vec![],
            action: Action::Custom {
                description: text.clone(),
            },
        })
        .collect()
}

fn collect_dead_character_names(registry: Option<&WorldRegistry>) -> HashSet<String> {
    let mut names = HashSet::new();
    let Some(registry) = registry else {
        return names;
    };
    for row in &registry.tables.characters {
        let Some(obj) = row.as_object() else {
            continue;
        };
        let name = obj
            .get("name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty());
        let Some(name) = name else {
            continue;
        };
        let dead_flag = obj.get("dead").and_then(Value::as_bool).unwrap_or(false)
            || obj.get("is_dead").and_then(Value::as_bool).unwrap_or(false)
            || obj
                .get("alive")
                .and_then(Value::as_bool)
                .map(|v| !v)
                .unwrap_or(false)
            || obj
                .get("is_alive")
                .and_then(Value::as_bool)
                .map(|v| !v)
                .unwrap_or(false)
            || obj
                .get("status")
                .and_then(Value::as_str)
                .map(|v| {
                    let lower = v.to_lowercase();
                    lower.contains("dead")
                        || lower.contains("deceased")
                        || v.contains("死亡")
                        || v.contains("阵亡")
                })
                .unwrap_or(false)
            || obj
                .get("life_status")
                .and_then(Value::as_str)
                .map(|v| {
                    let lower = v.to_lowercase();
                    lower.contains("dead")
                        || lower.contains("deceased")
                        || v.contains("死亡")
                        || v.contains("阵亡")
                })
                .unwrap_or(false);
        if dead_flag {
            names.insert(name.to_string());
        }
    }
    names
}

fn sanitize_generated_options(
    options: Vec<PlayerOption>,
    game_state: &GameState,
    current_scene_location: &str,
    registry: Option<&WorldRegistry>,
) -> (Vec<PlayerOption>, Vec<String>) {
    let mut notes = Vec::new();
    let mut sanitized = Vec::new();
    let dead_names = collect_dead_character_names(registry);
    let injured_heavy = game_state.player.combat_status.injury_level >= 7;
    let reachable_ids = compute_reachable_location_ids(game_state);
    let current_location_name = game_state
        .script
        .world_setting
        .locations
        .iter()
        .find(|loc| loc.id == current_scene_location)
        .map(|loc| loc.name.clone())
        .unwrap_or_else(|| current_scene_location.to_string());

    let mut location_name_to_id: HashMap<String, String> = HashMap::new();
    for loc in &game_state.script.world_setting.locations {
        location_name_to_id.insert(loc.name.clone(), loc.id.clone());
    }

    for mut option in options {
        let text = option.description.trim().to_string();
        if text.chars().count() < 2 {
            notes.push("option_repair:dropped_empty_option".to_string());
            continue;
        }

        let lower = text.to_lowercase();
        let asks_dialogue_with_dead = dead_names.iter().any(|name| {
            text.contains(name)
                && (text.contains("对话")
                    || text.contains("询问")
                    || text.contains("请教")
                    || text.contains("交谈")
                    || lower.contains("talk")
                    || lower.contains("speak"))
        });
        if asks_dialogue_with_dead {
            notes.push("option_repair:dropped_dead_character_dialogue".to_string());
            continue;
        }

        let talks_to_absent_character = registry
            .map(|r| {
                r.tables
                    .characters
                    .iter()
                    .filter_map(Value::as_object)
                    .any(|obj| {
                        let name = obj
                            .get("name")
                            .and_then(Value::as_str)
                            .map(str::trim)
                            .unwrap_or("");
                        if name.is_empty() || !text.contains(name) {
                            return false;
                        }
                        let location_id = obj
                            .get("location_id")
                            .and_then(Value::as_str)
                            .map(str::trim)
                            .unwrap_or("");
                        let asks_talk = text.contains("对话")
                            || text.contains("询问")
                            || text.contains("请教")
                            || text.contains("交谈")
                            || lower.contains("talk")
                            || lower.contains("speak");
                        asks_talk
                            && !location_id.is_empty()
                            && location_id != current_scene_location
                    })
            })
            .unwrap_or(false);
        if talks_to_absent_character {
            notes.push("option_repair:dropped_absent_character_dialogue".to_string());
            continue;
        }

        if injured_heavy
            && (text.contains("强攻") || text.contains("硬拼") || text.contains("搏命"))
        {
            option.description = "先稳住伤势与气机，再寻找可控破局点".to_string();
            option.action = Action::Custom {
                description: option.description.clone(),
            };
            notes.push("option_repair:rewrote_high_risk_option_due_to_injury".to_string());
        }

        let mut rewritten = false;
        for (loc_name, loc_id) in &location_name_to_id {
            if !text.contains(loc_name) {
                continue;
            }
            if loc_id == current_scene_location {
                continue;
            }
            if reachable_ids.iter().any(|id| id == loc_id) {
                continue;
            }
            option.description = format!(
                "先在{}附近探查线索，暂不贸然前往{}",
                current_location_name, loc_name
            );
            option.action = Action::Custom {
                description: option.description.clone(),
            };
            notes.push(format!(
                "option_repair:rewrote_unreachable_location({})",
                loc_name
            ));
            rewritten = true;
            break;
        }

        if !rewritten && option.description.trim().is_empty() {
            notes.push("option_repair:dropped_empty_after_rewrite".to_string());
            continue;
        }

        sanitized.push(option);
    }

    sanitized.truncate(4);
    for (idx, option) in sanitized.iter_mut().enumerate() {
        option.id = idx;
    }
    (sanitized, notes)
}

fn chapter_goal_regeneration_hint(interaction_count: u8) -> String {
    let goal = match interaction_count % 5 {
        0 => "冲突升级",
        1 => "角色成长",
        2 => "资源变化",
        3 => "关系变化",
        _ => "伏笔建立",
    };
    format!(
        "\n\n[章节目标重生成约束]\n本段必须明确命中：{}；禁止原地复述，必须出现可验证的新变化。",
        goal
    )
}

fn chapter_pacing_stage(interaction_count: u8) -> &'static str {
    match interaction_count % 4 {
        0 => "铺垫",
        1 => "冲突",
        2 => "转折",
        _ => "回落",
    }
}

fn narrative_segment_templates(stage: &str) -> Vec<&'static str> {
    match stage {
        "铺垫" => vec![
            "环境：风声与地形细节交代当前局势压力。",
            "动作：主角先做试探性动作，为后续冲突埋钩子。",
            "心理：写出动机与犹疑，不只报事件结果。",
        ],
        "冲突" => vec![
            "环境：声光与位移变化体现冲突升级。",
            "动作：至少一轮攻防转换，并带来可验证后果。",
            "心理：主角在风险下作出明确取舍。",
        ],
        "转折" => vec![
            "因果：上一段动作引发新变量（资源/关系/情报）。",
            "动作：给出打破僵局的新策略或代价。",
            "心理：执念或判断发生变化，推动下一步行动。",
        ],
        _ => vec![
            "环境：余波与现场状态交代局面收束。",
            "因果：明确本段收获/损失，形成下一段入口。",
            "心理：角色复盘本轮得失并调整目标。",
        ],
    }
}

fn narrative_dimension_coverage(text: &str) -> usize {
    let t = text.trim();
    if t.is_empty() {
        return 0;
    }
    let sensory_words = ["风", "雨", "雾", "声", "光", "影", "冷", "热", "血", "震"];
    let action_words = [
        "挥", "斩", "踏", "退", "冲", "挡", "运转", "催动", "闪", "击",
    ];
    let mental_words = [
        "犹豫", "执念", "惊", "怒", "惧", "定神", "思索", "决意", "迟疑", "判断",
    ];
    let mut covered = 0usize;
    if sensory_words.iter().any(|w| t.contains(w)) {
        covered += 1;
    }
    if action_words.iter().any(|w| t.contains(w)) {
        covered += 1;
    }
    if mental_words.iter().any(|w| t.contains(w)) {
        covered += 1;
    }
    covered
}

fn narrative_density_and_pacing_hint(interaction_count: u8) -> String {
    let stage = chapter_pacing_stage(interaction_count);
    let templates = narrative_segment_templates(stage);
    format!(
        "\n\n[叙事节奏与厚度约束]\n当前章节节奏阶段：{}；至少命中环境/动作/心理三类中的两类；必须显化角色内在状态（动机/犹疑/执念）并给出可验证因果；语言平实克制，避免空话和浮夸辞藻。\n可参考片段模板：\n- {}\n- {}\n- {}",
        stage, templates[0], templates[1], templates[2]
    )
}

const REGEN_LATENCY_BUDGET_MS: u128 = 2500;
const OPTION_LLM_LATENCY_BUDGET_MS: u128 = 3200;
const ACTION_PLOT_CACHE_TTL_SECS: u64 = 300;
const ACTION_PLOT_CACHE_MAX_ENTRIES: usize = 256;

#[derive(Debug, Clone)]
struct CachedPlotUpdate {
    update: crate::plot_engine::PlotUpdate,
    cached_at: u64,
}

fn action_plot_cache() -> &'static Mutex<HashMap<String, CachedPlotUpdate>> {
    static ACTION_PLOT_CACHE: OnceLock<Mutex<HashMap<String, CachedPlotUpdate>>> = OnceLock::new();
    ACTION_PLOT_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn normalize_action_bucket(action: &PlayerAction) -> String {
    let mut out = String::new();
    for ch in action.content.chars() {
        if ch.is_ascii_alphanumeric() || ch.is_alphabetic() || ch.is_numeric() {
            out.push(ch.to_ascii_lowercase());
        }
    }
    if out.is_empty() {
        return format!(
            "selected:{}",
            action
                .selected_option_id
                .map(|v| v.to_string())
                .unwrap_or_else(|| "none".to_string())
        );
    }
    out.chars().take(36).collect::<String>()
}

fn build_action_plot_cache_key(
    game_state: &GameState,
    plot_state: &PlotState,
    action: &PlayerAction,
) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}|{}",
        game_state.player.location,
        plot_state.current_chapter.index,
        plot_state.current_chapter.interaction_count,
        plot_state.segment_count,
        game_state.player.stats.cultivation_realm.level,
        action
            .selected_option_id
            .map(|v| v.to_string())
            .unwrap_or_else(|| "none".to_string()),
        normalize_action_bucket(action),
    )
}

fn get_cached_plot_update(key: &str) -> Option<crate::plot_engine::PlotUpdate> {
    let now = now_secs();
    let mut guard = match action_plot_cache().lock() {
        Ok(v) => v,
        Err(poisoned) => poisoned.into_inner(),
    };
    guard.retain(|_, entry| now.saturating_sub(entry.cached_at) <= ACTION_PLOT_CACHE_TTL_SECS);
    guard.get(key).map(|entry| entry.update.clone())
}

fn put_cached_plot_update(key: String, update: crate::plot_engine::PlotUpdate) {
    let now = now_secs();
    let mut guard = match action_plot_cache().lock() {
        Ok(v) => v,
        Err(poisoned) => poisoned.into_inner(),
    };
    guard.retain(|_, entry| now.saturating_sub(entry.cached_at) <= ACTION_PLOT_CACHE_TTL_SECS);
    if !guard.contains_key(&key) && guard.len() >= ACTION_PLOT_CACHE_MAX_ENTRIES {
        if let Some(oldest_key) = guard
            .iter()
            .min_by_key(|(_, entry)| entry.cached_at)
            .map(|(k, _)| k.clone())
        {
            guard.remove(&oldest_key);
        }
    }
    guard.insert(
        key,
        CachedPlotUpdate {
            update,
            cached_at: now,
        },
    );
}

fn hollow_expression_regeneration_hint() -> &'static str {
    "\n\n[叙事厚度重生成约束]\n禁止空洞套话与重复句式；至少补足环境、动作、心理三类中的两类，并给出可验证的因果变化；减少抽象抒情，优先具体事实。"
}

fn is_hollow_expression(text: &str) -> bool {
    let t = text.trim();
    if t.is_empty() {
        return true;
    }
    let filler_patterns = [
        "不由得",
        "一时间",
        "片刻之后",
        "气氛凝重",
        "心中一凛",
        "强大的气息",
        "隐隐作痛",
        "不禁",
        "仿佛",
        "似乎",
    ];
    let filler_hits = filler_patterns.iter().filter(|p| t.contains(**p)).count();

    let clauses = t
        .split(['。', '！', '？', ';', '；', '\n'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    let repeated_clause = if clauses.len() >= 3 {
        let unique = clauses
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        unique.len() * 10 <= clauses.len() * 7
    } else {
        false
    };

    let sensory_words = ["风", "雨", "雾", "声", "光", "影", "痛", "冷", "热", "血"];
    let action_words = ["挥", "斩", "踏", "退", "冲", "挡", "运转", "催动"];
    let mental_words = ["犹豫", "执念", "惊", "怒", "惧", "定神", "思索", "决意"];
    let mut covered = 0usize;
    if sensory_words.iter().any(|w| t.contains(w)) {
        covered += 1;
    }
    if action_words.iter().any(|w| t.contains(w)) {
        covered += 1;
    }
    if mental_words.iter().any(|w| t.contains(w)) {
        covered += 1;
    }

    (filler_hits >= 3 && t.chars().count() >= 30) || repeated_clause || covered <= 1
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct CombatExplanation {
    dominant_factors: Vec<String>,
    reversal_factors: Vec<String>,
    summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CombatStrategy {
    Cautious,
    Aggressive,
    Survival,
}

fn strategy_label(strategy: CombatStrategy) -> &'static str {
    match strategy {
        CombatStrategy::Cautious => "谨慎周旋",
        CombatStrategy::Aggressive => "强攻压制",
        CombatStrategy::Survival => "保命脱战",
    }
}

fn choose_combat_strategy(
    status: &crate::game_state::CombatAftermathStatus,
    option_hint: &str,
) -> (CombatStrategy, &'static str) {
    let hint = option_hint.to_lowercase();
    if status.injury_level >= 6 || status.qi_deviation >= 6 {
        return (CombatStrategy::Survival, "伤势/气机压力过高");
    }
    if hint.contains("保命") || hint.contains("撤") || hint.contains("脱战") {
        return (CombatStrategy::Survival, "玩家指令倾向保命");
    }
    if hint.contains("强攻")
        || hint.contains("硬拼")
        || hint.contains("冲锋")
        || hint.contains("搏命")
        || status.enmity >= 4
    {
        return (CombatStrategy::Aggressive, "高压进攻意图");
    }
    if hint.contains("试探")
        || hint.contains("防守")
        || hint.contains("周旋")
        || status.injury_level >= 3
    {
        return (CombatStrategy::Cautious, "以稳为主降低风险");
    }
    (CombatStrategy::Aggressive, "默认进攻策略")
}

fn strategy_power_modifier_pct(
    strategy: CombatStrategy,
    status: &crate::game_state::CombatAftermathStatus,
) -> i32 {
    match strategy {
        CombatStrategy::Aggressive => {
            if status.injury_level <= 2 && status.qi_deviation <= 3 {
                8
            } else {
                -6
            }
        }
        CombatStrategy::Cautious => {
            if status.injury_level >= 3 || status.qi_deviation >= 3 {
                5
            } else {
                -2
            }
        }
        CombatStrategy::Survival => -8,
    }
}

fn evaluate_environment_combat_modifier(game_state: &GameState) -> (i32, String) {
    let energy = location_spiritual_energy(game_state).unwrap_or(0.5);
    let status = &game_state.player.combat_status;
    let mut delta = 0i32;
    let mut reasons = Vec::new();

    if energy >= 0.8 {
        delta += 6;
        reasons.push(format!("高灵气场域增幅({:.2})", energy));
    } else if energy <= 0.25 {
        delta -= 6;
        reasons.push(format!("低灵气环境压制({:.2})", energy));
    } else if energy >= 0.55 {
        delta += 3;
        reasons.push(format!("中高灵气稳定输出({:.2})", energy));
    }

    if status.qi_deviation >= 6 {
        delta -= 5;
        reasons.push("气机紊乱削弱环境适应".to_string());
    }
    if status.injury_level >= 6 {
        delta -= 4;
        reasons.push("重伤状态难以利用地利".to_string());
    }

    let reason = if reasons.is_empty() {
        "环境影响中性".to_string()
    } else {
        reasons.join(" / ")
    };
    (delta.clamp(-20, 20), reason)
}

fn detect_player_styles(stats: &crate::models::CharacterStats) -> Vec<&'static str> {
    let mut styles = std::collections::BTreeSet::new();
    for tech in &stats.techniques {
        let t = tech.to_lowercase();
        if t.contains("剑") || t.contains("sword") {
            styles.insert("sword");
        }
        if t.contains("刀") || t.contains("blade") {
            styles.insert("blade");
        }
        if t.contains("拳") || t.contains("体") || t.contains("body") {
            styles.insert("body");
        }
        if t.contains("符") || t.contains("阵") || t.contains("talisman") || t.contains("array") {
            styles.insert("talisman");
        }
    }
    styles.into_iter().collect::<Vec<_>>()
}

fn extract_styles_from_text(hint: &str) -> Vec<&'static str> {
    let h = hint.to_lowercase();
    let mut styles = Vec::new();
    if h.contains("剑") || h.contains("sword") {
        styles.push("sword");
    }
    if h.contains("刀") || h.contains("blade") {
        styles.push("blade");
    }
    if h.contains("拳") || h.contains("体修") || h.contains("body") {
        styles.push("body");
    }
    if h.contains("符") || h.contains("阵") || h.contains("talisman") || h.contains("array") {
        styles.push("talisman");
    }
    styles.sort();
    styles.dedup();
    styles
}

fn detect_enemy_style(hint: &str) -> Option<&'static str> {
    let h = hint.to_lowercase();
    if h.contains("剑") || h.contains("sword") {
        return Some("sword");
    }
    if h.contains("刀") || h.contains("blade") {
        return Some("blade");
    }
    if h.contains("拳") || h.contains("体修") || h.contains("body") {
        return Some("body");
    }
    if h.contains("符") || h.contains("阵") || h.contains("talisman") || h.contains("array") {
        return Some("talisman");
    }
    None
}

fn style_counter_delta(player_style: &str, enemy_style: &str) -> i32 {
    crate::combat_style_rules::counter_delta(player_style, enemy_style)
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct CombatHardConstraintOutcome {
    accepted: bool,
    power_delta_pct: i32,
    reasons: Vec<String>,
}

#[allow(dead_code)]
fn infer_enemy_realm_level_from_hint(hint: &str) -> Option<u32> {
    let h = hint.to_lowercase();
    if h.contains("元婴") || h.contains("nascent soul") {
        return Some(4);
    }
    if h.contains("金丹") || h.contains("golden core") {
        return Some(3);
    }
    if h.contains("筑基") || h.contains("foundation") {
        return Some(2);
    }
    if h.contains("练气") || h.contains("qi") || h.contains("杂兵") {
        return Some(1);
    }
    None
}

#[allow(dead_code)]
fn evaluate_combat_hard_constraints(
    game_state: &GameState,
    combat_hint: &str,
) -> CombatHardConstraintOutcome {
    let mut accepted = true;
    let mut power_delta_pct = 0i32;
    let mut reasons = Vec::new();

    let player_realm = game_state.player.stats.cultivation_realm.level;
    if let Some(enemy_realm) = infer_enemy_realm_level_from_hint(combat_hint) {
        if enemy_realm >= player_realm + 2 {
            accepted = false;
            reasons.push(format!(
                "大境界压制：敌方境界 {} 显著高于我方 {}",
                enemy_realm, player_realm
            ));
        } else if enemy_realm > player_realm {
            power_delta_pct -= 12;
            reasons.push(format!(
                "境界差压制：敌方境界 {} 高于我方 {}",
                enemy_realm, player_realm
            ));
        }
    }

    let player_styles = detect_player_styles(&game_state.player.stats);
    let need_weapon = player_styles
        .iter()
        .any(|style| *style == "sword" || *style == "blade");
    let has_weapon = game_state.player.equipment_slots.weapon.is_some();
    if need_weapon && !has_weapon {
        power_delta_pct -= 14;
        reasons.push("兵器条件不足：剑/刀流派未装备武器".to_string());
    }

    let tendency = game_state.player.combat_tendency.as_str();
    if tendency == "survival" && combat_hint.contains("强攻") {
        power_delta_pct -= 6;
        reasons.push("战斗倾向与当前战术冲突：保命倾向下强攻受限".to_string());
    }

    CombatHardConstraintOutcome {
        accepted,
        power_delta_pct: power_delta_pct.clamp(-40, 20),
        reasons,
    }
}

fn evaluate_style_counter_modifier(
    stats: &crate::models::CharacterStats,
    combat_hint: &str,
) -> (i32, String) {
    let enemy_style = detect_enemy_style(combat_hint);
    let mut player_styles = detect_player_styles(stats);
    if player_styles.is_empty() {
        let hint_styles = extract_styles_from_text(combat_hint);
        if let Some(enemy) = enemy_style {
            if let Some(style) = hint_styles.iter().find(|s| **s != enemy) {
                player_styles.push(*style);
            }
        }
        if player_styles.is_empty() {
            if let Some(style) = hint_styles.first() {
                player_styles.push(*style);
            }
        }
    }
    let Some(enemy_style) = enemy_style else {
        return (0, "未识别敌方流派".to_string());
    };
    if player_styles.is_empty() {
        return (0, "我方流派未成型".to_string());
    }
    let mut best = 0;
    let mut best_style = "";
    for style in &player_styles {
        let d = style_counter_delta(style, enemy_style);
        if d > best {
            best = d;
            best_style = style;
        }
    }
    if best == 0 {
        return (
            0,
            format!(
                "未形成明确克制（我方={:?}, 敌方={}）",
                player_styles, enemy_style
            ),
        );
    }
    (
        best,
        format!("流派克制：{} 克制 {}", best_style, enemy_style),
    )
}

fn build_combat_explanation(
    action_result: &crate::numerical_system::ActionResult,
    player_realm_level: u32,
    player_combat_power: u64,
    numeric_guard_reason: Option<&str>,
    strategy_note: Option<&str>,
) -> CombatExplanation {
    let mut dominant_factors = vec![
        format!("境界层级: {}", player_realm_level),
        format!("战力基线: {}", player_combat_power),
    ];
    if action_result.success {
        dominant_factors.push("行动结果: 成功".to_string());
    } else {
        dominant_factors.push("行动结果: 受阻".to_string());
    }
    if let Some(note) = strategy_note {
        dominant_factors.push(format!("策略匹配: {}", note));
    }

    let mut reversal_factors = Vec::new();
    if let Some(reason) = numeric_guard_reason {
        reversal_factors.push(format!("数值守门裁决: {}", reason));
    }
    if !action_result.success {
        reversal_factors.push("执行环节出现失败分支".to_string());
    }
    if reversal_factors.is_empty() {
        reversal_factors.push("未触发显著反转因子".to_string());
    }

    let summary = format!(
        "主导因素={}；反转因素={}",
        dominant_factors.join(" / "),
        reversal_factors.join(" / ")
    );

    CombatExplanation {
        dominant_factors,
        reversal_factors,
        summary,
    }
}

fn apply_combat_aftermath(
    game_state: &mut GameState,
    combat_success: bool,
    strategy: Option<CombatStrategy>,
) -> String {
    let strategy = strategy.unwrap_or(CombatStrategy::Aggressive);
    let tendency = match strategy {
        CombatStrategy::Aggressive => "aggressive",
        CombatStrategy::Cautious => "cautious",
        CombatStrategy::Survival => "survival",
    };
    if game_state.player.set_combat_tendency(tendency) {
        push_growth_log(
            game_state,
            format!("战斗倾向更新：{}", game_state.player.combat_tendency),
        );
    }
    let status = &mut game_state.player.combat_status;
    if combat_success {
        status.reputation = status.reputation.saturating_add(2);
        status.enmity = status
            .enmity
            .saturating_add(if strategy == CombatStrategy::Aggressive {
                2
            } else {
                1
            });
        if status.injury_level > 0 && strategy != CombatStrategy::Aggressive {
            status.injury_level = status.injury_level.saturating_sub(1);
        }
        game_state.player.social_profile.favor =
            game_state.player.social_profile.favor.saturating_add(
                if strategy == CombatStrategy::Aggressive {
                    2
                } else {
                    1
                },
            );
        if strategy == CombatStrategy::Cautious {
            game_state.player.social_profile.mentor_bond = game_state
                .player
                .social_profile
                .mentor_bond
                .saturating_add(1);
        }
        game_state.player.social_profile.vendetta = game_state
            .player
            .social_profile
            .vendetta
            .saturating_sub(1)
            .max(0);
    } else {
        status.reputation = status.reputation.saturating_sub(1);
        status.enmity = status
            .enmity
            .saturating_add(if strategy == CombatStrategy::Survival {
                1
            } else {
                2
            });
        let injury_gain = if strategy == CombatStrategy::Survival {
            1
        } else {
            2
        };
        status.injury_level = status.injury_level.saturating_add(injury_gain).min(10);
        game_state.player.social_profile.vendetta =
            game_state.player.social_profile.vendetta.saturating_add(
                if strategy == CombatStrategy::Survival {
                    1
                } else {
                    2
                },
            );
        if strategy == CombatStrategy::Aggressive {
            game_state.player.social_profile.sect_affinity = game_state
                .player
                .social_profile
                .sect_affinity
                .saturating_sub(1);
        }
    }
    normalize_social_profile(&mut game_state.player.social_profile);

    format!(
        "战后状态更新：伤势={}, 威望={}, 仇恨={}",
        status.injury_level, status.reputation, status.enmity
    )
}

fn location_spiritual_energy(game_state: &GameState) -> Option<f32> {
    game_state
        .world_state
        .locations
        .get(&game_state.player.location)
        .map(|loc| loc.spiritual_energy)
}

fn cultivation_gain_multiplier_from_location(spiritual_energy: Option<f32>) -> f32 {
    let e = spiritual_energy.unwrap_or(0.5);
    (0.8 + e * 0.6).clamp(0.6, 1.6)
}

fn select_encounter_text(spiritual_energy: f32, seed: u64) -> &'static str {
    const LOW_RISK: [&str; 3] = [
        "途中遇到散修试探，你稳住阵脚后化解冲突。",
        "林间突发灵兽惊扰，短暂交锋后你安全脱身。",
        "遭遇路匪埋伏，但对方见势不妙迅速退去。",
    ];
    const MID_RISK: [&str; 3] = [
        "前路突现阵法余波，你强行破阵，气息略有紊乱。",
        "遭遇敌对门派巡逻，数轮试探后双方各自撤离。",
        "秘径中爆发灵压乱流，你顶住冲击后继续赶路。",
    ];
    const HIGH_RISK: [&str; 3] = [
        "深处魔息翻涌，突遭强敌伏击，激战后方得脱身。",
        "禁地边缘出现失控妖潮，你强行突围，代价不小。",
        "高阶修士威压横扫而过，你硬抗冲击后重整气机。",
    ];
    let bucket = if spiritual_energy >= 0.85 {
        &HIGH_RISK
    } else if spiritual_energy >= 0.5 {
        &MID_RISK
    } else {
        &LOW_RISK
    };
    let idx = (seed as usize) % bucket.len();
    bucket[idx]
}

fn push_growth_log(game_state: &mut GameState, entry: impl Into<String>) {
    let log = &mut game_state.player.growth_log;
    log.push(entry.into());
    const MAX_GROWTH_LOG: usize = 240;
    if log.len() > MAX_GROWTH_LOG {
        let overflow = log.len() - MAX_GROWTH_LOG;
        log.drain(0..overflow);
    }
}

fn normalize_social_profile(profile: &mut crate::game_state::SocialProfile) {
    profile.sect_affinity = profile.sect_affinity.clamp(-50, 50);
    profile.mentor_bond = profile.mentor_bond.clamp(-50, 50);
    profile.favor = profile.favor.clamp(-50, 50);
    profile.vendetta = profile.vendetta.clamp(0, 100);

    let stance_score =
        profile.sect_affinity + profile.mentor_bond + profile.favor - profile.vendetta;
    profile.camp_stance = if stance_score >= 12 {
        "righteous".to_string()
    } else if stance_score <= -8 {
        "demonic".to_string()
    } else {
        "neutral".to_string()
    };
}

fn apply_travel_and_encounter(
    game_state: &mut GameState,
    plot_state: &mut PlotState,
    target_location: &str,
) -> Result<(String, bool), String> {
    let target_node = game_state
        .world_state
        .locations
        .get(target_location)
        .cloned()
        .ok_or_else(|| format!("目标地点不存在: {}", target_location))?;
    let reachable_ids = compute_reachable_location_ids(game_state);
    if !reachable_ids.iter().any(|id| id == target_location) {
        return Err(format!(
            "当前状态下无法前往该地点: {}（请先降低伤势或分段行进）",
            target_location
        ));
    }
    let from_location = game_state.player.location.clone();
    if from_location == target_location {
        return Ok(("你已在当前地点，无需移动。".to_string(), false));
    }

    let from_name = game_state
        .world_state
        .locations
        .get(&from_location)
        .map(|loc| loc.name.clone())
        .unwrap_or_else(|| from_location.clone());
    let target_name = target_node.name.clone();
    let (mobility, max_energy) = compute_travel_capabilities(
        &game_state.player.stats.cultivation_realm,
        &game_state.player.combat_status,
    );
    let suggested_path = build_energy_path(game_state, target_location, mobility, max_energy)
        .unwrap_or_else(|| vec![from_location.clone(), target_location.to_string()]);
    let travel_days = suggested_path.len().saturating_sub(1).max(1) as u32;

    game_state.player.location = target_location.to_string();
    plot_state.current_scene.location = target_location.to_string();
    game_state.game_time.advance_days(travel_days);

    let total_days = game_state.game_time.total_days;
    let energy = target_node.spiritual_energy;
    let status = &game_state.player.combat_status;
    let cfg = crate::travel_rules::rules();
    let weighted_prob = (cfg.encounter_base_prob
        + (energy as f64 * cfg.encounter_energy_weight)
        + (status.enmity.max(0) as f64 * cfg.encounter_enmity_weight)
        + (status.qi_deviation as f64 * cfg.encounter_qi_weight))
        .clamp(cfg.encounter_prob_min, cfg.encounter_prob_max);
    let mut hash_acc = total_days as u64;
    for b in target_location.as_bytes() {
        hash_acc = hash_acc.wrapping_mul(131).wrapping_add(u64::from(*b));
    }
    let roll = (hash_acc % 100) as f64 / 100.0;
    let encounter_triggered = roll < weighted_prob;
    let mut message = format!(
        "你从{}前往{}，行程耗时{}日。",
        from_name, target_name, travel_days
    );
    if suggested_path.len() > 2 {
        message.push_str(&format!(" 建议分段：{}。", suggested_path.join(" -> ")));
    }

    if encounter_triggered {
        let status = &mut game_state.player.combat_status;
        let encounter_text = select_encounter_text(energy, hash_acc);
        status.enmity = status.enmity.saturating_add(1);
        game_state.player.social_profile.vendetta =
            game_state.player.social_profile.vendetta.saturating_add(1);
        if energy >= 0.8 {
            status.injury_level = status.injury_level.saturating_add(1).min(10);
            message.push(' ');
            message.push_str(encounter_text);
            message.push_str(" 伤势+1。");
        } else {
            message.push(' ');
            message.push_str(encounter_text);
            message.push_str(" 仇恨+1。");
        }
    } else {
        message.push_str(" 途中未遭遇显著冲突。");
        game_state.player.social_profile.favor =
            game_state.player.social_profile.favor.saturating_add(1);
    }
    if target_location.contains("sect") || target_name.contains("宗") {
        game_state.player.social_profile.sect_affinity = game_state
            .player
            .social_profile
            .sect_affinity
            .saturating_add(1);
    }
    normalize_social_profile(&mut game_state.player.social_profile);

    push_growth_log(
        game_state,
        format!(
            "行程变更：{} -> {}{}",
            from_name,
            target_name,
            if encounter_triggered {
                "（触发遭遇）"
            } else {
                ""
            }
        ),
    );
    plot_state.current_chapter.content.push(message.clone());
    Ok((message, encounter_triggered))
}

fn compute_reachable_location_ids(game_state: &GameState) -> Vec<String> {
    let cfg = crate::travel_rules::rules();
    let current_id = game_state.player.location.clone();
    let Some(current_node) = game_state.world_state.locations.get(&current_id) else {
        let mut all = game_state
            .world_state
            .locations
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        all.sort();
        return all;
    };
    let realm = &game_state.player.stats.cultivation_realm;
    let status = &game_state.player.combat_status;
    let (mobility, max_energy) = compute_travel_capabilities(realm, status);

    let mut by_gap = game_state
        .world_state
        .locations
        .values()
        .map(|loc| {
            let gap = (loc.spiritual_energy - current_node.spiritual_energy).abs();
            (loc.id.clone(), loc.spiritual_energy, gap)
        })
        .collect::<Vec<_>>();
    by_gap.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
    let nearest_two =
        if cfg.nearby_fallback_enabled && by_gap.len() >= cfg.nearby_fallback_min_location_count {
            by_gap
                .iter()
                .filter(|(id, _, _)| id != &current_id)
                .take(cfg.nearby_fallback_count)
                .map(|(id, _, _)| id.clone())
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

    let mut reachable = by_gap
        .into_iter()
        .filter(|(_, energy, gap)| *gap <= mobility && *energy <= max_energy + 0.05)
        .map(|(id, _, _)| id)
        .collect::<Vec<_>>();
    reachable.push(current_id);
    reachable.extend(nearest_two);
    reachable.sort();
    reachable.dedup();
    reachable
}

fn compute_travel_capabilities(
    realm: &crate::models::CultivationRealm,
    status: &crate::game_state::CombatAftermathStatus,
) -> (f32, f32) {
    let cfg = crate::travel_rules::rules();
    let mobility = (cfg.mobility_base
        + (realm.level as f32 * cfg.mobility_per_realm)
        + ((realm.sub_level.min(3)) as f32 * cfg.mobility_per_sub_level)
        - (status.injury_level as f32 * cfg.mobility_injury_penalty)
        - (status.qi_deviation as f32 * cfg.mobility_qi_penalty))
        .clamp(cfg.mobility_min, cfg.mobility_max);
    let max_energy = (cfg.max_energy_base + (realm.level as f32 * cfg.max_energy_per_realm)
        - (status.injury_level as f32 * cfg.max_energy_injury_penalty))
        .clamp(cfg.max_energy_min, cfg.max_energy_max);
    (mobility, max_energy)
}

fn build_energy_path(
    game_state: &GameState,
    target_id: &str,
    mobility: f32,
    max_energy: f32,
) -> Option<Vec<String>> {
    let current_id = game_state.player.location.clone();
    if current_id == target_id {
        return Some(vec![current_id]);
    }
    let current_energy = game_state
        .world_state
        .locations
        .get(&current_id)?
        .spiritual_energy;
    let target_energy = game_state
        .world_state
        .locations
        .get(target_id)?
        .spiritual_energy;
    if (current_energy - target_energy).abs() <= mobility && target_energy <= max_energy + 0.05 {
        return Some(vec![current_id, target_id.to_string()]);
    }

    let mut path = vec![current_id.clone()];
    let mut cursor_energy = current_energy;
    let mut visited = std::collections::BTreeSet::new();
    visited.insert(current_id.clone());

    for _ in 0..6 {
        let mut candidates = game_state
            .world_state
            .locations
            .values()
            .filter(|loc| !visited.contains(&loc.id))
            .filter(|loc| (loc.spiritual_energy - cursor_energy).abs() <= mobility)
            .filter(|loc| loc.spiritual_energy <= max_energy + 0.05)
            .collect::<Vec<_>>();
        candidates.sort_by(|a, b| {
            let da = (a.spiritual_energy - target_energy).abs();
            let db = (b.spiritual_energy - target_energy).abs();
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        });
        let next = candidates.first()?;
        path.push(next.id.clone());
        if next.id == target_id {
            return Some(path);
        }
        visited.insert(next.id.clone());
        cursor_energy = next.spiritual_energy;
    }
    None
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MapLocationOverview {
    pub location_id: String,
    pub name: String,
    pub spiritual_energy: f32,
    pub energy_gap: f32,
    pub reachable: bool,
    pub risk_tier: String,
    pub environment_tags: Vec<String>,
    pub resource_tags: Vec<String>,
    pub control_faction: String,
    pub event_hotspot: bool,
    pub estimated_steps: u32,
    pub suggested_path: Vec<String>,
}

fn infer_location_ecology(
    location: &crate::script::Location,
    world_setting: &crate::script::WorldSetting,
    risk_tier: &str,
) -> (Vec<String>, Vec<String>, String, bool) {
    let mut environment_tags = Vec::new();
    let mut resource_tags = Vec::new();
    let text = format!(
        "{} {} {}",
        location.id.to_lowercase(),
        location.name.to_lowercase(),
        location.description.to_lowercase()
    );

    if text.contains("宗") || text.contains("sect") {
        environment_tags.push("sect".to_string());
    }
    if text.contains("秘境") || text.contains("secret") || text.contains("realm") {
        environment_tags.push("secret_realm".to_string());
    }
    if text.contains("城")
        || text.contains("市")
        || text.contains("city")
        || text.contains("market")
    {
        environment_tags.push("town".to_string());
    }
    if text.contains("禁地") || text.contains("abyss") || text.contains("魔") {
        environment_tags.push("forbidden_zone".to_string());
    }
    if environment_tags.is_empty() {
        environment_tags.push("wilderness".to_string());
    }

    if location.spiritual_energy >= 0.75 {
        resource_tags.push("high_grade_ore".to_string());
        resource_tags.push("spirit_herb".to_string());
    } else if location.spiritual_energy >= 0.45 {
        resource_tags.push("spirit_herb".to_string());
        resource_tags.push("trade_material".to_string());
    } else {
        resource_tags.push("common_supply".to_string());
    }
    if environment_tags.iter().any(|tag| tag == "town") {
        resource_tags.push("market_goods".to_string());
    }
    resource_tags.sort();
    resource_tags.dedup();

    let mut control_faction = "unclaimed".to_string();
    for faction in &world_setting.factions {
        let f = faction.name.to_lowercase();
        if text.contains(&f) {
            control_faction = faction.id.clone();
            break;
        }
    }
    if control_faction == "unclaimed" && environment_tags.iter().any(|tag| tag == "sect") {
        control_faction = "sect_alliance".to_string();
    }

    let event_hotspot = risk_tier == "high"
        || (risk_tier == "medium" && location.spiritual_energy >= 0.65)
        || environment_tags
            .iter()
            .any(|tag| tag == "forbidden_zone" || tag == "secret_realm");

    (
        environment_tags,
        resource_tags,
        control_faction,
        event_hotspot,
    )
}

fn compute_map_overview(game_state: &GameState) -> Vec<MapLocationOverview> {
    let current_id = game_state.player.location.clone();
    let (mobility, max_energy) = compute_travel_capabilities(
        &game_state.player.stats.cultivation_realm,
        &game_state.player.combat_status,
    );
    let reachable = compute_reachable_location_ids(game_state)
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    let current_energy = game_state
        .world_state
        .locations
        .get(&current_id)
        .map(|loc| loc.spiritual_energy)
        .unwrap_or(0.0);

    let mut nodes = game_state
        .world_state
        .locations
        .values()
        .map(|loc| {
            let risk_tier = if loc.spiritual_energy >= 0.85 {
                "high"
            } else if loc.spiritual_energy >= 0.5 {
                "medium"
            } else {
                "low"
            };
            let (environment_tags, resource_tags, control_faction, event_hotspot) =
                infer_location_ecology(loc, &game_state.script.world_setting, risk_tier);
            let suggested_path =
                build_energy_path(game_state, &loc.id, mobility, max_energy).unwrap_or_default();
            let estimated_steps = suggested_path.len().saturating_sub(1) as u32;
            MapLocationOverview {
                location_id: loc.id.clone(),
                name: loc.name.clone(),
                spiritual_energy: loc.spiritual_energy,
                energy_gap: (loc.spiritual_energy - current_energy).abs(),
                reachable: reachable.contains(&loc.id),
                risk_tier: risk_tier.to_string(),
                environment_tags,
                resource_tags,
                control_faction,
                event_hotspot,
                estimated_steps,
                suggested_path,
            }
        })
        .collect::<Vec<_>>();
    nodes.sort_by(|a, b| {
        a.energy_gap
            .partial_cmp(&b.energy_gap)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    nodes
}

fn is_high_risk_technique_name(name: &str) -> bool {
    let t = name.to_lowercase();
    t.contains("禁")
        || t.contains("爆")
        || t.contains("噬")
        || t.contains("逆")
        || t.contains("魔")
        || t.contains("forbidden")
        || t.contains("berserk")
}

fn apply_breakthrough_failure_consequences(
    game_state: &mut GameState,
    action_result: &mut crate::numerical_system::ActionResult,
    high_risk_technique: bool,
) {
    let old_deviation = game_state.player.combat_status.qi_deviation;
    let old_injury = game_state.player.combat_status.injury_level;
    let deviation_gain = if high_risk_technique { 2 } else { 1 };
    let (new_deviation, new_injury) = {
        let status = &mut game_state.player.combat_status;
        status.qi_deviation = status.qi_deviation.saturating_add(deviation_gain).min(10);
        if high_risk_technique {
            status.injury_level = status.injury_level.saturating_add(1).min(10);
        }
        (status.qi_deviation, status.injury_level)
    };
    action_result.stat_changes.push(StatChange {
        stat_name: "qi_deviation".to_string(),
        old_value: old_deviation.to_string(),
        new_value: new_deviation.to_string(),
    });
    action_result.events.push(format!(
        "突破失败后果：气机紊乱+{}（当前 {}）",
        deviation_gain, new_deviation
    ));

    if high_risk_technique {
        action_result.stat_changes.push(StatChange {
            stat_name: "injury_level".to_string(),
            old_value: old_injury.to_string(),
            new_value: new_injury.to_string(),
        });
        action_result
            .events
            .push("高风险功法反噬：伤势+1，出现走火入魔征兆".to_string());
    }

    if new_deviation >= 7 {
        action_result
            .events
            .push("气机紊乱接近临界，需休整或降风险修炼".to_string());
    }

    action_result.description = format!(
        "{}（突破失败后果：气机紊乱+{}）",
        action_result.description, deviation_gain
    );
    push_growth_log(
        game_state,
        format!(
            "突破受挫：气机紊乱 {} -> {}{}",
            old_deviation,
            new_deviation,
            if high_risk_technique {
                "；触发高风险反噬"
            } else {
                ""
            }
        ),
    );
}

fn apply_cultivation_side_effects(
    game_state: &mut GameState,
    action_result: &mut crate::numerical_system::ActionResult,
) -> Option<String> {
    if game_state.player.stats.techniques.is_empty() {
        return None;
    }
    let (_, _, high_risk_technique) =
        evaluate_technique_semantic_modifier(&game_state.player.stats);
    let seed = game_state.game_time.total_days as usize + game_state.player.stats.techniques.len();

    if high_risk_technique && seed.is_multiple_of(2) {
        let old_qi = game_state.player.combat_status.qi_deviation;
        let old_injury = game_state.player.combat_status.injury_level;
        game_state.player.combat_status.qi_deviation = game_state
            .player
            .combat_status
            .qi_deviation
            .saturating_add(1)
            .min(10);
        if game_state.player.combat_status.qi_deviation >= 7 {
            game_state.player.combat_status.injury_level = game_state
                .player
                .combat_status
                .injury_level
                .saturating_add(1)
                .min(10);
        }
        action_result.stat_changes.push(StatChange {
            stat_name: "qi_deviation".to_string(),
            old_value: old_qi.to_string(),
            new_value: game_state.player.combat_status.qi_deviation.to_string(),
        });
        if old_injury != game_state.player.combat_status.injury_level {
            action_result.stat_changes.push(StatChange {
                stat_name: "injury_level".to_string(),
                old_value: old_injury.to_string(),
                new_value: game_state.player.combat_status.injury_level.to_string(),
            });
        }
        action_result
            .events
            .push("修炼反噬：高风险功法导致气机紊乱上升".to_string());
        push_growth_log(
            game_state,
            format!(
                "修炼反噬：气机紊乱 {} -> {}",
                old_qi, game_state.player.combat_status.qi_deviation
            ),
        );
        return Some("触发高风险功法反噬".to_string());
    }

    if game_state.player.stats.spiritual_root.affinity >= 0.75
        && game_state.player.combat_status.qi_deviation <= 3
        && seed.is_multiple_of(3)
        && game_state.player.stats.cultivation_realm.sub_level < 3
    {
        let old_sub = game_state.player.stats.cultivation_realm.sub_level;
        game_state.player.stats.cultivation_realm.sub_level += 1;
        game_state.player.stats.cultivation_realm.power_multiplier *= 1.08;
        game_state.player.stats.update_combat_power();
        let new_sub = game_state.player.stats.cultivation_realm.sub_level;
        action_result.stat_changes.push(StatChange {
            stat_name: "realm_sub_level".to_string(),
            old_value: old_sub.to_string(),
            new_value: new_sub.to_string(),
        });
        action_result
            .events
            .push("修炼顿悟：境界感悟提升".to_string());
        push_growth_log(
            game_state,
            format!("修炼顿悟：小境界 {} -> {}", old_sub, new_sub),
        );
        return Some("触发顿悟，小境界提升".to_string());
    }

    None
}

fn breakthrough_blocked_by_qi_deviation(qi_deviation: u8) -> bool {
    qi_deviation >= 8
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TechniqueSemanticProfile {
    technique_name: String,
    type_tags: Vec<String>,
    trait_tags: Vec<String>,
    condition_tags: Vec<String>,
    counter_tags: Vec<String>,
    risk_level: i32,
    required_realm_level: u32,
}

fn infer_technique_semantics(name: &str) -> TechniqueSemanticProfile {
    let lower = name.to_lowercase();
    let mut type_tags = Vec::new();
    let mut trait_tags = Vec::new();
    let mut condition_tags = Vec::new();
    let mut counter_tags = Vec::new();
    let mut risk_level = 0;
    let mut required_realm_level = 1u32;

    if lower.contains("剑") || lower.contains("sword") {
        type_tags.push("sword".to_string());
        counter_tags.push("body".to_string());
    }
    if lower.contains("刀") || lower.contains("blade") {
        type_tags.push("blade".to_string());
        counter_tags.push("talisman".to_string());
    }
    if lower.contains("拳") || lower.contains("体") || lower.contains("body") {
        type_tags.push("body".to_string());
        counter_tags.push("sword".to_string());
    }
    if lower.contains("符")
        || lower.contains("阵")
        || lower.contains("talisman")
        || lower.contains("array")
    {
        type_tags.push("talisman".to_string());
        counter_tags.push("blade".to_string());
    }
    if lower.contains("fire")
        || lower.contains("炎")
        || lower.contains("焰")
        || lower.contains("火")
    {
        trait_tags.push("fire".to_string());
    }
    if lower.contains("ice") || lower.contains("寒") || lower.contains("冰") {
        trait_tags.push("ice".to_string());
    }
    if lower.contains("water") || lower.contains("水") {
        trait_tags.push("water".to_string());
    }
    if lower.contains("thunder") || lower.contains("雷") {
        trait_tags.push("thunder".to_string());
    }
    if lower.contains("元婴") || lower.contains("nascent") {
        required_realm_level = 4;
        condition_tags.push("realm>=4".to_string());
    } else if lower.contains("金丹") || lower.contains("golden core") {
        required_realm_level = 3;
        condition_tags.push("realm>=3".to_string());
    } else if lower.contains("筑基") || lower.contains("foundation") {
        required_realm_level = 2;
        condition_tags.push("realm>=2".to_string());
    }
    if is_high_risk_technique_name(name) {
        risk_level += 2;
        trait_tags.push("high_risk".to_string());
        condition_tags.push("qi_stable_required".to_string());
    }

    type_tags.sort();
    type_tags.dedup();
    trait_tags.sort();
    trait_tags.dedup();
    condition_tags.sort();
    condition_tags.dedup();
    counter_tags.sort();
    counter_tags.dedup();

    TechniqueSemanticProfile {
        technique_name: name.to_string(),
        type_tags,
        trait_tags,
        condition_tags,
        counter_tags,
        risk_level,
        required_realm_level,
    }
}

fn evaluate_technique_semantic_modifier(
    stats: &crate::models::CharacterStats,
) -> (i32, Vec<String>, bool) {
    let mut percent_delta = 0i32;
    let mut reasons = Vec::new();
    let roots = stats
        .spiritual_root
        .effective_elements()
        .iter()
        .map(|e| format!("{:?}", e).to_lowercase())
        .collect::<Vec<_>>();
    let mut has_high_risk = false;

    for tech in &stats.techniques {
        let semantic = infer_technique_semantics(tech);
        if semantic.trait_tags.iter().any(|tag| tag == "fire") && roots.iter().any(|r| r == "fire")
        {
            percent_delta += 8;
            reasons.push(format!("功法 `{}` 与火灵根适配", semantic.technique_name));
        }
        if semantic.trait_tags.iter().any(|tag| tag == "ice")
            && (roots.iter().any(|r| r == "water") || roots.iter().any(|r| r == "ice"))
        {
            percent_delta += 6;
            reasons.push(format!(
                "功法 `{}` 与水/冰灵根适配",
                semantic.technique_name
            ));
        }
        if semantic.trait_tags.iter().any(|tag| tag == "water")
            && roots.iter().any(|r| r == "water")
        {
            percent_delta += 5;
            reasons.push(format!("功法 `{}` 与水灵根适配", semantic.technique_name));
        }
        if stats.cultivation_realm.level < semantic.required_realm_level {
            percent_delta -= 10;
            reasons.push(format!(
                "功法 `{}` 境界门槛偏高（需 {}），发挥受限",
                semantic.technique_name, semantic.required_realm_level
            ));
        }
        if semantic.risk_level > 0 {
            has_high_risk = true;
            percent_delta += 5;
            reasons.push(format!(
                "功法 `{}` 高风险强行驱动，短时增益",
                semantic.technique_name
            ));
        }
    }

    (percent_delta.clamp(-30, 30), reasons, has_high_risk)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GenerationTimingSample {
    total_ms: u64,
    plot_gen_ms: u64,
    option_gen_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GenerationTimingSummary {
    pub sample_count: usize,
    pub total_p50_ms: u64,
    pub total_p95_ms: u64,
    pub total_p99_ms: u64,
    pub plot_gen_p95_ms: u64,
    pub option_gen_p95_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GenerationFailureReason {
    pub stage: String,
    pub reason: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GenerationFailureSummary {
    pub sample_count: usize,
    pub structured_ok_count: usize,
    pub plain_ok_count: usize,
    pub skeleton_ok_count: usize,
    pub micro_ok_count: usize,
    pub preset_fallback_count: usize,
    pub turn_update_fallback_count: usize,
    pub option_llm_blocked_count: usize,
    pub top_reasons: Vec<GenerationFailureReason>,
}

fn percentile_u64(samples: &[u64], p: f64) -> u64 {
    if samples.is_empty() {
        return 0;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn parse_generation_timing_sample(diag: &str) -> Option<GenerationTimingSample> {
    let marker = "耗时(ms)：";
    let payload = diag.split(marker).last()?.trim();
    let mut total_ms = None;
    let mut plot_gen_ms = None;
    let mut option_gen_ms = None;

    for part in payload.split([',', '，', ';', '；']) {
        let mut iter = part.trim().splitn(2, '=');
        let key = iter.next()?.trim();
        let value = iter.next()?.trim().parse::<u64>().ok()?;
        match key {
            "total" => total_ms = Some(value),
            "plot_gen" => plot_gen_ms = Some(value),
            "option_gen" => option_gen_ms = Some(value),
            _ => {}
        }
    }

    Some(GenerationTimingSample {
        total_ms: total_ms?,
        plot_gen_ms: plot_gen_ms?,
        option_gen_ms: option_gen_ms?,
    })
}

fn summarize_generation_timing_diagnostics(
    diagnostics: &[String],
) -> Option<GenerationTimingSummary> {
    let samples = diagnostics
        .iter()
        .filter_map(|diag| parse_generation_timing_sample(diag))
        .collect::<Vec<_>>();
    if samples.is_empty() {
        return None;
    }

    let total = samples.iter().map(|s| s.total_ms).collect::<Vec<_>>();
    let plot = samples.iter().map(|s| s.plot_gen_ms).collect::<Vec<_>>();
    let option = samples.iter().map(|s| s.option_gen_ms).collect::<Vec<_>>();

    Some(GenerationTimingSummary {
        sample_count: samples.len(),
        total_p50_ms: percentile_u64(&total, 0.50),
        total_p95_ms: percentile_u64(&total, 0.95),
        total_p99_ms: percentile_u64(&total, 0.99),
        plot_gen_p95_ms: percentile_u64(&plot, 0.95),
        option_gen_p95_ms: percentile_u64(&option, 0.95),
    })
}

fn extract_turn_update_fallback_reason(diag: &str) -> Option<String> {
    let marker = "双通道生成：fallback(plot_engine_only";
    let start = diag.find(marker)?;
    let tail = &diag[start..];
    let reason_marker = "reason=";
    let reason_start = tail.find(reason_marker)?;
    let reason_tail = &tail[(reason_start + reason_marker.len())..];
    let end_idx = reason_tail
        .find(')')
        .or_else(|| reason_tail.find('；'))
        .unwrap_or(reason_tail.len());
    let reason = reason_tail[..end_idx].trim();
    if reason.is_empty() {
        None
    } else {
        Some(reason.to_string())
    }
}

fn extract_preset_fallback_reason(diag: &str) -> Option<String> {
    let marker = "回退：";
    let start = diag.find(marker)?;
    let tail = &diag[(start + marker.len())..];
    let end_marker = "；纯文本续写也失败，已使用预设文本";
    let end = tail.find(end_marker)?;
    let reason = tail[..end].trim();
    if reason.is_empty() {
        None
    } else {
        Some(reason.to_string())
    }
}

fn summarize_generation_failure_diagnostics(
    diagnostics: &[String],
) -> Option<GenerationFailureSummary> {
    if diagnostics.is_empty() {
        return None;
    }

    let mut structured_ok_count = 0usize;
    let mut plain_ok_count = 0usize;
    let mut skeleton_ok_count = 0usize;
    let mut micro_ok_count = 0usize;
    let mut preset_fallback_count = 0usize;
    let mut turn_update_fallback_count = 0usize;
    let mut option_llm_blocked_count = 0usize;
    let mut reason_counter: HashMap<(String, String), usize> = HashMap::new();

    for diag in diagnostics {
        if diag.contains("链路：structured_ok") {
            structured_ok_count += 1;
        }
        if diag.contains("链路：plain_ok") {
            plain_ok_count += 1;
        }
        if diag.contains("链路：skeleton_ok") {
            skeleton_ok_count += 1;
        }
        if diag.contains("链路：micro_ok") {
            micro_ok_count += 1;
        }
        if diag.contains("链路：preset_fallback") {
            preset_fallback_count += 1;
            if let Some(reason) = extract_preset_fallback_reason(diag) {
                *reason_counter
                    .entry(("preset_fallback".to_string(), reason))
                    .or_insert(0) += 1;
            }
        }
        if diag.contains("双通道生成：fallback(plot_engine_only") {
            turn_update_fallback_count += 1;
            if let Some(reason) = extract_turn_update_fallback_reason(diag) {
                *reason_counter
                    .entry(("turn_update".to_string(), reason))
                    .or_insert(0) += 1;
            }
        }
        if diag.contains("本轮为选项续写：未获得可用 LLM 剧情文本")
            || diag.contains("本次选项续写未获取到 LLM 剧情文本")
        {
            option_llm_blocked_count += 1;
        }
    }

    let mut top_reasons = reason_counter
        .into_iter()
        .map(|((stage, reason), count)| GenerationFailureReason {
            stage,
            reason,
            count,
        })
        .collect::<Vec<_>>();
    top_reasons.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| a.stage.cmp(&b.stage))
            .then_with(|| a.reason.cmp(&b.reason))
    });
    top_reasons.truncate(8);

    Some(GenerationFailureSummary {
        sample_count: diagnostics.len(),
        structured_ok_count,
        plain_ok_count,
        skeleton_ok_count,
        micro_ok_count,
        preset_fallback_count,
        turn_update_fallback_count,
        option_llm_blocked_count,
        top_reasons,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl From<anyhow::Error> for ErrorResponse {
    fn from(err: anyhow::Error) -> Self {
        ErrorResponse {
            error: err.to_string(),
        }
    }
}

fn map_error(context: &str, err: impl Into<AppError>) -> String {
    err.into().with_context(context).to_string()
}

fn validate_slot_id(slot_id: u32) -> Result<(), AppError> {
    if (1..=99).contains(&slot_id) {
        Ok(())
    } else {
        Err(AppError::new(
            crate::app_error::AppErrorKind::InvalidInput,
            format!("存档槽位必须在 1-99 之间，当前为 {}", slot_id),
        ))
    }
}

fn validate_file_path(path: &str, allowed_exts: &[&str]) -> Result<(), AppError> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(AppError::new(
            crate::app_error::AppErrorKind::NotFound,
            format!("文件不存在: {}", path),
        ));
    }
    if !p.is_file() {
        return Err(AppError::new(
            crate::app_error::AppErrorKind::InvalidInput,
            format!("路径不是文件: {}", path),
        ));
    }
    if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
        if allowed_exts
            .iter()
            .any(|allowed| ext.eq_ignore_ascii_case(allowed))
        {
            return Ok(());
        }
    }
    Err(AppError::new(
        crate::app_error::AppErrorKind::InvalidInput,
        format!("文件格式不支持: {}", path),
    ))
}

fn validate_endpoint(endpoint: &str) -> Result<(), AppError> {
    let trimmed = endpoint.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        Ok(())
    } else {
        Err(AppError::new(
            crate::app_error::AppErrorKind::InvalidInput,
            "LLM endpoint 必须以 http:// 或 https:// 开头",
        ))
    }
}

fn validate_non_empty(value: &str, label: &str) -> Result<(), AppError> {
    if value.trim().is_empty() {
        Err(AppError::new(
            crate::app_error::AppErrorKind::InvalidInput,
            format!("{}不能为空", label),
        ))
    } else {
        Ok(())
    }
}

fn validate_output_path(path: &str, allowed_exts: &[&str]) -> Result<(), AppError> {
    let p = Path::new(path);
    if let Some(parent) = p.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            return Err(AppError::new(
                crate::app_error::AppErrorKind::NotFound,
                format!("输出目录不存在: {}", parent.display()),
            ));
        }
    }
    if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
        if allowed_exts
            .iter()
            .any(|allowed| ext.eq_ignore_ascii_case(allowed))
        {
            return Ok(());
        }
    }
    Err(AppError::new(
        crate::app_error::AppErrorKind::InvalidInput,
        format!("输出文件格式不支持: {}", path),
    ))
}

fn validate_llm_config_input(input: &LLMConfigInput) -> Result<(), AppError> {
    validate_endpoint(&input.endpoint)?;
    validate_non_empty(&input.model, "模型ID(model)")?;
    if input.max_tokens == 0 || input.max_tokens > 8192 {
        return Err(AppError::new(
            crate::app_error::AppErrorKind::InvalidInput,
            "max_tokens 必须在 1-8192 之间",
        ));
    }
    if !(0.0..=2.0).contains(&input.temperature) {
        return Err(AppError::new(
            crate::app_error::AppErrorKind::InvalidInput,
            "temperature 必须在 0-2 之间",
        ));
    }
    if input.api_key.trim().is_empty() {
        let endpoint = input.endpoint.to_lowercase();
        let local = endpoint.contains("localhost") || endpoint.contains("127.0.0.1");
        if !local {
            return Err(AppError::new(
                crate::app_error::AppErrorKind::InvalidInput,
                "API Key 不能为空",
            ));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LLMConfigInput {
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

fn merge_llm_config_input_with_existing(
    input: &LLMConfigInput,
    existing: Option<LLMConfig>,
) -> LLMConfigInput {
    let existing_api_key = existing.map(|cfg| cfg.api_key).unwrap_or_default();

    LLMConfigInput {
        endpoint: input.endpoint.clone(),
        api_key: if input.api_key.trim().is_empty() {
            existing_api_key
        } else {
            input.api_key.clone()
        },
        model: input.model.clone(),
        max_tokens: input.max_tokens,
        temperature: input.temperature,
    }
}

#[tauri::command]
pub async fn set_llm_config(input: LLMConfigInput) -> Result<String, String> {
    let merged_input = merge_llm_config_input_with_existing(&input, resolve_llm_config());
    validate_llm_config_input(&merged_input).map_err(|e| map_error("LLM 配置校验失败", e))?;
    let config = LLMConfig {
        endpoint: merged_input.endpoint,
        api_key: merged_input.api_key,
        model: merged_input.model,
        max_tokens: merged_input.max_tokens,
        temperature: merged_input.temperature,
    };
    LLMService::new(config.clone()).map_err(|e| map_error("LLM 配置校验失败", e))?;
    set_runtime_llm_config(config);
    Ok("LLM 配置已更新".to_string())
}

#[tauri::command]
pub async fn clear_llm_config() -> Result<String, String> {
    clear_runtime_llm_config();
    Ok("已清除运行时 LLM 配置".to_string())
}

#[tauri::command]
pub async fn get_llm_config_status() -> Result<LLMConfigStatus, String> {
    Ok(runtime_llm_config_status())
}

#[tauri::command]
pub async fn test_llm_connection() -> Result<String, String> {
    let cfg = resolve_llm_config().ok_or_else(|| "未检测到 LLM 配置".to_string())?;
    let service = LLMService::new(cfg).map_err(|e| e.to_string())?;
    let response = service
        .generate(LLMRequest {
            prompt: "请回复：连接成功".to_string(),
            max_tokens: Some(32),
            temperature: Some(0.1),
        })
        .await
        .map_err(|e| e.to_string())?;
    Ok(response.text)
}

#[tauri::command]
pub async fn initialize_game(
    script: Script,
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<GameState, String> {
    const INIT_WORLD_REGISTRY_BOOTSTRAP_TIMEOUT_SECS: u64 = 20;
    let init_started = Instant::now();
    let mut game_state = {
        let mut engine = match engine.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        engine.initialize_game(script).map_err(|e| e.to_string())?
    };
    let input_assembly_ms = init_started.elapsed().as_millis();
    let bootstrap_started = Instant::now();
    let mut registry = match tokio::time::timeout(
        Duration::from_secs(INIT_WORLD_REGISTRY_BOOTSTRAP_TIMEOUT_SECS),
        WorldRegistry::bootstrap_with_llm(
            &game_state,
            WORLD_REGISTRY_REFERENCE,
            WORLD_REGISTRY_SUPPLEMENT,
        ),
    )
    .await
    {
        Ok(Some(registry)) => registry,
        Ok(None) => WorldRegistry::fallback_from_game_state(&game_state, "bootstrap_fallback"),
        Err(_) => {
            WorldRegistry::fallback_from_game_state(&game_state, "bootstrap_timeout_fallback")
        }
    };
    let model_request_parse_ms = bootstrap_started.elapsed().as_millis();
    let apply_started = Instant::now();
    apply_registry_to_game_state(&mut game_state, &registry);
    let parse_apply_ms = apply_started.elapsed().as_millis();
    let total_ms = init_started.elapsed().as_millis();
    registry.tables.world_facts.push(json!({
        "fact_id": format!("diag-init-{}", now_secs()),
        "subject": "runtime",
        "predicate": "init_timing",
        "object": format!(
            "初始化耗时(ms)：total={},input_assembly={},model_request_parse={},parse_apply={}",
            total_ms, input_assembly_ms, model_request_parse_ms, parse_apply_ms
        ),
        "confidence": 1.0
    }));
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    engine
        .update_current_state(game_state.clone())
        .map_err(|e| e.to_string())?;
    engine.update_world_registry(registry);
    Ok(game_state)
}

#[tauri::command]
pub async fn execute_player_action(
    action: PlayerAction,
    quick_mode: Option<bool>,
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<String, String> {
    let quick_mode = quick_mode.unwrap_or(false);
    let total_started = Instant::now();
    let (mut game_state, mut plot_state, mut world_registry) = {
        let engine = match engine.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let game_state = engine.get_current_state().map_err(|e| e.to_string())?;
        let plot_state = engine.get_plot_state().map_err(|e| e.to_string())?;
        let world_registry = engine.get_world_registry().ok();
        (game_state, plot_state, world_registry)
    };
    plot_state.interaction_state = PlotInteractionState::Resolving;
    let noname_action_summary = build_noname_action_summary(&action, &plot_state);

    let plot_engine = PlotEngine::new();
    let context = Context {
        location: game_state.player.location.clone(),
        time_of_day: "day".to_string(),
        weather: None,
    };

    let mut action_result = plot_engine
        .process_player_action(
            &action,
            &game_state.player.stats,
            &plot_state.current_scene.available_options,
            &context,
        )
        .map_err(|e| e.to_string())?;
    let selected_option_requires_llm_story = action.selected_option_id.is_some();

    let mut combat_guard_reason: Option<String> = None;
    let mut combat_strategy_note: Option<String> = None;
    let mut should_emit_combat_explanation = false;
    if let Some(selected_option_id) = action.selected_option_id {
        if let Some(selected_option) = plot_state
            .current_scene
            .available_options
            .get(selected_option_id)
        {
            match &selected_option.action {
                Action::Cultivate => {
                    let old_power = game_state.player.stats.combat_power;
                    let energy = location_spiritual_energy(&game_state);
                    let gain_multiplier = cultivation_gain_multiplier_from_location(energy);
                    let gain = ((old_power as f32 * 0.03 * gain_multiplier).round() as u64).max(1);
                    let new_power = old_power.saturating_add(gain);
                    game_state.player.stats.combat_power = new_power;
                    action_result.stat_changes.push(StatChange {
                        stat_name: "combat_power".to_string(),
                        old_value: old_power.to_string(),
                        new_value: new_power.to_string(),
                    });
                    action_result.description = format!(
                        "{} 战力提升了 {}（区域灵气修正 {:.2}x）。",
                        action_result.description, gain, gain_multiplier
                    );
                    if let Some(effect_note) =
                        apply_cultivation_side_effects(&mut game_state, &mut action_result)
                    {
                        action_result.description =
                            format!("{}（{}）", action_result.description, effect_note);
                    }
                    push_growth_log(
                        &mut game_state,
                        format!(
                            "修炼成长：战力 {} -> {}（灵气 {:.2}）",
                            old_power,
                            new_power,
                            energy.unwrap_or(0.5)
                        ),
                    );
                }
                Action::Breakthrough => {
                    let high_risk_technique = game_state
                        .player
                        .stats
                        .techniques
                        .iter()
                        .any(|t| is_high_risk_technique_name(t));
                    if breakthrough_blocked_by_qi_deviation(
                        game_state.player.combat_status.qi_deviation,
                    ) {
                        action_result.success = false;
                        action_result
                            .events
                            .push("突破被中断：气机紊乱接近失控".to_string());
                        action_result.description = format!(
                            "{}（气机紊乱过高，强行突破失败）",
                            action_result.description
                        );
                        apply_breakthrough_failure_consequences(
                            &mut game_state,
                            &mut action_result,
                            high_risk_technique,
                        );
                    } else if action_result.success
                        && game_state.player.stats.cultivation_realm.sub_level < 3
                    {
                        let old_sub = game_state.player.stats.cultivation_realm.sub_level;
                        game_state.player.stats.cultivation_realm.sub_level += 1;
                        game_state.player.stats.cultivation_realm.power_multiplier *= 1.2;
                        game_state.player.stats.update_combat_power();
                        action_result.stat_changes.push(StatChange {
                            stat_name: "realm_sub_level".to_string(),
                            old_value: old_sub.to_string(),
                            new_value: game_state
                                .player
                                .stats
                                .cultivation_realm
                                .sub_level
                                .to_string(),
                        });
                        let realm_name = game_state.player.stats.cultivation_realm.name.clone();
                        let new_sub = game_state.player.stats.cultivation_realm.sub_level;
                        let growth_entry =
                            format!("突破成长：{} 小境界 {} -> {}", realm_name, old_sub, new_sub);
                        push_growth_log(&mut game_state, growth_entry);
                    } else if !action_result.success {
                        apply_breakthrough_failure_consequences(
                            &mut game_state,
                            &mut action_result,
                            high_risk_technique,
                        );
                    }
                }
                Action::Combat { .. } => {
                    should_emit_combat_explanation = true;
                    let option_hint = format!("{} {}", selected_option.description, action.content);
                    let hard_constraints =
                        evaluate_combat_hard_constraints(&game_state, &option_hint);
                    if hard_constraints.power_delta_pct != 0 {
                        let old_power = game_state.player.stats.combat_power;
                        let scaled = (old_power as i128)
                            .saturating_mul((100 + hard_constraints.power_delta_pct) as i128)
                            / 100i128;
                        let new_power = scaled.max(1) as u64;
                        game_state.player.stats.combat_power = new_power;
                        action_result.stat_changes.push(StatChange {
                            stat_name: "combat_power".to_string(),
                            old_value: old_power.to_string(),
                            new_value: new_power.to_string(),
                        });
                        action_result.description = format!(
                            "{}（硬约束修正: {}%）",
                            action_result.description, hard_constraints.power_delta_pct
                        );
                        push_growth_log(
                            &mut game_state,
                            format!("战斗硬约束影响：战力 {} -> {}", old_power, new_power),
                        );
                    }
                    if !hard_constraints.reasons.is_empty() {
                        action_result.description = format!(
                            "{}；硬约束：{}",
                            action_result.description,
                            hard_constraints.reasons.join(" / ")
                        );
                    }
                    if !hard_constraints.accepted {
                        action_result.success = false;
                        let reason = hard_constraints
                            .reasons
                            .first()
                            .cloned()
                            .unwrap_or_else(|| "硬约束未通过".to_string());
                        combat_guard_reason = Some(reason.clone());
                        action_result
                            .events
                            .push(format!("战斗裁决中断：{}", reason));
                    }
                    let (strategy, strategy_reason) =
                        choose_combat_strategy(&game_state.player.combat_status, &option_hint);
                    let strategy_delta_pct =
                        strategy_power_modifier_pct(strategy, &game_state.player.combat_status);
                    if strategy_delta_pct != 0 && hard_constraints.accepted {
                        let old_power = game_state.player.stats.combat_power;
                        let scaled = (old_power as i128)
                            .saturating_mul((100 + strategy_delta_pct) as i128)
                            / 100i128;
                        let new_power = scaled.max(1) as u64;
                        game_state.player.stats.combat_power = new_power;
                        action_result.stat_changes.push(StatChange {
                            stat_name: "combat_power".to_string(),
                            old_value: old_power.to_string(),
                            new_value: new_power.to_string(),
                        });
                        action_result.description = format!(
                            "{}（行为策略修正: {} {}%）",
                            action_result.description,
                            strategy_label(strategy),
                            strategy_delta_pct
                        );
                        push_growth_log(
                            &mut game_state,
                            format!(
                                "战斗策略影响：{}，战力 {} -> {}",
                                strategy_label(strategy),
                                old_power,
                                new_power
                            ),
                        );
                    }
                    combat_strategy_note = Some(format!(
                        "{}（{}）",
                        strategy_label(strategy),
                        strategy_reason
                    ));
                    let (env_delta_pct, env_reason) =
                        evaluate_environment_combat_modifier(&game_state);
                    if env_delta_pct != 0 && hard_constraints.accepted {
                        let old_power = game_state.player.stats.combat_power;
                        let scaled = (old_power as i128)
                            .saturating_mul((100 + env_delta_pct) as i128)
                            / 100i128;
                        let new_power = scaled.max(1) as u64;
                        game_state.player.stats.combat_power = new_power;
                        action_result.stat_changes.push(StatChange {
                            stat_name: "combat_power".to_string(),
                            old_value: old_power.to_string(),
                            new_value: new_power.to_string(),
                        });
                        action_result.description = format!(
                            "{}（环境修正: {}%）",
                            action_result.description, env_delta_pct
                        );
                        push_growth_log(
                            &mut game_state,
                            format!(
                                "环境影响战斗：战力 {} -> {}（{}）",
                                old_power, new_power, env_reason
                            ),
                        );
                    }
                    if !env_reason.is_empty() {
                        action_result.description =
                            format!("{}；环境依据：{}", action_result.description, env_reason);
                    }
                    let (style_delta_pct, style_reason) =
                        evaluate_style_counter_modifier(&game_state.player.stats, &option_hint);
                    if style_delta_pct != 0 && hard_constraints.accepted {
                        let old_power = game_state.player.stats.combat_power;
                        let scaled = (old_power as i128)
                            .saturating_mul((100 + style_delta_pct) as i128)
                            / 100i128;
                        let new_power = scaled.max(1) as u64;
                        game_state.player.stats.combat_power = new_power;
                        action_result.stat_changes.push(StatChange {
                            stat_name: "combat_power".to_string(),
                            old_value: old_power.to_string(),
                            new_value: new_power.to_string(),
                        });
                        action_result.description = format!(
                            "{}（流派克制修正: {}%）",
                            action_result.description, style_delta_pct
                        );
                        push_growth_log(
                            &mut game_state,
                            format!(
                                "流派克制影响：战力 {} -> {}（{}）",
                                old_power, new_power, style_reason
                            ),
                        );
                    }
                    action_result.description =
                        format!("{}；克制依据：{}", action_result.description, style_reason);
                    let (semantic_delta_pct, semantic_reasons, high_risk_technique) =
                        evaluate_technique_semantic_modifier(&game_state.player.stats);
                    if semantic_delta_pct != 0 && hard_constraints.accepted {
                        let old_power = game_state.player.stats.combat_power;
                        let scaled = (old_power as i128)
                            .saturating_mul((100 + semantic_delta_pct) as i128)
                            / 100i128;
                        let new_power = scaled.max(1) as u64;
                        game_state.player.stats.combat_power = new_power;
                        action_result.stat_changes.push(StatChange {
                            stat_name: "combat_power".to_string(),
                            old_value: old_power.to_string(),
                            new_value: new_power.to_string(),
                        });
                        action_result.description = format!(
                            "{}（功法语义修正: {}%）",
                            action_result.description, semantic_delta_pct
                        );
                        push_growth_log(
                            &mut game_state,
                            format!("功法语义影响战斗：战力 {} -> {}", old_power, new_power),
                        );
                    }
                    if !semantic_reasons.is_empty() {
                        action_result.description = format!(
                            "{}；语义依据：{}",
                            action_result.description,
                            semantic_reasons.join(" / ")
                        );
                    }
                    let realm_level = game_state.player.stats.cultivation_realm.level;
                    let check = crate::numeric_guard::validate_character_combat_power(
                        realm_level,
                        game_state.player.stats.combat_power,
                    );
                    if check.accepted && check.normalized {
                        if let Some(v) = check.normalized_value {
                            let old_power = game_state.player.stats.combat_power;
                            let new_power = v.round().max(1.0) as u64;
                            game_state.player.stats.combat_power = new_power;
                            action_result.stat_changes.push(StatChange {
                                stat_name: "combat_power".to_string(),
                                old_value: old_power.to_string(),
                                new_value: new_power.to_string(),
                            });
                            if let Some(reason) = check.reason {
                                combat_guard_reason = Some(reason.clone());
                                action_result.description = format!(
                                    "{}（数值校验：{}）",
                                    action_result.description, reason
                                );
                            }
                        }
                    } else if !check.accepted {
                        action_result.success = false;
                        combat_guard_reason = check.reason.clone();
                        action_result.description = format!(
                            "{}（数值校验未通过：{}）",
                            action_result.description,
                            check.reason.unwrap_or_else(|| "未知原因".to_string())
                        );
                    }
                    let aftermath_summary = apply_combat_aftermath(
                        &mut game_state,
                        action_result.success,
                        Some(strategy),
                    );
                    if high_risk_technique {
                        let status = &mut game_state.player.combat_status;
                        status.injury_level = status.injury_level.saturating_add(1).min(10);
                        action_result
                            .events
                            .push("高风险功法反噬：伤势+1".to_string());
                    }
                    action_result.events.push(aftermath_summary.clone());
                    action_result.description =
                        format!("{}；{}", action_result.description, aftermath_summary);
                    push_growth_log(&mut game_state, format!("战斗成长：{}", aftermath_summary));
                }
                Action::Rest | Action::Custom { .. } => {}
            }
        }
    }

    game_state.game_time.advance_days(1);
    let timestamp = u64::from(game_state.game_time.total_days);

    let context_bundle = if quick_mode {
        None
    } else {
        build_plot_context_for_generation(&game_state, &plot_state, &action)
    };
    let mut plot_state_for_generation = plot_state.clone();
    let narrative_hint =
        narrative_density_and_pacing_hint(plot_state.current_chapter.interaction_count);
    plot_state_for_generation.current_scene.description = format!(
        "{}{}",
        plot_state_for_generation.current_scene.description, narrative_hint
    );
    if let Some(bundle) = &context_bundle {
        let inject = render_generation_context(bundle);
        if !inject.is_empty() {
            plot_state_for_generation.current_scene.description = format!(
                "{}{}",
                plot_state_for_generation.current_scene.description, inject
            );
        }
    }

    let cache_key = if quick_mode {
        None
    } else {
        Some(build_action_plot_cache_key(
            &game_state,
            &plot_state,
            &action,
        ))
    };
    let plot_generation_started = Instant::now();
    let (mut plot_update, cache_hit_semantic) = if quick_mode {
        (
            plot_engine.advance_plot_fast(&plot_state_for_generation, &action_result),
            false,
        )
    } else if let Some(key) = cache_key.as_deref() {
        if let Some(cached) = get_cached_plot_update(key) {
            (cached, true)
        } else {
            (
                plot_engine
                    .advance_plot_async_with_policy(
                        &plot_state_for_generation,
                        &action_result,
                        !selected_option_requires_llm_story,
                    )
                    .await,
                false,
            )
        }
    } else {
        (
            plot_engine
                .advance_plot_async_with_policy(
                    &plot_state_for_generation,
                    &action_result,
                    !selected_option_requires_llm_story,
                )
                .await,
            false,
        )
    };
    let plot_generation_ms = plot_generation_started.elapsed().as_millis();
    if !quick_mode && !cache_hit_semantic {
        if let Some(key) = cache_key.clone() {
            put_cached_plot_update(key, plot_update.clone());
        }
    }

    if let Some(bundle) = context_bundle {
        let ctx_diag = format!(
            "上下文注入：facts={}, recent_ctx={}, chapter_refs={}, recent_events={}, token_used={}",
            bundle.hard_facts.len(),
            bundle.recent_context.len(),
            bundle.chapter_summaries.len(),
            bundle.recent_events.len(),
            bundle.token_budget_used
        );
        match &mut plot_update.generation_diagnostics {
            Some(diag) => {
                diag.push('；');
                diag.push_str(&ctx_diag);
            }
            None => {
                plot_update.generation_diagnostics = Some(ctx_diag);
            }
        }
    }
    if quick_mode {
        match &mut plot_update.generation_diagnostics {
            Some(diag) => diag.push_str("；链路：quick_mode_rule_only"),
            None => {
                plot_update.generation_diagnostics = Some("链路：quick_mode_rule_only".to_string())
            }
        }
    }
    if cache_hit_semantic {
        match &mut plot_update.generation_diagnostics {
            Some(diag) => diag.push_str("；链路：cache_hit_semantic"),
            None => {
                plot_update.generation_diagnostics = Some("链路：cache_hit_semantic".to_string())
            }
        }
    }

    let mut consistency_report = validate_and_repair_plot_update(
        &plot_state,
        &plot_update,
        &action_result,
        game_state.player.stats.cultivation_realm.level,
        game_state.player.stats.combat_power,
        &game_state.player.name,
    );

    if !quick_mode
        && has_consistency_issue(&consistency_report, "chapter_goal_weak")
        && total_started.elapsed().as_millis() <= REGEN_LATENCY_BUDGET_MS
    {
        let mut regenerated_state = plot_state_for_generation.clone();
        regenerated_state.current_scene.description = format!(
            "{}{}",
            regenerated_state.current_scene.description,
            chapter_goal_regeneration_hint(plot_state.current_chapter.interaction_count)
        );

        let mut regenerated_update = plot_engine
            .advance_plot_async_with_policy(
                &regenerated_state,
                &action_result,
                !selected_option_requires_llm_story,
            )
            .await;
        let regenerated_report = validate_and_repair_plot_update(
            &plot_state,
            &regenerated_update,
            &action_result,
            game_state.player.stats.cultivation_realm.level,
            game_state.player.stats.combat_power,
            &game_state.player.name,
        );

        let improved_goal_hit = !has_consistency_issue(&regenerated_report, "chapter_goal_weak");
        let not_worse_risk = regenerated_report.risk_score() <= consistency_report.risk_score();
        if improved_goal_hit || not_worse_risk {
            match &mut regenerated_update.generation_diagnostics {
                Some(diag) => diag.push_str("；章节目标重生成：accepted"),
                None => {
                    regenerated_update.generation_diagnostics =
                        Some("章节目标重生成：accepted".to_string())
                }
            }
            plot_update = regenerated_update;
            consistency_report = regenerated_report;
        } else {
            match &mut plot_update.generation_diagnostics {
                Some(diag) => diag.push_str("；章节目标重生成：rejected"),
                None => {
                    plot_update.generation_diagnostics =
                        Some("章节目标重生成：rejected".to_string())
                }
            }
        }
    } else if !quick_mode && has_consistency_issue(&consistency_report, "chapter_goal_weak") {
        match &mut plot_update.generation_diagnostics {
            Some(diag) => diag.push_str("；章节目标重生成：skipped(latency_budget)"),
            None => {
                plot_update.generation_diagnostics =
                    Some("章节目标重生成：skipped(latency_budget)".to_string())
            }
        }
    }

    if !quick_mode
        && (is_hollow_expression(&plot_update.plot_text)
            || narrative_dimension_coverage(&plot_update.plot_text) < 2)
        && total_started.elapsed().as_millis() <= REGEN_LATENCY_BUDGET_MS
    {
        let mut regenerated_state = plot_state_for_generation.clone();
        let regen_hint = format!(
            "{}{}",
            hollow_expression_regeneration_hint(),
            narrative_density_and_pacing_hint(plot_state.current_chapter.interaction_count)
        );
        regenerated_state.current_scene.description = format!(
            "{}{}",
            regenerated_state.current_scene.description, regen_hint
        );
        let mut regenerated_update = plot_engine
            .advance_plot_async_with_policy(
                &regenerated_state,
                &action_result,
                !selected_option_requires_llm_story,
            )
            .await;
        let regenerated_report = validate_and_repair_plot_update(
            &plot_state,
            &regenerated_update,
            &action_result,
            game_state.player.stats.cultivation_realm.level,
            game_state.player.stats.combat_power,
            &game_state.player.name,
        );

        let improved_density = !is_hollow_expression(&regenerated_update.plot_text);
        let not_worse_risk = regenerated_report.risk_score() <= consistency_report.risk_score() + 2;
        if improved_density || not_worse_risk {
            match &mut regenerated_update.generation_diagnostics {
                Some(diag) => diag.push_str("；叙事厚度重生成：accepted"),
                None => {
                    regenerated_update.generation_diagnostics =
                        Some("叙事厚度重生成：accepted".to_string())
                }
            }
            plot_update = regenerated_update;
            consistency_report = regenerated_report;
        } else {
            match &mut plot_update.generation_diagnostics {
                Some(diag) => diag.push_str("；叙事厚度重生成：rejected"),
                None => {
                    plot_update.generation_diagnostics =
                        Some("叙事厚度重生成：rejected".to_string())
                }
            }
        }
    } else if !quick_mode
        && (is_hollow_expression(&plot_update.plot_text)
            || narrative_dimension_coverage(&plot_update.plot_text) < 2)
    {
        match &mut plot_update.generation_diagnostics {
            Some(diag) => diag.push_str("；叙事厚度重生成：skipped(latency_budget)"),
            None => {
                plot_update.generation_diagnostics =
                    Some("叙事厚度重生成：skipped(latency_budget)".to_string())
            }
        }
    }
    if let Some(text) = consistency_report.repaired_plot_text.clone() {
        plot_update.plot_text = text;
    }
    if let Some(next_location) = consistency_report.override_location.clone() {
        plot_state.current_scene.location = next_location;
    }
    if let Some(summary) = consistency_report.override_chapter_summary.clone() {
        plot_update.chapter_summary = Some(summary);
    }
    if !quick_mode
        && plot_state.settings.llm_strict_mode
        && diagnostics_used_preset_fallback(plot_update.generation_diagnostics.as_deref())
    {
        return Err(
            "LLM 严格模式：本轮未获得可用 LLM 剧情文本，已中止推进（未写入预设文本）。".to_string(),
        );
    }
    if !quick_mode
        && selected_option_requires_llm_story
        && diagnostics_used_preset_fallback(plot_update.generation_diagnostics.as_deref())
    {
        if let Some(rescue_text) = plot_engine
            .generate_option_plot_rescue_async(&plot_state_for_generation, &action_result)
            .await
        {
            plot_update.plot_text = rescue_text;
            plot_update.generation_diagnostics = match plot_update.generation_diagnostics.take() {
                Some(diag) => Some(format!("{diag}；选项续写救援：llm_plain_rescue")),
                None => Some("选项续写救援：llm_plain_rescue".to_string()),
            };
        } else {
            return Err(
                "本轮为选项续写：未获得可用 LLM 剧情文本，已阻止写入预设回退文本。请检查模型配置后重试。"
                    .to_string(),
            );
        }
    }

    // Phase 2: 双通道（state_patch + narrative）优先，失败则保留现有 plot_engine 结果。
    if !quick_mode {
        if let Some(mut registry) = world_registry.clone() {
            let turn_update_started = Instant::now();
            let (turn_result, turn_update_error) = registry
                .generate_turn_update_with_llm_diagnostic(
                    &game_state,
                    &plot_state,
                    &action.content,
                    WORLD_REGISTRY_REFERENCE,
                    WORLD_REGISTRY_SUPPLEMENT,
                )
                .await;
            let turn_update_ms = turn_update_started.elapsed().as_millis();
            if let Some(turn_result) = turn_result {
                let previous_plot_text = plot_update.plot_text.clone();
                let previous_options = plot_update.available_options.clone();
                let previous_waiting = plot_update.is_waiting_for_input;
                let registry_before_patch = registry.clone();
                match registry.apply_state_patch_transactional(&turn_result.state_patch) {
                    Ok(patch_notes) => {
                        let narrative = turn_result.narrative_segment.trim().to_string();
                        let contract_ok = if let Err(contract_err) =
                            registry.validate_turn_narrative_contract(&narrative)
                        {
                            if plot_state.settings.llm_strict_mode {
                                return Err(format!(
                                    "LLM 严格模式：叙事合同校验失败：{}",
                                    contract_err
                                ));
                            }
                            registry = registry_before_patch.clone();
                            plot_update.plot_text = previous_plot_text.clone();
                            plot_update.available_options = previous_options.clone();
                            plot_update.is_waiting_for_input = previous_waiting;
                            match &mut plot_update.generation_diagnostics {
                                Some(diag) => {
                                    diag.push('；');
                                    diag.push_str(&format!("双通道叙事丢弃：{}", contract_err));
                                }
                                None => {
                                    plot_update.generation_diagnostics =
                                        Some(format!("双通道叙事丢弃：{}", contract_err))
                                }
                            }
                            false
                        } else {
                            true
                        };

                        let entity_ref_ok = if !contract_ok {
                            false
                        } else if let Err(unknown_entities) =
                            registry.validate_narrative_entity_references(&narrative)
                        {
                            let detail = format!(
                                "实体引用校验失败，未入表实体: {}",
                                unknown_entities.join(",")
                            );
                            if plot_state.settings.llm_strict_mode {
                                return Err(format!("LLM 严格模式：{}", detail));
                            }
                            registry = registry_before_patch.clone();
                            plot_update.plot_text = previous_plot_text.clone();
                            plot_update.available_options = previous_options.clone();
                            plot_update.is_waiting_for_input = previous_waiting;
                            match &mut plot_update.generation_diagnostics {
                                Some(diag) => {
                                    diag.push('；');
                                    diag.push_str(&detail);
                                }
                                None => plot_update.generation_diagnostics = Some(detail),
                            }
                            false
                        } else {
                            true
                        };

                        if entity_ref_ok {
                            if !narrative.is_empty() {
                                plot_update.plot_text = narrative;
                            }
                            if !turn_result.choices.is_empty() {
                                plot_update.available_options =
                                    player_options_from_choice_texts(&turn_result.choices);
                                plot_update.is_waiting_for_input = true;
                            }
                        }
                        apply_registry_to_game_state(&mut game_state, &registry);
                        world_registry = Some(registry);
                        let note = if patch_notes.is_empty() {
                            format!(
                                "双通道生成：llm_turn_update(no_patch,turn_update_ms={})",
                                turn_update_ms
                            )
                        } else {
                            format!(
                                "双通道生成：llm_turn_update({},turn_update_ms={})",
                                patch_notes.join(","),
                                turn_update_ms
                            )
                        };
                        match &mut plot_update.generation_diagnostics {
                            Some(diag) => {
                                diag.push('；');
                                diag.push_str(&note);
                            }
                            None => plot_update.generation_diagnostics = Some(note),
                        }
                    }
                    Err(err) => {
                        if plot_state.settings.llm_strict_mode {
                            return Err(format!(
                                "LLM 严格模式：state_patch 校验失败，已中止推进。原因: {}",
                                err
                            ));
                        }
                        world_registry = Some(registry);
                        match &mut plot_update.generation_diagnostics {
                            Some(diag) => {
                                diag.push_str(&format!("；双通道补丁丢弃：{}", err));
                            }
                            None => {
                                plot_update.generation_diagnostics =
                                    Some(format!("双通道补丁丢弃：{}", err))
                            }
                        }
                    }
                };
            } else {
                let fallback_note = match turn_update_error {
                    Some(err) => format!(
                        "双通道生成：fallback(plot_engine_only,turn_update_ms={},reason={})",
                        turn_update_ms, err
                    ),
                    None => format!(
                        "双通道生成：fallback(plot_engine_only,turn_update_ms={})",
                        turn_update_ms
                    ),
                };
                match &mut plot_update.generation_diagnostics {
                    Some(diag) => {
                        diag.push('；');
                        diag.push_str(&fallback_note);
                    }
                    None => plot_update.generation_diagnostics = Some(fallback_note),
                }
            }
        }
    }

    let log_entry = if let Some(selected_option_id) = action.selected_option_id {
        if let Some(selected_option) = plot_state
            .current_scene
            .available_options
            .get(selected_option_id)
        {
            match &selected_option.action {
                Action::Combat { .. } => Some((
                    "combat",
                    format!("Player engaged in combat: {}", selected_option.description),
                    EventImportance::Important,
                )),
                Action::Breakthrough => Some((
                    "breakthrough_attempt",
                    format!(
                        "Player attempted breakthrough: {}",
                        selected_option.description
                    ),
                    EventImportance::Important,
                )),
                Action::Custom { .. } | Action::Cultivate | Action::Rest => Some((
                    "player_action",
                    selected_option.description.clone(),
                    EventImportance::Normal,
                )),
            }
        } else {
            None
        }
    } else if matches!(action.action_type, crate::plot_engine::ActionType::FreeText) {
        Some((
            "player_free_text",
            action.content.clone(),
            EventImportance::Normal,
        ))
    } else {
        None
    };

    plot_state.last_action_result = Some(action_result.clone());
    let plot_text_for_history = plot_update.plot_text.clone();
    let mut plot_text_for_player = plot_update.plot_text.clone();
    let mut noname_result = run_noname_observe_only_for_turn(
        &noname_action_summary,
        &game_state,
        &plot_state,
        timestamp,
    )
    .ok();
    let mut noname_plot_text_applied = false;
    if let Some(result) = noname_result.as_mut() {
        let plans = build_noname_apply_plan_set(
            &result.proposal,
            Some(&plot_state),
            Some(plot_text_for_player.as_str()),
        );
        for plan in &plans {
            record_noname_apply_plan(&mut result.trace, plan);
        }
        noname_plot_text_applied = apply_noname_plot_text_hint(&mut plot_text_for_player, result);
    }

    plot_state.append_segment(plot_text_for_history.clone());

    if let Some(title) = plot_update.chapter_title.clone() {
        if !title.trim().is_empty() {
            plot_state.current_chapter.title = title.trim().to_string();
            plot_state.current_scene.name = plot_state.current_chapter.title.clone();
        }
    }

    if plot_update.is_waiting_for_input {
        plot_state.current_chapter.interaction_count = plot_state
            .current_chapter
            .interaction_count
            .saturating_add(1);
    }

    if plot_update.chapter_end {
        plot_state.finalize_chapter(plot_update.chapter_title, plot_update.chapter_summary);
    }

    plot_state.last_generation_diagnostics = plot_update.generation_diagnostics.clone();

    // 用最新段落更新场景描述，避免选项生成长期绑定旧描述导致“选项不变”。
    if !plot_text_for_history.trim().is_empty() {
        plot_state.current_scene.description = plot_text_for_history.trim().to_string();
    }

    let previous_options = plot_state.current_scene.available_options.clone();
    let options_started = Instant::now();

    let mut option_source: String = if plot_update.is_waiting_for_input {
        if !plot_update.available_options.is_empty() {
            plot_state.current_scene.available_options = plot_update.available_options;
            "llm_structured".to_string()
        } else {
            let llm_regenerated = if !quick_mode
                && options_started.elapsed().as_millis() <= OPTION_LLM_LATENCY_BUDGET_MS
            {
                plot_engine.generate_player_options_with_llm(
                    &plot_state.current_scene,
                    &game_state.player.stats,
                )
            } else {
                None
            };
            let (mut regenerated_options, mut source) = if let Some(options) = llm_regenerated {
                (options, "llm_regenerated".to_string())
            } else {
                (
                    plot_engine.generate_player_options(
                        &plot_state.current_scene,
                        &game_state.player.stats,
                    ),
                    if quick_mode {
                        "quick_mode_rule_only".to_string()
                    } else if options_started.elapsed().as_millis() <= OPTION_LLM_LATENCY_BUDGET_MS
                    {
                        "rule_fallback".to_string()
                    } else {
                        "rule_fallback_latency_budget".to_string()
                    },
                )
            };

            if regenerated_options.is_empty() {
                regenerated_options = previous_options;
                source = "previous_reused".to_string();
            }

            // 通过时间推进对兜底选项做轻量轮转，确保连续交互时选项呈现有变化。
            if !regenerated_options.is_empty() {
                let rotation =
                    (game_state.game_time.total_days as usize) % regenerated_options.len();
                regenerated_options.rotate_left(rotation);
                for (idx, option) in regenerated_options.iter_mut().enumerate() {
                    option.id = idx;
                }
            }
            plot_state.current_scene.available_options = regenerated_options;
            source
        }
    } else {
        plot_state.current_scene.available_options.clear();
        "not_waiting_for_input".to_string()
    };

    if plot_update.is_waiting_for_input
        && !plot_update.chapter_end
        && plot_state.current_scene.available_options.is_empty()
    {
        plot_update.is_waiting_for_input = false;
        option_source = "consistency_non_waiting_fallback".to_string();
        match &mut plot_state.last_generation_diagnostics {
            Some(diag) => diag.push_str("；一致性兜底：无选项等待态已改为自动推进"),
            None => {
                plot_state.last_generation_diagnostics =
                    Some("一致性兜底：无选项等待态已改为自动推进".to_string())
            }
        }
    }

    if plot_update.is_waiting_for_input && !plot_update.chapter_end {
        let (mut sanitized_options, repair_notes) = sanitize_generated_options(
            plot_state.current_scene.available_options.clone(),
            &game_state,
            &plot_state.current_scene.location,
            world_registry.as_ref(),
        );

        if sanitized_options.len() < 2 {
            let mut fallback_options = plot_engine
                .generate_player_options(&plot_state.current_scene, &game_state.player.stats);
            let existing = sanitized_options
                .iter()
                .map(|o| o.description.trim().to_string())
                .collect::<HashSet<_>>();
            fallback_options.retain(|opt| !existing.contains(opt.description.trim()));
            fallback_options.truncate(4usize.saturating_sub(sanitized_options.len()));
            sanitized_options.extend(fallback_options);
            if sanitized_options.len() >= 2 {
                match &mut plot_state.last_generation_diagnostics {
                    Some(diag) => diag.push_str("；选项一致性修正：已补足可执行保守选项"),
                    None => {
                        plot_state.last_generation_diagnostics =
                            Some("选项一致性修正：已补足可执行保守选项".to_string())
                    }
                }
            }
        }

        for (idx, option) in sanitized_options.iter_mut().enumerate() {
            option.id = idx;
        }
        plot_state.current_scene.available_options = sanitized_options;

        if !repair_notes.is_empty() {
            let joined = repair_notes.join(",");
            match &mut plot_state.last_generation_diagnostics {
                Some(diag) => {
                    diag.push_str("；选项一致性修正：");
                    diag.push_str(&joined);
                }
                None => {
                    plot_state.last_generation_diagnostics =
                        Some(format!("选项一致性修正：{}", joined))
                }
            }
        }
    }

    let options_generation_ms = options_started.elapsed().as_millis();

    let effective_consistency_report = effective_consistency_report_after_option_resolution(
        consistency_report,
        plot_update.is_waiting_for_input,
        plot_update.chapter_end,
        plot_state.current_scene.available_options.len(),
    );
    let risk_score = effective_consistency_report.risk_score();
    plot_state.last_consistency_risk_score = if risk_score > 0 {
        Some(risk_score)
    } else {
        None
    };
    if let Some(diag) = effective_consistency_report.to_diagnostics() {
        match &mut plot_state.last_generation_diagnostics {
            Some(existing) => {
                existing.push('；');
                existing.push_str(&diag);
            }
            None => {
                plot_state.last_generation_diagnostics = Some(diag);
            }
        }
    }

    plot_state.is_waiting_for_input = plot_update.is_waiting_for_input;
    plot_state.recalculate_interaction_state();
    if plot_update.chapter_end {
        plot_state.interaction_state = PlotInteractionState::Cooldown;
    }

    plot_state.last_option_generation_source = Some(option_source.clone());
    match &mut plot_state.last_generation_diagnostics {
        Some(diag) => {
            diag.push_str(&format!("；选项来源：{}", option_source));
        }
        None => {
            plot_state.last_generation_diagnostics = Some(format!("选项来源：{}", option_source));
        }
    }
    let total_ms = total_started.elapsed().as_millis();
    match &mut plot_state.last_generation_diagnostics {
        Some(diag) => {
            diag.push_str(&format!(
                "；耗时(ms)：total={},plot_gen={},option_gen={}",
                total_ms, plot_generation_ms, options_generation_ms
            ));
        }
        None => {
            plot_state.last_generation_diagnostics = Some(format!(
                "耗时(ms)：total={},plot_gen={},option_gen={}",
                total_ms, plot_generation_ms, options_generation_ms
            ));
        }
    }

    let mut engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    if let Some((event_type, message, importance)) = log_entry {
        engine.log_event(timestamp, event_type, message, importance);
    }

    if should_emit_combat_explanation {
        let explanation = build_combat_explanation(
            &action_result,
            game_state.player.stats.cultivation_realm.level,
            game_state.player.stats.combat_power,
            combat_guard_reason.as_deref(),
            combat_strategy_note.as_deref(),
        );
        let payload = serde_json::to_string(&explanation).unwrap_or(explanation.summary);
        engine.log_event(
            timestamp,
            "combat_explanation",
            payload,
            EventImportance::Important,
        );
    }

    let _npc_reactions = engine
        .process_npc_reactions_for_events(&plot_update.triggered_events)
        .map_err(|e| e.to_string())?;

    // V2 记忆回写：短期事件 / 中期章节摘要 / 长期事实
    if let Ok(mut memory) = memory_layers().lock() {
        for (idx, event_text) in plot_update.triggered_events.iter().enumerate() {
            memory.push_recent_event(
                MemoryEntry {
                    event_id: format!("evt-{}-{}", timestamp, idx),
                    summary: event_text.clone(),
                    turn: timestamp,
                },
                120,
            );
        }

        if let Some(last_chapter) = plot_state.chapters.last() {
            memory.upsert_chapter_summary(ChapterSummary {
                chapter_id: format!("chapter-{}", last_chapter.index),
                title: last_chapter.title.clone(),
                summary: if last_chapter.summary.trim().is_empty() {
                    last_chapter
                        .content
                        .last()
                        .cloned()
                        .unwrap_or_else(|| "（无摘要）".to_string())
                } else {
                    last_chapter.summary.clone()
                },
            });
        }

        memory.upsert_world_fact(WorldFact {
            fact_id: format!("fact-location-{}", timestamp),
            subject: "player".to_string(),
            predicate: "at".to_string(),
            object: game_state.player.location.clone(),
        });
    }

    if let Some(mut noname_result) = noname_result {
        apply_noname_low_risk_outputs(
            &mut plot_state,
            &mut noname_result,
            noname_plot_text_applied,
        );
        append_noname_observation_diagnostics(
            &mut plot_state.last_generation_diagnostics,
            noname_result.trace.mode,
            &noname_result.observation,
            noname_result.guardrail_result.as_ref(),
            &noname_result.trace,
        );
    }

    game_state.player.refresh_profile_views();
    engine
        .update_current_state(game_state)
        .map_err(|e| e.to_string())?;
    if let Some(registry) = world_registry {
        engine.update_world_registry(registry);
    }
    engine
        .update_plot_state(plot_state)
        .map_err(|e| e.to_string())?;

    Ok(plot_text_for_player)
}

#[tauri::command]
pub async fn get_noname_recent_traces() -> Result<Vec<NoNameTrace>, String> {
    let runtime = noname_runtime()
        .lock()
        .map_err(|_| "NoName runtime lock poisoned".to_string())?;
    Ok(runtime.get_recent_traces())
}

#[tauri::command]
pub async fn clear_noname_recent_traces() -> Result<(), String> {
    let mut runtime = noname_runtime()
        .lock()
        .map_err(|_| "NoName runtime lock poisoned".to_string())?;
    runtime.clear_traces();
    Ok(())
}

#[tauri::command]
pub async fn get_noname_mode() -> Result<NoNameMode, String> {
    let runtime = noname_runtime()
        .lock()
        .map_err(|_| "NoName runtime lock poisoned".to_string())?;
    Ok(runtime.mode())
}

#[tauri::command]
pub async fn set_noname_mode(
    mode: NoNameMode,
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<NoNameMode, String> {
    let mut runtime = noname_runtime()
        .lock()
        .map_err(|_| "NoName runtime lock poisoned".to_string())?;
    runtime.set_mode(mode);
    let mut engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    engine.set_noname_mode(runtime.mode());
    Ok(runtime.mode())
}

#[tauri::command]
pub async fn get_game_state(engine: State<'_, Mutex<GameEngine>>) -> Result<GameState, String> {
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    engine.get_current_state().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_world_registry(
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<WorldRegistry, String> {
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    engine.get_world_registry().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn apply_world_registry_patch(
    patch: Value,
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<WorldRegistry, String> {
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let mut registry = engine.get_world_registry().map_err(|e| e.to_string())?;
    registry
        .apply_state_patch_transactional(&patch)
        .map_err(|e| format!("属性表 patch 校验失败: {}", e))?;

    let mut game_state = engine.get_current_state().map_err(|e| e.to_string())?;
    apply_registry_to_game_state(&mut game_state, &registry);
    engine
        .update_current_state(game_state)
        .map_err(|e| e.to_string())?;
    engine.update_world_registry(registry.clone());
    Ok(registry)
}

#[tauri::command]
pub async fn save_game(slot_id: u32, engine: State<'_, Mutex<GameEngine>>) -> Result<(), String> {
    validate_slot_id(slot_id).map_err(|e| map_error("保存存档失败", e))?;
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    engine.save_game(slot_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_game(
    slot_id: u32,
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<GameState, String> {
    validate_slot_id(slot_id).map_err(|e| map_error("加载存档失败", e))?;
    let mut engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let game_state = engine.load_game(slot_id).map_err(|e| e.to_string())?;
    if let Ok(mut runtime) = noname_runtime().lock() {
        runtime.set_mode(engine.noname_mode());
    }
    Ok(game_state)
}

#[tauri::command]
pub async fn list_save_slots(
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<Vec<SaveInfo>, String> {
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    engine.list_saves().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_reachable_locations(
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<Vec<String>, String> {
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let game_state = engine.get_current_state().map_err(|e| e.to_string())?;
    Ok(compute_reachable_location_ids(&game_state))
}

#[tauri::command]
pub async fn get_map_overview(
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<Vec<MapLocationOverview>, String> {
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let game_state = engine.get_current_state().map_err(|e| e.to_string())?;
    Ok(compute_map_overview(&game_state))
}

#[tauri::command]
pub async fn travel_to_location(
    location_id: String,
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<String, String> {
    validate_non_empty(&location_id, "目标地点").map_err(|e| map_error("移动失败", e))?;
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let mut game_state = engine.get_current_state().map_err(|e| e.to_string())?;
    let mut plot_state = engine.get_plot_state().map_err(|e| e.to_string())?;
    let timestamp_before = u64::from(game_state.game_time.total_days);

    let (message, encounter_triggered) =
        apply_travel_and_encounter(&mut game_state, &mut plot_state, &location_id)?;
    let timestamp = u64::from(game_state.game_time.total_days.max(timestamp_before as u32));
    engine.log_event(
        timestamp,
        "travel",
        message.clone(),
        EventImportance::Normal,
    );
    if encounter_triggered {
        engine.log_event(
            timestamp,
            "encounter",
            format!("移动遭遇：{}", message),
            EventImportance::Important,
        );
    }
    game_state.player.refresh_profile_views();
    engine
        .update_current_state(game_state)
        .map_err(|e| e.to_string())?;
    engine
        .update_plot_state(plot_state)
        .map_err(|e| e.to_string())?;
    Ok(message)
}

#[tauri::command]
pub async fn migrate_all_saves(
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<MigrationBatchReport, String> {
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    engine.migrate_all_saves().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_script(
    script_path: String,
    _engine: State<'_, Mutex<GameEngine>>,
) -> Result<Script, String> {
    use crate::script_manager::ScriptManager;

    validate_file_path(&script_path, &["json"]).map_err(|e| map_error("加载剧本失败", e))?;
    let manager = ScriptManager::new();
    manager
        .load_custom_script(&script_path)
        .map_err(|e| map_error("加载剧本失败", e))
}

#[tauri::command]
pub async fn generate_random_script() -> Result<Script, String> {
    use crate::script_manager::ScriptManager;

    let manager = ScriptManager::new();
    manager
        .generate_random_script()
        .await
        .map_err(|e| map_error("随机剧本生成失败", e))
}

#[tauri::command]
pub async fn parse_novel_characters(novel_path: String) -> Result<Vec<String>, String> {
    use crate::script_manager::ScriptManager;

    validate_file_path(&novel_path, &["txt", "md"])
        .map_err(|e| map_error("解析小说角色失败", e))?;
    let manager = ScriptManager::new();
    manager
        .extract_novel_characters(&novel_path)
        .map_err(|e| map_error("解析小说角色失败", e))
}

#[tauri::command]
pub async fn load_existing_novel(
    novel_path: String,
    selected_character: String,
) -> Result<Script, String> {
    use crate::script_manager::ScriptManager;

    validate_file_path(&novel_path, &["txt", "md"])
        .map_err(|e| map_error("导入现有小说失败", e))?;
    if selected_character.trim().is_empty() {
        return Err(map_error(
            "导入现有小说失败",
            AppError::new(
                crate::app_error::AppErrorKind::InvalidInput,
                "请选择有效角色",
            ),
        ));
    }
    let manager = ScriptManager::new();
    manager
        .load_existing_novel(&novel_path, &selected_character)
        .map_err(|e| map_error("导入现有小说失败", e))
}

#[tauri::command]
pub async fn get_player_options(
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<Vec<PlayerOption>, String> {
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let plot_state = engine.get_plot_state().map_err(|e| e.to_string())?;
    Ok(plot_state.current_scene.available_options)
}

#[tauri::command]
pub async fn initialize_plot(engine: State<'_, Mutex<GameEngine>>) -> Result<PlotState, String> {
    let total_started = Instant::now();
    let (player_name, realm_name, spiritual_root, location) = {
        let engine = match engine.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let state = engine.get_current_state().map_err(|e| e.to_string())?;
        (
            state.player.name,
            state.player.stats.cultivation_realm.name,
            state.player.stats.spiritual_root.display_elements(),
            state
                .script
                .world_setting
                .locations
                .iter()
                .find(|loc| loc.id == state.player.location)
                .map(|loc| loc.name.clone())
                .unwrap_or(state.player.location),
        )
    };

    let plot_engine = PlotEngine::new();
    let opening_started = Instant::now();
    let opening = plot_engine
        .generate_opening_plot_async(&player_name, &realm_name, &spiritual_root, &location)
        .await;
    let opening_gen_ms = opening_started.elapsed().as_millis();
    if !opening.from_llm {
        return Err(
            "初始化剧情失败：未获取到 LLM 开局内容（已禁止预设文案回退）。请检查模型配置、网络与超时后重试。"
                .to_string(),
        );
    }

    let opening_option_started = Instant::now();
    let opening_options = if opening.options.is_empty() {
        None
    } else {
        Some(
            opening
                .options
                .iter()
                .enumerate()
                .map(|(idx, text)| PlayerOption {
                    id: idx,
                    description: text.clone(),
                    requirements: vec![],
                    action: Action::Custom {
                        description: text.clone(),
                    },
                })
                .collect(),
        )
    };
    let opening_option_build_ms = opening_option_started.elapsed().as_millis();

    let mut engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let persist_started = Instant::now();
    let mut plot_state = engine
        .initialize_plot_with_opening(opening.text, opening_options)
        .map_err(|e| e.to_string())?;
    let state_persist_ms = persist_started.elapsed().as_millis();
    let total_ms = total_started.elapsed().as_millis();
    plot_state.last_option_generation_source =
        Some(if plot_state.current_scene.available_options.is_empty() {
            "opening_default_rule".to_string()
        } else {
            "opening_llm_options".to_string()
        });
    plot_state.last_generation_diagnostics = Some(format!(
        "初始化耗时(ms)：total={},input_assembly=0,model_request_parse={},opening_option_build={},state_persist={}",
        total_ms, opening_gen_ms, opening_option_build_ms, state_persist_ms
    ));
    engine
        .update_plot_state(plot_state.clone())
        .map_err(|e| e.to_string())?;
    Ok(plot_state)
}

#[tauri::command]
pub async fn get_plot_state(engine: State<'_, Mutex<GameEngine>>) -> Result<PlotState, String> {
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    engine.get_plot_state().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rehydrate_last_quick_mode_segment(
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<bool, String> {
    let (mut plot_state, action_result) = {
        let engine = match engine.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let plot_state = engine.get_plot_state().map_err(|e| e.to_string())?;
        let action_result = match plot_state.last_action_result.clone() {
            Some(v) => v,
            None => return Ok(false),
        };
        (plot_state, action_result)
    };

    if !plot_state
        .last_generation_diagnostics
        .as_deref()
        .unwrap_or("")
        .contains("quick_mode_rule_only")
    {
        return Ok(false);
    }

    let plot_engine = PlotEngine::new();
    let Some(rehydrated_text) = plot_engine
        .generate_option_plot_rescue_async(&plot_state, &action_result)
        .await
    else {
        return Ok(false);
    };
    if rehydrated_text.trim().is_empty() {
        return Ok(false);
    }

    if let Some(last) = plot_state.current_chapter.content.last_mut() {
        *last = rehydrated_text.clone();
    }
    if let Some(last) = plot_state.plot_history.last_mut() {
        *last = rehydrated_text.clone();
    }
    plot_state.current_scene.description = rehydrated_text;

    match &mut plot_state.last_generation_diagnostics {
        Some(diag) => diag.push_str("；快速模式后台补全：llm_plain_rehydrate"),
        None => {
            plot_state.last_generation_diagnostics =
                Some("快速模式后台补全：llm_plain_rehydrate".to_string())
        }
    }

    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    engine
        .update_plot_state(plot_state)
        .map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub async fn update_plot_settings(
    settings: PlotSettings,
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<PlotState, String> {
    if settings.min_interactions_per_chapter == 0
        || settings.max_interactions_per_chapter == 0
        || settings.min_interactions_per_chapter > settings.max_interactions_per_chapter
    {
        return Err(map_error(
            "更新剧情设置失败",
            AppError::new(
                crate::app_error::AppErrorKind::InvalidInput,
                "每章互动次数范围不合法",
            ),
        ));
    }
    if settings.target_chapter_words_min == 0
        || settings.target_chapter_words_max == 0
        || settings.target_chapter_words_min > settings.target_chapter_words_max
    {
        return Err(map_error(
            "更新剧情设置失败",
            AppError::new(
                crate::app_error::AppErrorKind::InvalidInput,
                "章节字数范围不合法",
            ),
        ));
    }
    if settings.novel_style.trim().is_empty() {
        return Err(map_error(
            "更新剧情设置失败",
            AppError::new(
                crate::app_error::AppErrorKind::InvalidInput,
                "小说风格不能为空",
            ),
        ));
    }
    let engine = match engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    engine
        .update_plot_settings(settings)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_consistency_policy() -> Result<ConsistencyPolicy, String> {
    Ok(get_runtime_policy())
}

#[tauri::command]
pub async fn update_consistency_policy(
    policy: ConsistencyPolicy,
) -> Result<ConsistencyPolicy, String> {
    update_runtime_policy(policy).map_err(|e| {
        map_error(
            "更新一致性策略失败",
            AppError::new(crate::app_error::AppErrorKind::InvalidInput, e),
        )
    })
}

#[tauri::command]
pub async fn reset_consistency_policy() -> Result<ConsistencyPolicy, String> {
    reset_runtime_policy().map_err(|e| {
        map_error(
            "重置一致性策略失败",
            AppError::new(crate::app_error::AppErrorKind::InvalidInput, e),
        )
    })
}

#[tauri::command]
pub async fn generate_novel(
    title: String,
    engine: State<'_, Mutex<GameEngine>>,
) -> Result<Novel, String> {
    validate_non_empty(&title, "小说标题").map_err(|e| map_error("生成小说失败", e))?;
    let (events, plot_state) = {
        let engine = engine.lock().map_err(|e| e.to_string())?;
        let state = engine.get_current_state().map_err(|e| e.to_string())?;
        let plot_state = engine.get_plot_state().ok();
        (state.event_history, plot_state)
    };
    if let Some(plot_state) = plot_state {
        if !plot_state.chapters.is_empty() || !plot_state.current_chapter.content.is_empty() {
            return Ok(generate_novel_from_plot_state(&title, &plot_state));
        }
    }
    generate_novel_from_events(&title, &events).await
}

#[tauri::command]
pub async fn export_novel(novel: Novel, output_path: String) -> Result<(), String> {
    validate_output_path(&output_path, &["txt"]).map_err(|e| map_error("导出小说失败", e))?;
    export_novel_to_path(&novel, &output_path)
}

#[tauri::command]
pub async fn summarize_generation_diagnostics(
    diagnostics: Vec<String>,
) -> Result<GenerationTimingSummary, String> {
    summarize_generation_timing_diagnostics(&diagnostics)
        .ok_or_else(|| "未找到可解析的耗时诊断数据".to_string())
}

#[tauri::command]
pub async fn summarize_generation_failures(
    diagnostics: Vec<String>,
) -> Result<GenerationFailureSummary, String> {
    summarize_generation_failure_diagnostics(&diagnostics)
        .ok_or_else(|| "未提供可统计的诊断数据".to_string())
}

async fn generate_novel_from_events(
    title: &str,
    events: &[crate::event_log::GameEvent],
) -> Result<Novel, String> {
    let generator = NovelGenerator::new();
    generator.generate_novel(title.to_string(), events).await
}

fn generate_novel_from_plot_state(title: &str, plot_state: &PlotState) -> Novel {
    let generator = NovelGenerator::new();
    generator.generate_chronicle_from_plot(title.to_string(), plot_state)
}

fn export_novel_to_path(novel: &Novel, output_path: &str) -> Result<(), String> {
    let generator = NovelGenerator::new();
    generator.export_to_file(novel, output_path)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitEntitiesInput {
    pub world_id: String,
    pub run_id: String,
    pub candidates: Vec<EntityCandidateRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitEntitiesResponse {
    pub committed: Vec<StoredEntity>,
    pub rejected: Vec<ResolvedEntity>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateCandidatesInput {
    pub entity_type: EntityType,
    pub hint: Option<String>,
    pub count: Option<u8>,
}

#[tauri::command]
pub async fn generate_entity_candidates(
    input: GenerateCandidatesInput,
) -> Result<Vec<EntityCandidateRequest>, String> {
    let count = input.count.unwrap_or(1).clamp(1, 5) as usize;
    let hint = input.hint.unwrap_or_else(|| "default".to_string());
    let candidates = (0..count)
        .map(|idx| {
            let payload = match input.entity_type {
                EntityType::Technique => json!({
                    "techniqueId": format!("tech_{}_{}", hint, idx + 1),
                    "name": format!("{} Technique {}", hint, idx + 1),
                    "tags": ["generated"],
                    "realmRequirement": 1,
                    "rootAffinity": ["Fire"],
                    "basePower": 32.0,
                    "riskTags": [],
                    "description": "Generated candidate technique"
                }),
                EntityType::Character => json!({
                    "characterId": format!("char_{}_{}", hint, idx + 1),
                    "name": format!("{} Character {}", hint, idx + 1),
                    "realm": "Qi Condensation",
                    "personalityTags": ["cautious"],
                    "relationshipEdges": [],
                    "knownTechniques": []
                }),
                EntityType::MapNode => json!({
                    "nodeId": format!("map_{}_{}", hint, idx + 1),
                    "name": format!("{} Region {}", hint, idx + 1),
                    "nodeType": "wilderness",
                    "dangerTier": 3,
                    "auraDensity": 0.8,
                    "factionControl": "neutral",
                    "connectedNodes": []
                }),
                EntityType::Item => json!({
                    "itemId": format!("item_{}_{}", hint, idx + 1),
                    "name": format!("{} Item {}", hint, idx + 1),
                    "itemType": "artifact",
                    "qualityTier": 2,
                    "description": "Generated candidate item"
                }),
            };
            EntityCandidateRequest {
                entity_type: input.entity_type,
                payload,
                source_trace_id: Some(format!("candidate-gen-{}", idx + 1)),
            }
        })
        .collect();
    Ok(candidates)
}

#[tauri::command]
pub async fn commit_entities(input: CommitEntitiesInput) -> Result<CommitEntitiesResponse, String> {
    if input.world_id.trim().is_empty() || input.run_id.trim().is_empty() {
        return Err("world_id and run_id must be non-empty".to_string());
    }

    let mut committed = Vec::new();
    let mut rejected = Vec::new();
    let mut store = entity_store()
        .lock()
        .map_err(|_| "failed to lock entity store".to_string())?;
    let mut memory = memory_layers()
        .lock()
        .map_err(|_| "failed to lock memory layers".to_string())?;

    for candidate in &input.candidates {
        let resolved = resolve_candidate(candidate);
        if matches!(
            resolved.validation_report.status,
            ValidationStatus::Rejected
        ) {
            rejected.push(resolved);
            continue;
        }

        let stored = StoredEntity {
            world_id: input.world_id.clone(),
            run_id: input.run_id.clone(),
            entity_id: resolved.entity_id.clone(),
            entity_type: resolved.entity_type,
            payload: resolved.payload.clone(),
            updated_at: 0,
        };
        store.upsert(stored.clone());
        committed.push(stored.clone());

        memory.upsert_world_fact(crate::memory_layers::WorldFact {
            fact_id: format!("fact_{}_{}", input.run_id, resolved.entity_id),
            subject: resolved.entity_id,
            predicate: "defined_as".to_string(),
            object: format!("{:?}", resolved.entity_type),
        });
    }

    Ok(CommitEntitiesResponse {
        committed,
        rejected,
    })
}

#[tauri::command]
pub async fn query_entities(query: EntityQuery) -> Result<Vec<StoredEntity>, String> {
    if query.world_id.trim().is_empty() || query.run_id.trim().is_empty() {
        return Err("world_id and run_id must be non-empty".to_string());
    }
    let store = entity_store()
        .lock()
        .map_err(|_| "failed to lock entity store".to_string())?;
    Ok(store.list_by_query(&query))
}

#[tauri::command]
pub async fn build_context_bundle_command(
    input: ContextBuildInput,
) -> Result<ContextBundle, String> {
    let store = entity_store()
        .lock()
        .map_err(|_| "failed to lock entity store".to_string())?;
    let memory = memory_layers()
        .lock()
        .map_err(|_| "failed to lock memory layers".to_string())?;
    Ok(build_context_bundle(&store, &memory, &input))
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_log::{EventImportance, GameEvent};
    use crate::models::{CultivationRealm, Element, Grade, SpiritualRoot};
    use crate::script::{InitialState, Location, ScriptType, WorldSetting};
    use tempfile::tempdir;

    fn create_test_script() -> Script {
        let mut world_setting = WorldSetting::new();
        world_setting.cultivation_realms = vec![
            CultivationRealm::new("Qi Condensation".to_string(), 1, 0, 1.0),
            CultivationRealm::new("Foundation Establishment".to_string(), 2, 0, 2.0),
        ];
        world_setting.locations = vec![Location {
            id: "sect".to_string(),
            name: "Azure Cloud Sect".to_string(),
            description: "A peaceful cultivation sect".to_string(),
            spiritual_energy: 1.0,
        }];

        let initial_state = InitialState {
            player_name: "Test Player".to_string(),
            player_spiritual_root: SpiritualRoot {
                element: Element::Fire,
                grade: Grade::Heavenly,
                affinity: 0.9,
                elements: Vec::new(),
            },
            starting_location: "sect".to_string(),
            starting_age: 16,
        };

        Script::new(
            "test".to_string(),
            "Test Script".to_string(),
            ScriptType::Custom,
            world_setting,
            initial_state,
        )
    }

    #[test]
    fn test_command_logic_initialize_game() {
        let mut engine = GameEngine::new();
        let script = create_test_script();

        let result = engine.initialize_game(script);

        assert!(result.is_ok());
        let game_state = result.unwrap();
        assert_eq!(game_state.player.name, "Test Player");
        assert_eq!(game_state.player.location, "sect");
    }

    #[test]
    fn test_command_logic_get_game_state_before_initialization() {
        let engine = GameEngine::new();
        let result = engine.get_current_state();

        assert!(result.is_err());
    }

    #[test]
    fn test_error_response_conversion() {
        let error = anyhow::anyhow!("Test error");
        let response: ErrorResponse = error.into();

        assert_eq!(response.error, "Test error");
    }

    #[test]
    fn test_validate_slot_id_bounds() {
        assert!(validate_slot_id(1).is_ok());
        assert!(validate_slot_id(99).is_ok());
        assert!(validate_slot_id(0).is_err());
        assert!(validate_slot_id(120).is_err());
    }

    #[test]
    fn test_validate_file_path_extension() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("sample.json");
        std::fs::write(&file, "{}").unwrap();
        let ok = validate_file_path(file.to_str().unwrap(), &["json"]);
        assert!(ok.is_ok());
        let bad = validate_file_path(file.to_str().unwrap(), &["txt"]);
        assert!(bad.is_err());
    }

    #[test]
    fn test_validate_endpoint_scheme() {
        assert!(validate_endpoint("https://example.com").is_ok());
        assert!(validate_endpoint("http://localhost").is_ok());
        assert!(validate_endpoint("ftp://example.com").is_err());
        assert!(validate_endpoint("example.com").is_err());
    }

    #[test]
    fn test_validate_llm_config_requires_api_key_for_remote() {
        let input = LLMConfigInput {
            endpoint: "https://api.example.com/v1".to_string(),
            api_key: "".to_string(),
            model: "test".to_string(),
            max_tokens: 128,
            temperature: 0.7,
        };
        assert!(validate_llm_config_input(&input).is_err());
    }

    #[test]
    fn test_validate_llm_config_allows_local_without_key() {
        let input = LLMConfigInput {
            endpoint: "http://localhost:8000/v1".to_string(),
            api_key: "".to_string(),
            model: "test".to_string(),
            max_tokens: 128,
            temperature: 0.7,
        };
        assert!(validate_llm_config_input(&input).is_ok());
    }

    #[test]
    fn test_merge_llm_config_input_reuses_existing_api_key() {
        let input = LLMConfigInput {
            endpoint: "https://api.example.com/v1".to_string(),
            api_key: "".to_string(),
            model: "test".to_string(),
            max_tokens: 128,
            temperature: 0.7,
        };
        let existing = Some(LLMConfig {
            endpoint: "https://api.example.com/v1".to_string(),
            api_key: "saved-secret".to_string(),
            model: "saved-model".to_string(),
            max_tokens: 256,
            temperature: 0.5,
        });

        let merged = merge_llm_config_input_with_existing(&input, existing);

        assert_eq!(merged.api_key, "saved-secret");
        assert_eq!(merged.endpoint, input.endpoint);
        assert_eq!(merged.model, input.model);
    }

    #[test]
    fn test_merge_llm_config_input_prefers_new_api_key() {
        let input = LLMConfigInput {
            endpoint: "https://api.example.com/v1".to_string(),
            api_key: "new-secret".to_string(),
            model: "test".to_string(),
            max_tokens: 128,
            temperature: 0.7,
        };
        let existing = Some(LLMConfig {
            endpoint: "https://api.example.com/v1".to_string(),
            api_key: "saved-secret".to_string(),
            model: "saved-model".to_string(),
            max_tokens: 256,
            temperature: 0.5,
        });

        let merged = merge_llm_config_input_with_existing(&input, existing);

        assert_eq!(merged.api_key, "new-secret");
    }

    #[test]
    fn test_validate_output_path_extension() {
        let dir = tempdir().unwrap();
        let out = dir.path().join("novel.txt");
        let ok = validate_output_path(out.to_str().unwrap(), &["txt"]);
        assert!(ok.is_ok());
        let bad = dir.path().join("novel.md");
        let err = validate_output_path(bad.to_str().unwrap(), &["txt"]);
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn test_generate_novel_command_logic() {
        let events = vec![
            GameEvent {
                id: 1,
                timestamp: 1,
                event_type: std::sync::Arc::from("cultivation"),
                description: std::sync::Arc::from("Player cultivated"),
                importance: EventImportance::Normal,
            },
            GameEvent {
                id: 2,
                timestamp: 2,
                event_type: std::sync::Arc::from("combat"),
                description: std::sync::Arc::from("Player won duel"),
                importance: EventImportance::Important,
            },
        ];

        let novel = generate_novel_from_events("Test Novel", &events)
            .await
            .unwrap();
        assert_eq!(novel.title, "Test Novel");
        assert_eq!(novel.total_events, 2);
        assert!(!novel.chapters.is_empty());
    }

    #[test]
    fn test_export_novel_command_logic() {
        let novel = Novel {
            title: "Exported Novel".to_string(),
            chapters: vec![crate::novel_generator::Chapter {
                index: 1,
                title: "Start".to_string(),
                content: "A new journey starts.".to_string(),
                source_event_ids: vec![1],
            }],
            toc: vec![crate::novel_generator::TocEntry {
                index: 1,
                title: "Start".to_string(),
                summary: "A new journey starts.".to_string(),
                source_event_count: 1,
            }],
            total_events: 1,
        };

        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("novel_out.txt");
        let result = export_novel_to_path(&novel, output.to_str().unwrap());
        assert!(result.is_ok());
        assert!(output.exists());
    }

    #[test]
    fn test_parse_generation_timing_sample() {
        let diag = "选项来源：rule_fallback；耗时(ms)：total=12,plot_gen=7,option_gen=3";
        let parsed = parse_generation_timing_sample(diag).unwrap();
        assert_eq!(
            parsed,
            GenerationTimingSample {
                total_ms: 12,
                plot_gen_ms: 7,
                option_gen_ms: 3,
            }
        );
    }

    #[test]
    fn test_summarize_generation_timing_diagnostics() {
        let input = vec![
            "耗时(ms)：total=10,plot_gen=6,option_gen=2".to_string(),
            "上下文注入：facts=2；耗时(ms)：total=14,plot_gen=8,option_gen=4".to_string(),
            "耗时(ms)：total=13,plot_gen=7,option_gen=3".to_string(),
            "invalid-line".to_string(),
        ];

        let summary = summarize_generation_timing_diagnostics(&input).unwrap();
        assert_eq!(summary.sample_count, 3);
        assert_eq!(summary.total_p50_ms, 13);
        assert_eq!(summary.total_p95_ms, 14);
        assert_eq!(summary.plot_gen_p95_ms, 8);
        assert_eq!(summary.option_gen_p95_ms, 4);
    }

    #[test]
    fn test_has_consistency_issue() {
        let mut report = ConsistencyReport::default();
        report
            .issues
            .push(crate::plot_consistency::ConsistencyIssue {
                level: crate::plot_consistency::IssueLevel::Warning,
                code: "chapter_goal_weak",
                message: "goal weak".to_string(),
            });
        assert!(has_consistency_issue(&report, "chapter_goal_weak"));
        assert!(!has_consistency_issue(&report, "duplicate_segment"));
    }

    #[test]
    fn test_diagnostics_used_preset_fallback_detects_marker() {
        assert!(diagnostics_used_preset_fallback(Some(
            "回退：LLM 结构化剧情生成失败；纯文本续写也失败，已使用预设文本",
        )));
        assert!(!diagnostics_used_preset_fallback(Some(
            "回退：仅降级为纯文本续写"
        )));
        assert!(!diagnostics_used_preset_fallback(None));
    }

    #[test]
    fn test_summarize_generation_failure_diagnostics() {
        let input = vec![
            "链路：structured_ok；阶段耗时(ms)：structured=900,plain=0,skeleton=0,micro=0；双通道生成：fallback(plot_engine_only,turn_update_ms=2000,reason=turn update request timeout)".to_string(),
            "回退：LLM 结构化剧情生成超时；骨架生成失败(骨架生成超时)；纯文本续写失败(纯文本续写请求超时)；轻量续写失败(轻量续写请求超时)；纯文本续写也失败，已使用预设文本；链路：preset_fallback；阶段耗时(ms)：structured=1000,plain=1000,skeleton=1000,micro=1000；双通道生成：fallback(plot_engine_only,turn_update_ms=1500,reason=turn update output is not valid JSON object)".to_string(),
            "链路：plain_ok；阶段耗时(ms)：structured=1200,plain=800,skeleton=0,micro=0；本次选项续写未获取到 LLM 剧情文本".to_string(),
        ];
        let summary = summarize_generation_failure_diagnostics(&input).unwrap();
        assert_eq!(summary.sample_count, 3);
        assert_eq!(summary.structured_ok_count, 1);
        assert_eq!(summary.plain_ok_count, 1);
        assert_eq!(summary.preset_fallback_count, 1);
        assert_eq!(summary.turn_update_fallback_count, 2);
        assert_eq!(summary.option_llm_blocked_count, 1);
        assert!(!summary.top_reasons.is_empty());
        assert!(
            summary.top_reasons[0].stage == "turn_update"
                || summary.top_reasons[0].stage == "preset_fallback"
        );
    }

    #[test]
    fn test_effective_consistency_report_removes_waiting_issue_when_options_recovered() {
        let mut report = ConsistencyReport::default();
        report
            .issues
            .push(crate::plot_consistency::ConsistencyIssue {
                level: crate::plot_consistency::IssueLevel::Warning,
                code: "waiting_without_options",
                message: "waiting without options".to_string(),
            });
        report
            .issues
            .push(crate::plot_consistency::ConsistencyIssue {
                level: crate::plot_consistency::IssueLevel::Warning,
                code: "chapter_goal_weak",
                message: "goal weak".to_string(),
            });

        let effective =
            effective_consistency_report_after_option_resolution(report, true, false, 3);
        assert!(!has_consistency_issue(
            &effective,
            "waiting_without_options"
        ));
        assert!(has_consistency_issue(&effective, "chapter_goal_weak"));
    }

    #[test]
    fn test_effective_consistency_report_keeps_waiting_issue_when_options_missing() {
        let mut report = ConsistencyReport::default();
        report
            .issues
            .push(crate::plot_consistency::ConsistencyIssue {
                level: crate::plot_consistency::IssueLevel::Warning,
                code: "waiting_without_options",
                message: "waiting without options".to_string(),
            });

        let effective =
            effective_consistency_report_after_option_resolution(report, true, false, 0);
        assert!(has_consistency_issue(&effective, "waiting_without_options"));
    }

    #[test]
    fn test_chapter_goal_regeneration_hint_contains_goal_anchor() {
        let hint = chapter_goal_regeneration_hint(2);
        assert!(hint.contains("章节目标重生成约束"));
        assert!(hint.contains("资源变化"));
    }

    #[test]
    fn test_is_hollow_expression_detects_filler_text() {
        let text = "一时间气氛凝重，你不由得心中一凛，似乎四周都安静了下来。";
        assert!(is_hollow_expression(text));
    }

    #[test]
    fn test_is_hollow_expression_allows_dense_text() {
        let text = "山风掠过石阶，你踏前半步催动灵力，心神却在师门旧训与眼前杀机间迅速权衡。";
        assert!(!is_hollow_expression(text));
    }

    #[test]
    fn test_narrative_dimension_coverage_counts_multiple_dimensions() {
        let text = "冷风卷过石阶，你踏步逼近，心中迟疑却仍决定先试探一剑。";
        assert!(narrative_dimension_coverage(text) >= 2);
    }

    #[test]
    fn test_narrative_density_and_pacing_hint_contains_stage_and_templates() {
        let hint = narrative_density_and_pacing_hint(2);
        assert!(hint.contains("转折"));
        assert!(hint.contains("片段模板"));
        assert!(hint.contains("内在状态"));
    }

    #[test]
    fn test_build_combat_explanation_contains_dominant_and_reversal() {
        let action_result = crate::numerical_system::ActionResult {
            success: false,
            description: "战斗受阻".to_string(),
            stat_changes: vec![],
            events: vec![],
        };
        let explanation = build_combat_explanation(
            &action_result,
            3,
            4200,
            Some("combat_power above maximum for realm 3"),
            Some("强攻压制（高压进攻意图）"),
        );
        assert!(explanation.summary.contains("主导因素="));
        assert!(explanation.summary.contains("反转因素="));
        assert!(explanation
            .reversal_factors
            .iter()
            .any(|item| item.contains("数值守门裁决")));
    }

    #[test]
    fn test_apply_combat_aftermath_updates_persistent_status() {
        let mut state = crate::game_state::GameState {
            script: crate::script::Script::new(
                "id".to_string(),
                "name".to_string(),
                crate::script::ScriptType::Custom,
                crate::script::WorldSetting::new(),
                crate::script::InitialState {
                    player_name: "p".to_string(),
                    player_spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    starting_location: "sect".to_string(),
                    starting_age: 16,
                },
            ),
            player: crate::game_state::Character::new(
                "player".to_string(),
                "Tester".to_string(),
                crate::models::CharacterStats {
                    spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    cultivation_realm: crate::models::CultivationRealm::new(
                        "Qi".to_string(),
                        1,
                        0,
                        1.0,
                    ),
                    techniques: vec![],
                    lifespan: crate::models::Lifespan {
                        current_age: 16,
                        max_age: 100,
                        realm_bonus: 0,
                    },
                    combat_power: 120,
                },
                "sect".to_string(),
            ),
            world_state: crate::game_state::WorldState::new(),
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
        };

        let success_summary =
            apply_combat_aftermath(&mut state, true, Some(CombatStrategy::Aggressive));
        assert!(success_summary.contains("战后状态更新"));
        assert_eq!(state.player.combat_status.reputation, 2);
        assert_eq!(state.player.combat_status.enmity, 2);

        let fail_summary =
            apply_combat_aftermath(&mut state, false, Some(CombatStrategy::Survival));
        assert!(fail_summary.contains("战后状态更新"));
        assert_eq!(state.player.combat_status.reputation, 1);
        assert_eq!(state.player.combat_status.enmity, 3);
        assert_eq!(state.player.combat_status.injury_level, 1);
    }

    #[test]
    fn test_push_growth_log_keeps_recent_entries() {
        let mut state = crate::game_state::GameState {
            script: crate::script::Script::new(
                "id".to_string(),
                "name".to_string(),
                crate::script::ScriptType::Custom,
                crate::script::WorldSetting::new(),
                crate::script::InitialState {
                    player_name: "p".to_string(),
                    player_spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    starting_location: "sect".to_string(),
                    starting_age: 16,
                },
            ),
            player: crate::game_state::Character::new(
                "player".to_string(),
                "Tester".to_string(),
                crate::models::CharacterStats {
                    spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    cultivation_realm: crate::models::CultivationRealm::new(
                        "Qi".to_string(),
                        1,
                        0,
                        1.0,
                    ),
                    techniques: vec![],
                    lifespan: crate::models::Lifespan {
                        current_age: 16,
                        max_age: 100,
                        realm_bonus: 0,
                    },
                    combat_power: 120,
                },
                "sect".to_string(),
            ),
            world_state: crate::game_state::WorldState::new(),
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
        };

        for i in 0..260 {
            push_growth_log(&mut state, format!("entry-{}", i));
        }
        assert_eq!(state.player.growth_log.len(), 240);
        assert_eq!(
            state.player.growth_log.first().map(String::as_str),
            Some("entry-20")
        );
        assert_eq!(
            state.player.growth_log.last().map(String::as_str),
            Some("entry-259")
        );
    }

    #[test]
    fn test_evaluate_technique_semantic_modifier_affinity_and_risk() {
        let stats = crate::models::CharacterStats {
            spiritual_root: crate::models::SpiritualRoot {
                element: crate::models::Element::Fire,
                elements: vec![crate::models::Element::Fire],
                grade: crate::models::Grade::Heavenly,
                affinity: 0.8,
            },
            cultivation_realm: crate::models::CultivationRealm::new("Qi".to_string(), 2, 0, 1.0),
            techniques: vec!["赤炎诀".to_string(), "禁术爆燃".to_string()],
            lifespan: crate::models::Lifespan {
                current_age: 18,
                max_age: 100,
                realm_bonus: 0,
            },
            combat_power: 200,
        };
        let (delta, reasons, risk) = evaluate_technique_semantic_modifier(&stats);
        assert!(delta > 0);
        assert!(!reasons.is_empty());
        assert!(risk);
    }

    #[test]
    fn test_infer_technique_semantics_contains_core_dimensions() {
        let semantic = infer_technique_semantics("元婴禁术爆炎剑");
        assert!(semantic.type_tags.iter().any(|t| t == "sword"));
        assert!(semantic.trait_tags.iter().any(|t| t == "fire"));
        assert!(semantic
            .condition_tags
            .iter()
            .any(|t| t.contains("realm>=")));
        assert!(semantic.risk_level > 0);
        assert!(semantic.required_realm_level >= 4);
    }

    #[test]
    fn test_is_high_risk_technique_name_detects_cn_and_en_keywords() {
        assert!(is_high_risk_technique_name("禁术爆燃"));
        assert!(is_high_risk_technique_name("forbidden flame"));
        assert!(!is_high_risk_technique_name("清心诀"));
    }

    #[test]
    fn test_apply_breakthrough_failure_consequences_updates_qi_deviation_and_injury() {
        let mut state = crate::game_state::GameState {
            script: crate::script::Script::new(
                "id".to_string(),
                "name".to_string(),
                crate::script::ScriptType::Custom,
                crate::script::WorldSetting::new(),
                crate::script::InitialState {
                    player_name: "p".to_string(),
                    player_spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    starting_location: "sect".to_string(),
                    starting_age: 16,
                },
            ),
            player: crate::game_state::Character::new(
                "player".to_string(),
                "Tester".to_string(),
                crate::models::CharacterStats {
                    spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    cultivation_realm: crate::models::CultivationRealm::new(
                        "Qi".to_string(),
                        1,
                        2,
                        1.0,
                    ),
                    techniques: vec!["禁术爆燃".to_string()],
                    lifespan: crate::models::Lifespan {
                        current_age: 16,
                        max_age: 100,
                        realm_bonus: 0,
                    },
                    combat_power: 120,
                },
                "sect".to_string(),
            ),
            world_state: crate::game_state::WorldState::new(),
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
        };
        let mut action_result = crate::numerical_system::ActionResult {
            success: false,
            description: "突破失败".to_string(),
            stat_changes: vec![],
            events: vec![],
        };

        apply_breakthrough_failure_consequences(&mut state, &mut action_result, true);

        assert_eq!(state.player.combat_status.qi_deviation, 2);
        assert_eq!(state.player.combat_status.injury_level, 1);
        assert!(action_result
            .events
            .iter()
            .any(|e| e.contains("突破失败后果：气机紊乱+2")));
        assert!(action_result
            .events
            .iter()
            .any(|e| e.contains("走火入魔征兆")));
        assert!(state
            .player
            .growth_log
            .iter()
            .any(|e| e.contains("突破受挫：气机紊乱")));
    }

    #[test]
    fn test_breakthrough_blocked_by_qi_deviation_threshold() {
        assert!(!breakthrough_blocked_by_qi_deviation(7));
        assert!(breakthrough_blocked_by_qi_deviation(8));
        assert!(breakthrough_blocked_by_qi_deviation(10));
    }

    #[test]
    fn test_apply_cultivation_side_effects_triggers_backlash_for_high_risk() {
        let mut script = create_test_script();
        script
            .world_setting
            .locations
            .push(crate::script::Location {
                id: "cave".to_string(),
                name: "灵息洞".to_string(),
                description: "修炼洞府".to_string(),
                spiritual_energy: 0.6,
            });
        let mut state = crate::game_state::GameState {
            player: crate::game_state::Character::new(
                "player".to_string(),
                "Tester".to_string(),
                crate::models::CharacterStats {
                    spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    cultivation_realm: crate::models::CultivationRealm::new(
                        "Qi".to_string(),
                        1,
                        0,
                        1.0,
                    ),
                    techniques: vec!["禁术爆燃".to_string()],
                    lifespan: crate::models::Lifespan {
                        current_age: 16,
                        max_age: 100,
                        realm_bonus: 0,
                    },
                    combat_power: 120,
                },
                "sect".to_string(),
            ),
            world_state: crate::game_state::WorldState::from_script(&script),
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
            script,
        };
        state.game_time.total_days = 2; // seed = 3 -> not backlash, set even with techniques len 1
        let mut result = crate::numerical_system::ActionResult {
            success: true,
            description: "修炼".to_string(),
            stat_changes: vec![],
            events: vec![],
        };
        state.game_time.total_days = 1; // seed = 2, backlash
        let note = apply_cultivation_side_effects(&mut state, &mut result);
        assert!(note.is_some());
        assert!(state.player.combat_status.qi_deviation >= 1);
        assert!(result.events.iter().any(|e| e.contains("修炼反噬")));
    }

    #[test]
    fn test_apply_cultivation_side_effects_can_trigger_insight() {
        let script = create_test_script();
        let mut state = crate::game_state::GameState {
            player: crate::game_state::Character::new(
                "player".to_string(),
                "Tester".to_string(),
                crate::models::CharacterStats {
                    spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.9,
                    },
                    cultivation_realm: crate::models::CultivationRealm::new(
                        "Qi".to_string(),
                        1,
                        0,
                        1.0,
                    ),
                    techniques: vec!["赤炎诀".to_string(), "青霜剑诀".to_string()],
                    lifespan: crate::models::Lifespan {
                        current_age: 16,
                        max_age: 100,
                        realm_bonus: 0,
                    },
                    combat_power: 120,
                },
                "sect".to_string(),
            ),
            world_state: crate::game_state::WorldState::from_script(&script),
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
            script,
        };
        state.game_time.total_days = 1; // seed = 3 -> insight
        let mut result = crate::numerical_system::ActionResult {
            success: true,
            description: "修炼".to_string(),
            stat_changes: vec![],
            events: vec![],
        };
        let old_sub = state.player.stats.cultivation_realm.sub_level;
        let note = apply_cultivation_side_effects(&mut state, &mut result);
        assert!(note.is_some());
        assert!(state.player.stats.cultivation_realm.sub_level >= old_sub);
        assert!(result.events.iter().any(|e| e.contains("顿悟")));
    }

    #[test]
    fn test_apply_travel_and_encounter_moves_location_and_advances_time() {
        let mut world_setting = crate::script::WorldSetting::new();
        world_setting.locations = vec![
            crate::script::Location {
                id: "sect".to_string(),
                name: "青云宗".to_string(),
                description: "宗门驻地".to_string(),
                spiritual_energy: 0.3,
            },
            crate::script::Location {
                id: "valley".to_string(),
                name: "幽风谷".to_string(),
                description: "灵压紊乱".to_string(),
                spiritual_energy: 0.3,
            },
        ];
        let script = crate::script::Script::new(
            "id".to_string(),
            "name".to_string(),
            crate::script::ScriptType::Custom,
            world_setting.clone(),
            crate::script::InitialState {
                player_name: "p".to_string(),
                player_spiritual_root: crate::models::SpiritualRoot {
                    element: crate::models::Element::Fire,
                    elements: vec![crate::models::Element::Fire],
                    grade: crate::models::Grade::Heavenly,
                    affinity: 0.8,
                },
                starting_location: "sect".to_string(),
                starting_age: 16,
            },
        );
        let mut state = crate::game_state::GameState {
            player: crate::game_state::Character::new(
                "player".to_string(),
                "Tester".to_string(),
                crate::models::CharacterStats {
                    spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    cultivation_realm: crate::models::CultivationRealm::new(
                        "Qi".to_string(),
                        1,
                        0,
                        1.0,
                    ),
                    techniques: vec![],
                    lifespan: crate::models::Lifespan {
                        current_age: 16,
                        max_age: 100,
                        realm_bonus: 0,
                    },
                    combat_power: 120,
                },
                "sect".to_string(),
            ),
            world_state: crate::game_state::WorldState::from_script(&script),
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
            script,
        };
        let mut plot_state = crate::plot_engine::PlotState::new(crate::plot_engine::Scene::new(
            "s1".to_string(),
            "scene".to_string(),
            "desc".to_string(),
            "sect".to_string(),
        ));

        let result = apply_travel_and_encounter(&mut state, &mut plot_state, "valley").unwrap();

        assert_eq!(state.player.location, "valley");
        assert_eq!(plot_state.current_scene.location, "valley");
        assert_eq!(state.game_time.total_days, 2);
        assert!(result.0.contains("前往"));
        assert!(state
            .player
            .growth_log
            .iter()
            .any(|entry| entry.contains("行程变更")));
    }

    #[test]
    fn test_apply_travel_and_encounter_supports_segmented_travel_days() {
        let mut world_setting = crate::script::WorldSetting::new();
        world_setting.locations = vec![
            crate::script::Location {
                id: "sect".to_string(),
                name: "青云宗".to_string(),
                description: "宗门驻地".to_string(),
                spiritual_energy: 0.2,
            },
            crate::script::Location {
                id: "mid".to_string(),
                name: "过渡点".to_string(),
                description: "中继区域".to_string(),
                spiritual_energy: 0.45,
            },
            crate::script::Location {
                id: "valley".to_string(),
                name: "幽风谷".to_string(),
                description: "目标区域".to_string(),
                spiritual_energy: 0.55,
            },
            crate::script::Location {
                id: "far".to_string(),
                name: "远域".to_string(),
                description: "高风险区域".to_string(),
                spiritual_energy: 0.95,
            },
        ];
        let script = crate::script::Script::new(
            "id".to_string(),
            "name".to_string(),
            crate::script::ScriptType::Custom,
            world_setting.clone(),
            crate::script::InitialState {
                player_name: "p".to_string(),
                player_spiritual_root: crate::models::SpiritualRoot {
                    element: crate::models::Element::Fire,
                    elements: vec![crate::models::Element::Fire],
                    grade: crate::models::Grade::Heavenly,
                    affinity: 0.8,
                },
                starting_location: "sect".to_string(),
                starting_age: 16,
            },
        );
        let mut state = crate::game_state::GameState {
            player: crate::game_state::Character::new(
                "player".to_string(),
                "Tester".to_string(),
                crate::models::CharacterStats {
                    spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    cultivation_realm: crate::models::CultivationRealm::new(
                        "Qi".to_string(),
                        1,
                        0,
                        1.0,
                    ),
                    techniques: vec![],
                    lifespan: crate::models::Lifespan {
                        current_age: 16,
                        max_age: 100,
                        realm_bonus: 0,
                    },
                    combat_power: 120,
                },
                "sect".to_string(),
            ),
            world_state: crate::game_state::WorldState::from_script(&script),
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
            script,
        };
        let mut plot_state = crate::plot_engine::PlotState::new(crate::plot_engine::Scene::new(
            "s1".to_string(),
            "scene".to_string(),
            "desc".to_string(),
            "sect".to_string(),
        ));

        let result = apply_travel_and_encounter(&mut state, &mut plot_state, "valley").unwrap();
        assert_eq!(state.player.location, "valley");
        assert_eq!(state.game_time.total_days, 3);
        assert!(result.0.contains("耗时2日"));
        assert!(result.0.contains("建议分段"));
    }

    #[test]
    fn test_apply_travel_and_encounter_rejects_unknown_location() {
        let script = create_test_script();
        let mut state = crate::game_state::GameState {
            player: crate::game_state::Character::new(
                "player".to_string(),
                "Tester".to_string(),
                crate::models::CharacterStats {
                    spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    cultivation_realm: crate::models::CultivationRealm::new(
                        "Qi".to_string(),
                        1,
                        0,
                        1.0,
                    ),
                    techniques: vec![],
                    lifespan: crate::models::Lifespan {
                        current_age: 16,
                        max_age: 100,
                        realm_bonus: 0,
                    },
                    combat_power: 120,
                },
                "sect".to_string(),
            ),
            world_state: crate::game_state::WorldState::from_script(&script),
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
            script,
        };
        let mut plot_state = crate::plot_engine::PlotState::new(crate::plot_engine::Scene::new(
            "s1".to_string(),
            "scene".to_string(),
            "desc".to_string(),
            "sect".to_string(),
        ));

        let err = apply_travel_and_encounter(&mut state, &mut plot_state, "unknown")
            .expect_err("unknown location should be rejected");
        assert!(err.contains("目标地点不存在"));
    }

    #[test]
    fn test_apply_travel_and_encounter_rejects_unreachable_location() {
        let mut world_setting = crate::script::WorldSetting::new();
        world_setting.locations = vec![
            crate::script::Location {
                id: "sect".to_string(),
                name: "青云宗".to_string(),
                description: "宗门驻地".to_string(),
                spiritual_energy: 0.2,
            },
            crate::script::Location {
                id: "abyss".to_string(),
                name: "魔渊".to_string(),
                description: "极高灵压".to_string(),
                spiritual_energy: 1.5,
            },
        ];
        let script = crate::script::Script::new(
            "id".to_string(),
            "name".to_string(),
            crate::script::ScriptType::Custom,
            world_setting.clone(),
            crate::script::InitialState {
                player_name: "p".to_string(),
                player_spiritual_root: crate::models::SpiritualRoot {
                    element: crate::models::Element::Fire,
                    elements: vec![crate::models::Element::Fire],
                    grade: crate::models::Grade::Triple,
                    affinity: 0.3,
                },
                starting_location: "sect".to_string(),
                starting_age: 16,
            },
        );

        let mut state = crate::game_state::GameState {
            player: crate::game_state::Character::new(
                "player".to_string(),
                "Tester".to_string(),
                crate::models::CharacterStats {
                    spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Triple,
                        affinity: 0.3,
                    },
                    cultivation_realm: crate::models::CultivationRealm::new(
                        "Qi".to_string(),
                        1,
                        0,
                        1.0,
                    ),
                    techniques: vec![],
                    lifespan: crate::models::Lifespan {
                        current_age: 16,
                        max_age: 100,
                        realm_bonus: 0,
                    },
                    combat_power: 80,
                },
                "sect".to_string(),
            ),
            world_state: crate::game_state::WorldState::from_script(&script),
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
            script,
        };
        state.player.combat_status.injury_level = 8;
        state.player.combat_status.qi_deviation = 8;
        let mut plot_state = crate::plot_engine::PlotState::new(crate::plot_engine::Scene::new(
            "s1".to_string(),
            "scene".to_string(),
            "desc".to_string(),
            "sect".to_string(),
        ));

        let err = apply_travel_and_encounter(&mut state, &mut plot_state, "abyss")
            .expect_err("unreachable location should be rejected");
        assert!(err.contains("无法前往"));
    }

    #[test]
    fn test_cultivation_gain_multiplier_from_location_is_monotonic() {
        let low = cultivation_gain_multiplier_from_location(Some(0.1));
        let mid = cultivation_gain_multiplier_from_location(Some(0.5));
        let high = cultivation_gain_multiplier_from_location(Some(1.0));
        assert!(low <= mid);
        assert!(mid <= high);
        assert!(low >= 0.6 && high <= 1.6);
    }

    #[test]
    fn test_location_spiritual_energy_reads_current_node() {
        let mut world_setting = crate::script::WorldSetting::new();
        world_setting.locations = vec![crate::script::Location {
            id: "cave".to_string(),
            name: "灵息洞".to_string(),
            description: "灵气汇聚".to_string(),
            spiritual_energy: 0.95,
        }];
        let script = crate::script::Script::new(
            "id".to_string(),
            "name".to_string(),
            crate::script::ScriptType::Custom,
            world_setting.clone(),
            crate::script::InitialState {
                player_name: "p".to_string(),
                player_spiritual_root: crate::models::SpiritualRoot {
                    element: crate::models::Element::Fire,
                    elements: vec![crate::models::Element::Fire],
                    grade: crate::models::Grade::Heavenly,
                    affinity: 0.8,
                },
                starting_location: "cave".to_string(),
                starting_age: 16,
            },
        );
        let state = crate::game_state::GameState {
            player: crate::game_state::Character::new(
                "player".to_string(),
                "Tester".to_string(),
                crate::models::CharacterStats {
                    spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    cultivation_realm: crate::models::CultivationRealm::new(
                        "Qi".to_string(),
                        1,
                        0,
                        1.0,
                    ),
                    techniques: vec![],
                    lifespan: crate::models::Lifespan {
                        current_age: 16,
                        max_age: 100,
                        realm_bonus: 0,
                    },
                    combat_power: 120,
                },
                "cave".to_string(),
            ),
            world_state: crate::game_state::WorldState::from_script(&script),
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
            script,
        };
        assert_eq!(location_spiritual_energy(&state), Some(0.95));
    }

    #[test]
    fn test_select_encounter_text_uses_risk_buckets() {
        let low = select_encounter_text(0.2, 1);
        let mid = select_encounter_text(0.6, 1);
        let high = select_encounter_text(0.95, 1);
        assert!(low.contains("散修") || low.contains("灵兽") || low.contains("路匪"));
        assert!(mid.contains("阵法") || mid.contains("巡逻") || mid.contains("乱流"));
        assert!(high.contains("魔息") || high.contains("妖潮") || high.contains("威压"));
    }

    #[test]
    fn test_choose_combat_strategy_prefers_survival_when_status_is_bad() {
        let status = crate::game_state::CombatAftermathStatus {
            injury_level: 7,
            reputation: 0,
            enmity: 0,
            qi_deviation: 6,
        };
        let (strategy, reason) = choose_combat_strategy(&status, "强攻压制");
        assert_eq!(strategy, CombatStrategy::Survival);
        assert!(reason.contains("伤势/气机"));
    }

    #[test]
    fn test_strategy_power_modifier_pct_matches_expected_bias() {
        let status = crate::game_state::CombatAftermathStatus {
            injury_level: 1,
            reputation: 0,
            enmity: 2,
            qi_deviation: 1,
        };
        let aggressive = strategy_power_modifier_pct(CombatStrategy::Aggressive, &status);
        let survival = strategy_power_modifier_pct(CombatStrategy::Survival, &status);
        assert!(aggressive > 0);
        assert!(survival < 0);
    }

    #[test]
    fn test_evaluate_combat_hard_constraints_rejects_large_realm_gap() {
        let script = create_test_script();
        let mut state = crate::game_state::GameState {
            player: crate::game_state::Character::new(
                "player".to_string(),
                "Tester".to_string(),
                crate::models::CharacterStats {
                    spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    cultivation_realm: crate::models::CultivationRealm::new(
                        "Qi".to_string(),
                        1,
                        0,
                        1.0,
                    ),
                    techniques: vec!["青霜剑诀".to_string()],
                    lifespan: crate::models::Lifespan {
                        current_age: 16,
                        max_age: 100,
                        realm_bonus: 0,
                    },
                    combat_power: 120,
                },
                "sect".to_string(),
            ),
            world_state: crate::game_state::WorldState::from_script(&script),
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
            script,
        };
        state.player.equipment_slots.weapon = Some("青锋剑".to_string());
        let outcome = evaluate_combat_hard_constraints(&state, "遭遇元婴强者拦截");
        assert!(!outcome.accepted);
        assert!(outcome.reasons.iter().any(|r| r.contains("大境界压制")));
    }

    #[test]
    fn test_evaluate_combat_hard_constraints_penalizes_missing_weapon_for_blade_style() {
        let script = create_test_script();
        let state = crate::game_state::GameState {
            player: crate::game_state::Character::new(
                "player".to_string(),
                "Tester".to_string(),
                crate::models::CharacterStats {
                    spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    cultivation_realm: crate::models::CultivationRealm::new(
                        "Qi".to_string(),
                        2,
                        0,
                        1.0,
                    ),
                    techniques: vec!["断浪刀法".to_string()],
                    lifespan: crate::models::Lifespan {
                        current_age: 16,
                        max_age: 100,
                        realm_bonus: 0,
                    },
                    combat_power: 220,
                },
                "sect".to_string(),
            ),
            world_state: crate::game_state::WorldState::from_script(&script),
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
            script,
        };
        let outcome = evaluate_combat_hard_constraints(&state, "与筑基对手近战刀拼");
        assert!(outcome.accepted);
        assert!(outcome.power_delta_pct < 0);
        assert!(outcome.reasons.iter().any(|r| r.contains("未装备武器")));
    }

    #[test]
    fn test_apply_combat_aftermath_updates_social_profile() {
        let script = create_test_script();
        let mut state = crate::game_state::GameState {
            player: crate::game_state::Character::new(
                "player".to_string(),
                "Tester".to_string(),
                crate::models::CharacterStats {
                    spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    cultivation_realm: crate::models::CultivationRealm::new(
                        "Qi".to_string(),
                        1,
                        0,
                        1.0,
                    ),
                    techniques: vec![],
                    lifespan: crate::models::Lifespan {
                        current_age: 16,
                        max_age: 100,
                        realm_bonus: 0,
                    },
                    combat_power: 120,
                },
                "sect".to_string(),
            ),
            world_state: crate::game_state::WorldState::from_script(&script),
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
            script,
        };
        state.player.social_profile.sect_affinity = 10;
        state.player.social_profile.vendetta = 4;

        let _ = apply_combat_aftermath(&mut state, true, Some(CombatStrategy::Cautious));
        assert!(state.player.social_profile.favor >= 1);
        assert!(state.player.social_profile.mentor_bond >= 1);
        assert!(state.player.social_profile.vendetta <= 4);
        assert_eq!(state.player.combat_tendency, "cautious");
        assert!(
            state.player.social_profile.camp_stance == "righteous"
                || state.player.social_profile.camp_stance == "neutral"
        );
    }

    #[test]
    fn test_evaluate_environment_combat_modifier_high_energy_bonus() {
        let mut world_setting = crate::script::WorldSetting::new();
        world_setting.locations = vec![crate::script::Location {
            id: "peak".to_string(),
            name: "灵峰".to_string(),
            description: "灵气充沛".to_string(),
            spiritual_energy: 0.9,
        }];
        let script = crate::script::Script::new(
            "id".to_string(),
            "name".to_string(),
            crate::script::ScriptType::Custom,
            world_setting.clone(),
            crate::script::InitialState {
                player_name: "p".to_string(),
                player_spiritual_root: crate::models::SpiritualRoot {
                    element: crate::models::Element::Fire,
                    elements: vec![crate::models::Element::Fire],
                    grade: crate::models::Grade::Heavenly,
                    affinity: 0.8,
                },
                starting_location: "peak".to_string(),
                starting_age: 16,
            },
        );
        let state = crate::game_state::GameState {
            player: crate::game_state::Character::new(
                "player".to_string(),
                "Tester".to_string(),
                crate::models::CharacterStats {
                    spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    cultivation_realm: crate::models::CultivationRealm::new(
                        "Qi".to_string(),
                        1,
                        0,
                        1.0,
                    ),
                    techniques: vec![],
                    lifespan: crate::models::Lifespan {
                        current_age: 16,
                        max_age: 100,
                        realm_bonus: 0,
                    },
                    combat_power: 120,
                },
                "peak".to_string(),
            ),
            world_state: crate::game_state::WorldState::from_script(&script),
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
            script,
        };
        let (delta, reason) = evaluate_environment_combat_modifier(&state);
        assert!(delta > 0);
        assert!(reason.contains("高灵气场域增幅"));
    }

    #[test]
    fn test_compute_map_overview_contains_reachability() {
        let mut world_setting = crate::script::WorldSetting::new();
        world_setting.locations = vec![
            crate::script::Location {
                id: "sect".to_string(),
                name: "青云宗".to_string(),
                description: "宗门驻地".to_string(),
                spiritual_energy: 0.3,
            },
            crate::script::Location {
                id: "market".to_string(),
                name: "坊市".to_string(),
                description: "交易区域".to_string(),
                spiritual_energy: 0.35,
            },
        ];
        let script = crate::script::Script::new(
            "id".to_string(),
            "name".to_string(),
            crate::script::ScriptType::Custom,
            world_setting.clone(),
            crate::script::InitialState {
                player_name: "p".to_string(),
                player_spiritual_root: crate::models::SpiritualRoot {
                    element: crate::models::Element::Fire,
                    elements: vec![crate::models::Element::Fire],
                    grade: crate::models::Grade::Heavenly,
                    affinity: 0.8,
                },
                starting_location: "sect".to_string(),
                starting_age: 16,
            },
        );
        let state = crate::game_state::GameState {
            player: crate::game_state::Character::new(
                "player".to_string(),
                "Tester".to_string(),
                crate::models::CharacterStats {
                    spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    cultivation_realm: crate::models::CultivationRealm::new(
                        "Qi".to_string(),
                        1,
                        0,
                        1.0,
                    ),
                    techniques: vec![],
                    lifespan: crate::models::Lifespan {
                        current_age: 16,
                        max_age: 100,
                        realm_bonus: 0,
                    },
                    combat_power: 120,
                },
                "sect".to_string(),
            ),
            world_state: crate::game_state::WorldState::from_script(&script),
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
            script,
        };
        let overview = compute_map_overview(&state);
        assert_eq!(overview.len(), 2);
        assert!(overview
            .iter()
            .any(|node| node.location_id == "sect" && node.reachable));
    }

    #[test]
    fn test_compute_map_overview_contains_ecology_fields() {
        let mut world_setting = crate::script::WorldSetting::new();
        world_setting.factions = vec![crate::script::Faction {
            id: "qingyun".to_string(),
            name: "青云宗".to_string(),
            description: "宗门势力".to_string(),
            power_level: 8,
        }];
        world_setting.locations = vec![
            crate::script::Location {
                id: "sect".to_string(),
                name: "青云宗驻地".to_string(),
                description: "宗门与坊市交界".to_string(),
                spiritual_energy: 0.62,
            },
            crate::script::Location {
                id: "forbidden_valley".to_string(),
                name: "禁地幽谷".to_string(),
                description: "魔息波动强烈".to_string(),
                spiritual_energy: 0.93,
            },
        ];
        let script = crate::script::Script::new(
            "id".to_string(),
            "name".to_string(),
            crate::script::ScriptType::Custom,
            world_setting.clone(),
            crate::script::InitialState {
                player_name: "p".to_string(),
                player_spiritual_root: crate::models::SpiritualRoot {
                    element: crate::models::Element::Fire,
                    elements: vec![crate::models::Element::Fire],
                    grade: crate::models::Grade::Heavenly,
                    affinity: 0.8,
                },
                starting_location: "sect".to_string(),
                starting_age: 16,
            },
        );
        let state = crate::game_state::GameState {
            player: crate::game_state::Character::new(
                "player".to_string(),
                "Tester".to_string(),
                crate::models::CharacterStats {
                    spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    cultivation_realm: crate::models::CultivationRealm::new(
                        "Qi".to_string(),
                        1,
                        0,
                        1.0,
                    ),
                    techniques: vec![],
                    lifespan: crate::models::Lifespan {
                        current_age: 16,
                        max_age: 100,
                        realm_bonus: 0,
                    },
                    combat_power: 120,
                },
                "sect".to_string(),
            ),
            world_state: crate::game_state::WorldState::from_script(&script),
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
            script,
        };
        let overview = compute_map_overview(&state);
        let sect_node = overview
            .iter()
            .find(|n| n.location_id == "sect")
            .expect("sect node");
        assert!(!sect_node.environment_tags.is_empty());
        assert!(!sect_node.resource_tags.is_empty());
        assert!(!sect_node.control_faction.is_empty());
    }

    #[test]
    fn test_infer_location_ecology_marks_hotspot_for_high_risk_nodes() {
        let location = crate::script::Location {
            id: "forbidden_abyss".to_string(),
            name: "禁地深渊".to_string(),
            description: "魔息翻涌".to_string(),
            spiritual_energy: 0.9,
        };
        let world_setting = crate::script::WorldSetting::new();
        let (_, _, _, hotspot) = infer_location_ecology(&location, &world_setting, "high");
        assert!(hotspot);
    }

    #[test]
    fn test_evaluate_style_counter_modifier_detects_counter_bonus() {
        let stats = crate::models::CharacterStats {
            spiritual_root: crate::models::SpiritualRoot {
                element: crate::models::Element::Fire,
                elements: vec![crate::models::Element::Fire],
                grade: crate::models::Grade::Heavenly,
                affinity: 0.8,
            },
            cultivation_realm: crate::models::CultivationRealm::new("Qi".to_string(), 2, 0, 1.0),
            techniques: vec!["青霜剑诀".to_string()],
            lifespan: crate::models::Lifespan {
                current_age: 18,
                max_age: 100,
                realm_bonus: 0,
            },
            combat_power: 220,
        };
        let (delta, reason) = evaluate_style_counter_modifier(&stats, "敌方体修强攻");
        assert!(delta > 0);
        assert!(reason.contains("克制"));
    }

    #[test]
    fn test_evaluate_style_counter_modifier_handles_unknown_enemy_style() {
        let stats = crate::models::CharacterStats {
            spiritual_root: crate::models::SpiritualRoot {
                element: crate::models::Element::Fire,
                elements: vec![crate::models::Element::Fire],
                grade: crate::models::Grade::Heavenly,
                affinity: 0.8,
            },
            cultivation_realm: crate::models::CultivationRealm::new("Qi".to_string(), 2, 0, 1.0),
            techniques: vec!["青霜剑诀".to_string()],
            lifespan: crate::models::Lifespan {
                current_age: 18,
                max_age: 100,
                realm_bonus: 0,
            },
            combat_power: 220,
        };
        let (delta, reason) = evaluate_style_counter_modifier(&stats, "敌方神秘修士");
        assert_eq!(delta, 0);
        assert!(reason.contains("未识别"));
    }

    #[test]
    fn test_extract_styles_from_text_extracts_styles() {
        assert!(extract_styles_from_text("我以剑诀起手").contains(&"sword"));
        assert!(extract_styles_from_text("贴身体修对轰").contains(&"body"));
        assert!(extract_styles_from_text("普通描述").is_empty());
    }

    #[test]
    fn test_evaluate_style_counter_modifier_uses_hint_when_technique_empty() {
        let stats = crate::models::CharacterStats {
            spiritual_root: crate::models::SpiritualRoot {
                element: crate::models::Element::Fire,
                elements: vec![crate::models::Element::Fire],
                grade: crate::models::Grade::Heavenly,
                affinity: 0.8,
            },
            cultivation_realm: crate::models::CultivationRealm::new("Qi".to_string(), 2, 0, 1.0),
            techniques: vec![],
            lifespan: crate::models::Lifespan {
                current_age: 18,
                max_age: 100,
                realm_bonus: 0,
            },
            combat_power: 220,
        };
        let (delta, reason) = evaluate_style_counter_modifier(&stats, "我以剑诀迎战体修强攻");
        assert!(delta > 0);
        assert!(reason.contains("克制"));
    }

    fn make_option(id: usize, description: &str) -> PlayerOption {
        PlayerOption {
            id,
            description: description.to_string(),
            requirements: vec![],
            action: Action::Custom {
                description: description.to_string(),
            },
        }
    }

    fn make_state_for_option_sanitize() -> GameState {
        let mut world_setting = crate::script::WorldSetting::new();
        world_setting.locations = vec![
            crate::script::Location {
                id: "sect".to_string(),
                name: "青云宗".to_string(),
                description: "宗门驻地".to_string(),
                spiritual_energy: 0.30,
            },
            crate::script::Location {
                id: "market".to_string(),
                name: "坊市".to_string(),
                description: "交易区".to_string(),
                spiritual_energy: 0.32,
            },
            crate::script::Location {
                id: "forest".to_string(),
                name: "幽林".to_string(),
                description: "外围山林".to_string(),
                spiritual_energy: 0.36,
            },
            crate::script::Location {
                id: "far_city".to_string(),
                name: "天机城".to_string(),
                description: "遥远城池".to_string(),
                spiritual_energy: 2.50,
            },
        ];

        let script = crate::script::Script::new(
            "id".to_string(),
            "name".to_string(),
            crate::script::ScriptType::Custom,
            world_setting,
            crate::script::InitialState {
                player_name: "p".to_string(),
                player_spiritual_root: crate::models::SpiritualRoot {
                    element: crate::models::Element::Fire,
                    elements: vec![crate::models::Element::Fire],
                    grade: crate::models::Grade::Heavenly,
                    affinity: 0.8,
                },
                starting_location: "sect".to_string(),
                starting_age: 16,
            },
        );

        crate::game_state::GameState {
            player: crate::game_state::Character::new(
                "player".to_string(),
                "Tester".to_string(),
                crate::models::CharacterStats {
                    spiritual_root: crate::models::SpiritualRoot {
                        element: crate::models::Element::Fire,
                        elements: vec![crate::models::Element::Fire],
                        grade: crate::models::Grade::Heavenly,
                        affinity: 0.8,
                    },
                    cultivation_realm: crate::models::CultivationRealm::new(
                        "Qi".to_string(),
                        1,
                        0,
                        1.0,
                    ),
                    techniques: vec![],
                    lifespan: crate::models::Lifespan {
                        current_age: 16,
                        max_age: 100,
                        realm_bonus: 0,
                    },
                    combat_power: 120,
                },
                "sect".to_string(),
            ),
            world_state: crate::game_state::WorldState::from_script(&script),
            game_time: crate::game_state::GameTime::new(1, 1, 1),
            event_history: vec![],
            script,
        }
    }

    #[test]
    fn test_sanitize_generated_options_drops_dead_character_dialogue() {
        let game_state = make_state_for_option_sanitize();
        let mut registry = WorldRegistry::fallback_from_game_state(&game_state, "test");
        registry.tables.characters.push(serde_json::json!({
            "id": "c_dead",
            "name": "韩长老",
            "dead": true,
            "location_id": "sect"
        }));
        let options = vec![
            make_option(0, "向韩长老请教剑道"),
            make_option(1, "在宗门广场观摩切磋"),
        ];

        let (sanitized, notes) =
            sanitize_generated_options(options, &game_state, "sect", Some(&registry));

        assert_eq!(sanitized.len(), 1);
        assert!(sanitized[0].description.contains("观摩"));
        assert!(notes
            .iter()
            .any(|n| n == "option_repair:dropped_dead_character_dialogue"));
    }

    #[test]
    fn test_sanitize_generated_options_drops_absent_character_dialogue() {
        let game_state = make_state_for_option_sanitize();
        let mut registry = WorldRegistry::fallback_from_game_state(&game_state, "test");
        registry.tables.characters.push(serde_json::json!({
            "id": "c_far",
            "name": "苏执事",
            "location_id": "far_city",
            "dead": false
        }));
        let options = vec![
            make_option(0, "与苏执事对话打探消息"),
            make_option(1, "先整顿行装"),
        ];

        let (sanitized, notes) =
            sanitize_generated_options(options, &game_state, "sect", Some(&registry));

        assert_eq!(sanitized.len(), 1);
        assert!(sanitized[0].description.contains("整顿行装"));
        assert!(notes
            .iter()
            .any(|n| n == "option_repair:dropped_absent_character_dialogue"));
    }

    #[test]
    fn test_sanitize_generated_options_rewrites_unreachable_location() {
        let game_state = make_state_for_option_sanitize();
        let options = vec![make_option(0, "立刻前往天机城追查线索")];

        let (sanitized, notes) = sanitize_generated_options(options, &game_state, "sect", None);

        assert_eq!(sanitized.len(), 1);
        assert!(sanitized[0].description.contains("暂不贸然前往天机城"));
        assert!(notes
            .iter()
            .any(|n| n.starts_with("option_repair:rewrote_unreachable_location(")));
    }

    #[test]
    fn test_sanitize_generated_options_rewrites_high_risk_when_injured() {
        let mut game_state = make_state_for_option_sanitize();
        game_state.player.combat_status.injury_level = 8;
        let options = vec![make_option(0, "强攻破阵，硬拼到底")];

        let (sanitized, notes) = sanitize_generated_options(options, &game_state, "sect", None);

        assert_eq!(sanitized.len(), 1);
        assert!(sanitized[0].description.contains("先稳住伤势"));
        assert!(notes
            .iter()
            .any(|n| n == "option_repair:rewrote_high_risk_option_due_to_injury"));
    }

    #[test]
    fn test_normalize_action_bucket_stable_and_fallback() {
        let action = PlayerAction {
            action_type: crate::plot_engine::ActionType::FreeText,
            content: "  Talk-To Elder_01!!! ".to_string(),
            selected_option_id: Some(3),
            meta: None,
        };
        assert_eq!(normalize_action_bucket(&action), "talktoelder01");

        let selected_only = PlayerAction {
            action_type: crate::plot_engine::ActionType::SelectedOption,
            content: "   ".to_string(),
            selected_option_id: Some(2),
            meta: None,
        };
        assert_eq!(normalize_action_bucket(&selected_only), "selected:2");
    }

    #[test]
    fn test_build_noname_action_summary_prefers_selected_option_text() {
        let action = PlayerAction {
            action_type: crate::plot_engine::ActionType::SelectedOption,
            content: String::new(),
            selected_option_id: Some(0),
            meta: None,
        };
        let plot_state = PlotState {
            current_scene: crate::plot_engine::Scene {
                id: "scene-1".to_string(),
                name: "山门".to_string(),
                description: "山门风声渐紧".to_string(),
                location: "sect".to_string(),
                available_options: vec![make_option(0, "返回山门广场")],
            },
            plot_history: vec![],
            is_waiting_for_input: true,
            interaction_state: PlotInteractionState::WaitingForChoice,
            last_action_result: None,
            settings: PlotSettings::default(),
            current_chapter: crate::plot_engine::ChapterState::new(1, "第一章".to_string()),
            chapters: vec![],
            segment_count: 0,
            last_generation_diagnostics: None,
            last_option_generation_source: None,
            last_consistency_risk_score: None,
        };

        let summary = build_noname_action_summary(&action, &plot_state);
        assert!(summary.contains("返回山门广场"));
    }

    #[test]
    fn test_append_noname_observation_diagnostics_appends_note() {
        let mut diagnostics = Some("原始诊断".to_string());
        let observation = NoNameDirectorObservation {
            role: NoNameRole::Director,
            action_summary: "选择返回山门".to_string(),
            focus: "山门危机".to_string(),
            rationale: "应优先观察山门危机".to_string(),
            prompt_preview: "prompt".to_string(),
            proposal: crate::noname_types::NoNameProposal {
                proposal_id: "proposal-1".to_string(),
                kind: crate::noname_types::NoNameProposalKind::PlotCandidate,
                producer_role: NoNameRole::Director,
                title: "Director提案：山门危机".to_string(),
                summary: "建议优先观察山门危机".to_string(),
                focus: "山门危机".to_string(),
                target_segment: crate::noname_types::NoNameTargetSegment::CurrentTurnTail,
                intended_effect: "维持低风险观察导向".to_string(),
                rationale: "应优先观察山门危机".to_string(),
                suggested_action: Some("保持 observe-only".to_string()),
                labels: vec!["director".to_string()],
                apply_scopes: vec![crate::noname_types::NoNameApplyScope::Diagnostics],
                status: crate::noname_types::NoNameProposalStatus::Observed,
                applyable: false,
            },
        };
        let guardrail = NoNameGuardrailResult::accept();

        append_noname_observation_diagnostics(
            &mut diagnostics,
            NoNameMode::ObserveOnly,
            &observation,
            Some(&guardrail),
            &NoNameTrace {
                trace_id: "trace-1".to_string(),
                session_id: "session-1".to_string(),
                turn_id: "turn-1".to_string(),
                mode: NoNameMode::ObserveOnly,
                graph_path: vec![],
                capability_calls: vec![],
                proposals: vec![],
                proposal_transition_log: vec!["proposal-1:observed".to_string()],
                apply_plan_log: vec![],
                apply_execution_log: vec![],
                related_observations: vec![],
                protocol_events: vec![],
                guardrail_result: None,
                apply_result: Some(crate::noname_trace::NoNameApplyTraceResult {
                    attempted: false,
                    outcome: "skipped_observe_only".to_string(),
                    reason: Some("当前模式仅观察，不尝试 assisted apply".to_string()),
                }),
                fallback_used: false,
                elapsed_ms: 0,
            },
        );

        let text = diagnostics.expect("diagnostics should exist");
        assert!(text.contains("原始诊断"));
        assert!(text.contains("NoName.observe_only"));
        assert!(text.contains("山门危机"));
        assert!(text.contains("Director提案"));
        assert!(text.contains("proposal_status=observed"));
        assert!(text.contains("applyable=no"));
        assert!(text.contains("guardrail=accept"));
        assert!(text.contains("apply=skipped_observe_only"));
    }

    #[test]
    fn test_apply_noname_low_risk_outputs_updates_summary_hint_for_applied_proposal() {
        let mut plot_state = PlotState {
            current_scene: crate::plot_engine::Scene {
                id: "scene-1".to_string(),
                name: "山门".to_string(),
                description: "山门风声渐紧".to_string(),
                location: "sect".to_string(),
                available_options: vec![make_option(0, "返回山门广场")],
            },
            plot_history: vec![],
            is_waiting_for_input: true,
            interaction_state: PlotInteractionState::WaitingForChoice,
            last_action_result: None,
            settings: PlotSettings::default(),
            current_chapter: crate::plot_engine::ChapterState::new(1, "第一章".to_string()),
            chapters: vec![],
            segment_count: 0,
            last_generation_diagnostics: None,
            last_option_generation_source: None,
            last_consistency_risk_score: None,
        };
        let mut result = NoNameDirectorRunResult {
            trace: NoNameTrace {
                trace_id: "trace-1".to_string(),
                session_id: "session-1".to_string(),
                turn_id: "turn-1".to_string(),
                mode: NoNameMode::Assisted,
                graph_path: vec![crate::noname_types::NoNameTraceStage::ApplyProposal],
                capability_calls: vec![],
                proposals: vec![],
                proposal_transition_log: vec![],
                apply_plan_log: vec![],
                apply_execution_log: vec![],
                related_observations: vec![],
                protocol_events: vec![],
                guardrail_result: None,
                apply_result: None,
                fallback_used: false,
                elapsed_ms: 0,
            },
            observation: NoNameDirectorObservation {
                role: NoNameRole::Director,
                action_summary: "选择返回山门".to_string(),
                focus: "山门危机".to_string(),
                rationale: "应优先观察山门危机".to_string(),
                prompt_preview: "prompt".to_string(),
                proposal: crate::noname_types::NoNameProposal {
                    proposal_id: "proposal-1".to_string(),
                    kind: crate::noname_types::NoNameProposalKind::PlotCandidate,
                    producer_role: NoNameRole::Director,
                    title: "Director提案：山门危机".to_string(),
                    summary: "建议优先观察山门危机".to_string(),
                    focus: "山门危机".to_string(),
                    target_segment: crate::noname_types::NoNameTargetSegment::CurrentTurnTail,
                    intended_effect: "为下一轮提供冲突承接".to_string(),
                    rationale: "应优先观察山门危机".to_string(),
                    suggested_action: Some("进入低风险 apply".to_string()),
                    labels: vec![
                        "director".to_string(),
                        "apply_scope_diagnostics".to_string(),
                    ],
                    apply_scopes: vec![
                        crate::noname_types::NoNameApplyScope::Diagnostics,
                        crate::noname_types::NoNameApplyScope::ChapterSummaryHint,
                        crate::noname_types::NoNameApplyScope::OptionBiasHint,
                    ],
                    status: crate::noname_types::NoNameProposalStatus::Applied,
                    applyable: true,
                },
            },
            proposal: crate::noname_types::NoNameProposal {
                proposal_id: "proposal-1".to_string(),
                kind: crate::noname_types::NoNameProposalKind::PlotCandidate,
                producer_role: NoNameRole::Director,
                title: "Director提案：山门危机".to_string(),
                summary: "建议优先观察山门危机".to_string(),
                focus: "山门危机".to_string(),
                target_segment: crate::noname_types::NoNameTargetSegment::CurrentTurnTail,
                intended_effect: "为下一轮提供冲突承接".to_string(),
                rationale: "应优先观察山门危机".to_string(),
                suggested_action: Some("进入低风险 apply".to_string()),
                labels: vec![
                    "director".to_string(),
                    "apply_scope_diagnostics".to_string(),
                ],
                apply_scopes: vec![
                    crate::noname_types::NoNameApplyScope::Diagnostics,
                    crate::noname_types::NoNameApplyScope::ChapterSummaryHint,
                    crate::noname_types::NoNameApplyScope::OptionBiasHint,
                ],
                status: crate::noname_types::NoNameProposalStatus::Applied,
                applyable: true,
            },
            guardrail_result: None,
            related_observations: vec![],
        };

        apply_noname_low_risk_outputs(&mut plot_state, &mut result, false);

        assert!(plot_state.current_chapter.summary.contains("NoName提示"));
        assert!(plot_state.current_chapter.summary.contains("山门危机"));
        assert!(plot_state
            .last_generation_diagnostics
            .as_deref()
            .unwrap_or_default()
            .contains("NoName选项偏置"));
        assert!(result
            .trace
            .proposal_transition_log
            .iter()
            .any(|item| item.contains("chapter_summary_hint")));
        assert!(result
            .trace
            .proposal_transition_log
            .iter()
            .any(|item| item.contains("option_bias_hint")));
        assert_eq!(
            result
                .trace
                .apply_result
                .as_ref()
                .map(|item| item.outcome.as_str()),
            Some("applied_scoped_outputs")
        );
        assert!(result
            .trace
            .apply_result
            .as_ref()
            .and_then(|item| item.reason.as_ref())
            .unwrap_or(&String::new())
            .contains("chapter_summary_hint, option_bias_hint"));
    }

    #[test]
    fn test_apply_noname_plot_text_hint_appends_note() {
        let mut result = NoNameDirectorRunResult {
            trace: NoNameTrace {
                trace_id: "trace-plot-text".to_string(),
                session_id: "session-1".to_string(),
                turn_id: "turn-1".to_string(),
                mode: NoNameMode::Assisted,
                graph_path: vec![crate::noname_types::NoNameTraceStage::ApplyProposal],
                capability_calls: vec![],
                proposals: vec![],
                proposal_transition_log: vec![],
                apply_plan_log: vec![],
                apply_execution_log: vec![],
                related_observations: vec![],
                protocol_events: vec![],
                guardrail_result: None,
                apply_result: None,
                fallback_used: false,
                elapsed_ms: 0,
            },
            observation: NoNameDirectorObservation {
                role: NoNameRole::Director,
                action_summary: "选择返回山门".to_string(),
                focus: "山门危机".to_string(),
                rationale: "应优先观察山门危机".to_string(),
                prompt_preview: "prompt".to_string(),
                proposal: crate::noname_types::NoNameProposal {
                    proposal_id: "proposal-plot-text".to_string(),
                    kind: crate::noname_types::NoNameProposalKind::PlotCandidate,
                    producer_role: NoNameRole::Director,
                    title: "Director提案：山门危机".to_string(),
                    summary: "建议优先观察山门危机".to_string(),
                    focus: "山门危机".to_string(),
                    target_segment: crate::noname_types::NoNameTargetSegment::CurrentTurnTail,
                    intended_effect: "对正文尾段补充轻量导向".to_string(),
                    rationale: "应优先观察山门危机".to_string(),
                    suggested_action: Some("进入低风险 apply".to_string()),
                    labels: vec!["director".to_string()],
                    apply_scopes: vec![crate::noname_types::NoNameApplyScope::PlotTextHint],
                    status: crate::noname_types::NoNameProposalStatus::Applied,
                    applyable: true,
                },
            },
            proposal: crate::noname_types::NoNameProposal {
                proposal_id: "proposal-plot-text".to_string(),
                kind: crate::noname_types::NoNameProposalKind::PlotCandidate,
                producer_role: NoNameRole::Director,
                title: "Director提案：山门危机".to_string(),
                summary: "建议优先观察山门危机".to_string(),
                focus: "山门危机".to_string(),
                target_segment: crate::noname_types::NoNameTargetSegment::CurrentTurnTail,
                intended_effect: "对正文尾段补充轻量导向".to_string(),
                rationale: "应优先观察山门危机".to_string(),
                suggested_action: Some("进入低风险 apply".to_string()),
                labels: vec!["director".to_string()],
                apply_scopes: vec![crate::noname_types::NoNameApplyScope::PlotTextHint],
                status: crate::noname_types::NoNameProposalStatus::Applied,
                applyable: true,
            },
            guardrail_result: None,
            related_observations: vec![],
        };
        let mut plot_text = "山门风声渐紧".to_string();

        let applied = apply_noname_plot_text_hint(&mut plot_text, &mut result);

        assert!(applied);
        assert!(plot_text.contains("【NoName】重点关注：山门危机"));
        assert!(result
            .trace
            .proposal_transition_log
            .iter()
            .any(|item| item.contains("plot_text_hint")));
        assert!(result
            .trace
            .apply_execution_log
            .iter()
            .any(|item| item.target == "plot_text_hint" && item.outcome == "applied"));
    }

    #[test]
    fn test_apply_noname_plot_text_hint_can_prepend_to_head() {
        let mut result = NoNameDirectorRunResult {
            trace: NoNameTrace {
                trace_id: "trace-plot-text-head".to_string(),
                session_id: "session-1".to_string(),
                turn_id: "turn-1".to_string(),
                mode: NoNameMode::Assisted,
                graph_path: vec![crate::noname_types::NoNameTraceStage::ApplyProposal],
                capability_calls: vec![],
                proposals: vec![],
                proposal_transition_log: vec![],
                apply_plan_log: vec![],
                apply_execution_log: vec![],
                related_observations: vec![],
                protocol_events: vec![],
                guardrail_result: None,
                apply_result: None,
                fallback_used: false,
                elapsed_ms: 0,
            },
            observation: NoNameDirectorObservation {
                role: NoNameRole::Director,
                action_summary: "选择返回山门".to_string(),
                focus: "山门危机".to_string(),
                rationale: "应优先观察山门危机".to_string(),
                prompt_preview: "prompt".to_string(),
                proposal: crate::noname_types::NoNameProposal {
                    proposal_id: "proposal-plot-text-head".to_string(),
                    kind: crate::noname_types::NoNameProposalKind::PlotCandidate,
                    producer_role: NoNameRole::Director,
                    title: "Director提案：山门危机".to_string(),
                    summary: "建议优先观察山门危机".to_string(),
                    focus: "山门危机".to_string(),
                    target_segment: crate::noname_types::NoNameTargetSegment::CurrentTurnHead,
                    intended_effect: "对正文首段补充轻量导向".to_string(),
                    rationale: "应优先观察山门危机".to_string(),
                    suggested_action: Some("进入低风险 apply".to_string()),
                    labels: vec!["director".to_string()],
                    apply_scopes: vec![crate::noname_types::NoNameApplyScope::PlotTextHint],
                    status: crate::noname_types::NoNameProposalStatus::Applied,
                    applyable: true,
                },
            },
            proposal: crate::noname_types::NoNameProposal {
                proposal_id: "proposal-plot-text-head".to_string(),
                kind: crate::noname_types::NoNameProposalKind::PlotCandidate,
                producer_role: NoNameRole::Director,
                title: "Director提案：山门危机".to_string(),
                summary: "建议优先观察山门危机".to_string(),
                focus: "山门危机".to_string(),
                target_segment: crate::noname_types::NoNameTargetSegment::CurrentTurnHead,
                intended_effect: "对正文首段补充轻量导向".to_string(),
                rationale: "应优先观察山门危机".to_string(),
                suggested_action: Some("进入低风险 apply".to_string()),
                labels: vec!["director".to_string()],
                apply_scopes: vec![crate::noname_types::NoNameApplyScope::PlotTextHint],
                status: crate::noname_types::NoNameProposalStatus::Applied,
                applyable: true,
            },
            guardrail_result: None,
            related_observations: vec![],
        };
        let mut plot_text = "山门风声渐紧".to_string();

        let applied = apply_noname_plot_text_hint(&mut plot_text, &mut result);

        assert!(applied);
        assert!(plot_text.starts_with("【NoName】重点关注：山门危机"));
        assert!(plot_text.ends_with("山门风声渐紧"));
    }

    #[test]
    fn test_apply_noname_low_risk_outputs_respects_apply_scope() {
        let mut plot_state = PlotState {
            current_scene: crate::plot_engine::Scene {
                id: "scene-1".to_string(),
                name: "山门".to_string(),
                description: "山门风声渐紧".to_string(),
                location: "sect".to_string(),
                available_options: vec![make_option(0, "返回山门广场")],
            },
            plot_history: vec![],
            is_waiting_for_input: true,
            interaction_state: PlotInteractionState::WaitingForChoice,
            last_action_result: None,
            settings: PlotSettings::default(),
            current_chapter: crate::plot_engine::ChapterState::new(1, "第一章".to_string()),
            chapters: vec![],
            segment_count: 0,
            last_generation_diagnostics: None,
            last_option_generation_source: None,
            last_consistency_risk_score: None,
        };
        let mut result = NoNameDirectorRunResult {
            trace: NoNameTrace {
                trace_id: "trace-scope".to_string(),
                session_id: "session-1".to_string(),
                turn_id: "turn-1".to_string(),
                mode: NoNameMode::Assisted,
                graph_path: vec![crate::noname_types::NoNameTraceStage::ApplyProposal],
                capability_calls: vec![],
                proposals: vec![],
                proposal_transition_log: vec![],
                apply_plan_log: vec![],
                apply_execution_log: vec![],
                related_observations: vec![],
                protocol_events: vec![],
                guardrail_result: None,
                apply_result: None,
                fallback_used: false,
                elapsed_ms: 0,
            },
            observation: NoNameDirectorObservation {
                role: NoNameRole::Director,
                action_summary: "选择返回山门".to_string(),
                focus: "山门危机".to_string(),
                rationale: "应优先观察山门危机".to_string(),
                prompt_preview: "prompt".to_string(),
                proposal: crate::noname_types::NoNameProposal {
                    proposal_id: "proposal-scope".to_string(),
                    kind: crate::noname_types::NoNameProposalKind::PlotCandidate,
                    producer_role: NoNameRole::Director,
                    title: "Director提案：山门危机".to_string(),
                    summary: "建议优先观察山门危机".to_string(),
                    focus: "山门危机".to_string(),
                    target_segment: crate::noname_types::NoNameTargetSegment::CurrentTurnTail,
                    intended_effect: "只影响下一轮选项偏置".to_string(),
                    rationale: "应优先观察山门危机".to_string(),
                    suggested_action: Some("进入低风险 apply".to_string()),
                    labels: vec!["director".to_string()],
                    apply_scopes: vec![crate::noname_types::NoNameApplyScope::OptionBiasHint],
                    status: crate::noname_types::NoNameProposalStatus::Applied,
                    applyable: true,
                },
            },
            proposal: crate::noname_types::NoNameProposal {
                proposal_id: "proposal-scope".to_string(),
                kind: crate::noname_types::NoNameProposalKind::PlotCandidate,
                producer_role: NoNameRole::Director,
                title: "Director提案：山门危机".to_string(),
                summary: "建议优先观察山门危机".to_string(),
                focus: "山门危机".to_string(),
                target_segment: crate::noname_types::NoNameTargetSegment::CurrentTurnTail,
                intended_effect: "只影响下一轮选项偏置".to_string(),
                rationale: "应优先观察山门危机".to_string(),
                suggested_action: Some("进入低风险 apply".to_string()),
                labels: vec!["director".to_string()],
                apply_scopes: vec![crate::noname_types::NoNameApplyScope::OptionBiasHint],
                status: crate::noname_types::NoNameProposalStatus::Applied,
                applyable: true,
            },
            guardrail_result: None,
            related_observations: vec![],
        };

        apply_noname_low_risk_outputs(&mut plot_state, &mut result, false);

        assert!(!plot_state.current_chapter.summary.contains("NoName提示"));
        assert!(plot_state
            .last_generation_diagnostics
            .as_deref()
            .unwrap_or_default()
            .contains("NoName选项偏置"));
        assert!(result
            .trace
            .apply_execution_log
            .iter()
            .any(|item| item.target == "chapter_summary_hint"
                && item.outcome == "skipped_scope_forbidden"));
        assert_eq!(
            result
                .trace
                .apply_result
                .as_ref()
                .map(|item| item.outcome.as_str()),
            Some("applied_scoped_outputs")
        );
    }

    #[test]
    fn test_apply_noname_summary_hint_can_prepend_to_summary_head() {
        let mut plot_state = PlotState {
            current_scene: crate::plot_engine::Scene {
                id: "scene-1".to_string(),
                name: "山门".to_string(),
                description: "山门风声渐紧".to_string(),
                location: "sect".to_string(),
                available_options: vec![make_option(0, "返回山门广场")],
            },
            plot_history: vec![],
            is_waiting_for_input: true,
            interaction_state: PlotInteractionState::WaitingForChoice,
            last_action_result: None,
            settings: PlotSettings::default(),
            current_chapter: crate::plot_engine::ChapterState {
                summary: "已有摘要".to_string(),
                ..crate::plot_engine::ChapterState::new(1, "第一章".to_string())
            },
            chapters: vec![],
            segment_count: 0,
            last_generation_diagnostics: None,
            last_option_generation_source: None,
            last_consistency_risk_score: None,
        };
        let mut result = NoNameDirectorRunResult {
            trace: NoNameTrace {
                trace_id: "trace-summary-head".to_string(),
                session_id: "session-1".to_string(),
                turn_id: "turn-1".to_string(),
                mode: NoNameMode::Assisted,
                graph_path: vec![crate::noname_types::NoNameTraceStage::ApplyProposal],
                capability_calls: vec![],
                proposals: vec![],
                proposal_transition_log: vec![],
                apply_plan_log: vec![],
                apply_execution_log: vec![],
                related_observations: vec![],
                protocol_events: vec![],
                guardrail_result: None,
                apply_result: None,
                fallback_used: false,
                elapsed_ms: 0,
            },
            observation: NoNameDirectorObservation {
                role: NoNameRole::Director,
                action_summary: "选择返回山门".to_string(),
                focus: "山门危机".to_string(),
                rationale: "应优先观察山门危机".to_string(),
                prompt_preview: "prompt".to_string(),
                proposal: crate::noname_types::NoNameProposal {
                    proposal_id: "proposal-summary-head".to_string(),
                    kind: crate::noname_types::NoNameProposalKind::PlotCandidate,
                    producer_role: NoNameRole::Director,
                    title: "Director提案：山门危机".to_string(),
                    summary: "建议优先观察山门危机".to_string(),
                    focus: "山门危机".to_string(),
                    target_segment: crate::noname_types::NoNameTargetSegment::ChapterSummaryHead,
                    intended_effect: "将摘要提示置于已有摘要之前".to_string(),
                    rationale: "应优先观察山门危机".to_string(),
                    suggested_action: Some("进入低风险 apply".to_string()),
                    labels: vec!["director".to_string()],
                    apply_scopes: vec![crate::noname_types::NoNameApplyScope::ChapterSummaryHint],
                    status: crate::noname_types::NoNameProposalStatus::Applied,
                    applyable: true,
                },
            },
            proposal: crate::noname_types::NoNameProposal {
                proposal_id: "proposal-summary-head".to_string(),
                kind: crate::noname_types::NoNameProposalKind::PlotCandidate,
                producer_role: NoNameRole::Director,
                title: "Director提案：山门危机".to_string(),
                summary: "建议优先观察山门危机".to_string(),
                focus: "山门危机".to_string(),
                target_segment: crate::noname_types::NoNameTargetSegment::ChapterSummaryHead,
                intended_effect: "将摘要提示置于已有摘要之前".to_string(),
                rationale: "应优先观察山门危机".to_string(),
                suggested_action: Some("进入低风险 apply".to_string()),
                labels: vec!["director".to_string()],
                apply_scopes: vec![crate::noname_types::NoNameApplyScope::ChapterSummaryHint],
                status: crate::noname_types::NoNameProposalStatus::Applied,
                applyable: true,
            },
            guardrail_result: None,
            related_observations: vec![],
        };

        apply_noname_low_risk_outputs(&mut plot_state, &mut result, false);

        assert!(plot_state
            .current_chapter
            .summary
            .starts_with("NoName提示：后续重点关注山门危机；已有摘要"));
    }

    #[test]
    fn test_apply_noname_plot_text_hint_skips_when_target_segment_is_summary_only() {
        let mut result = NoNameDirectorRunResult {
            trace: NoNameTrace {
                trace_id: "trace-plot-text-mismatch".to_string(),
                session_id: "session-1".to_string(),
                turn_id: "turn-1".to_string(),
                mode: NoNameMode::Assisted,
                graph_path: vec![crate::noname_types::NoNameTraceStage::ApplyProposal],
                capability_calls: vec![],
                proposals: vec![],
                proposal_transition_log: vec![],
                apply_plan_log: vec![],
                apply_execution_log: vec![],
                related_observations: vec![],
                protocol_events: vec![],
                guardrail_result: None,
                apply_result: None,
                fallback_used: false,
                elapsed_ms: 0,
            },
            observation: NoNameDirectorObservation {
                role: NoNameRole::Director,
                action_summary: "选择返回山门".to_string(),
                focus: "山门危机".to_string(),
                rationale: "应优先观察山门危机".to_string(),
                prompt_preview: "prompt".to_string(),
                proposal: crate::noname_types::NoNameProposal {
                    proposal_id: "proposal-plot-text-mismatch".to_string(),
                    kind: crate::noname_types::NoNameProposalKind::PlotCandidate,
                    producer_role: NoNameRole::Director,
                    title: "Director提案：山门危机".to_string(),
                    summary: "建议优先观察山门危机".to_string(),
                    focus: "山门危机".to_string(),
                    target_segment: crate::noname_types::NoNameTargetSegment::ChapterSummaryHead,
                    intended_effect: "错误地尝试改写正文".to_string(),
                    rationale: "应优先观察山门危机".to_string(),
                    suggested_action: Some("进入低风险 apply".to_string()),
                    labels: vec!["director".to_string()],
                    apply_scopes: vec![crate::noname_types::NoNameApplyScope::PlotTextHint],
                    status: crate::noname_types::NoNameProposalStatus::Applied,
                    applyable: true,
                },
            },
            proposal: crate::noname_types::NoNameProposal {
                proposal_id: "proposal-plot-text-mismatch".to_string(),
                kind: crate::noname_types::NoNameProposalKind::PlotCandidate,
                producer_role: NoNameRole::Director,
                title: "Director提案：山门危机".to_string(),
                summary: "建议优先观察山门危机".to_string(),
                focus: "山门危机".to_string(),
                target_segment: crate::noname_types::NoNameTargetSegment::ChapterSummaryHead,
                intended_effect: "错误地尝试改写正文".to_string(),
                rationale: "应优先观察山门危机".to_string(),
                suggested_action: Some("进入低风险 apply".to_string()),
                labels: vec!["director".to_string()],
                apply_scopes: vec![crate::noname_types::NoNameApplyScope::PlotTextHint],
                status: crate::noname_types::NoNameProposalStatus::Applied,
                applyable: true,
            },
            guardrail_result: None,
            related_observations: vec![],
        };
        let mut plot_text = "山门风声渐紧".to_string();

        let applied = apply_noname_plot_text_hint(&mut plot_text, &mut result);

        assert!(!applied);
        assert_eq!(plot_text, "山门风声渐紧");
        assert!(result
            .trace
            .apply_execution_log
            .iter()
            .any(|item| item.target == "plot_text_hint"
                && item.outcome == "skipped_target_mismatch"));
    }

    #[test]
    fn test_apply_noname_summary_hint_skips_duplicate_focus() {
        let mut plot_state = PlotState {
            current_scene: crate::plot_engine::Scene {
                id: "scene-1".to_string(),
                name: "山门".to_string(),
                description: "山门风声渐紧".to_string(),
                location: "sect".to_string(),
                available_options: vec![make_option(0, "返回山门广场")],
            },
            plot_history: vec![],
            is_waiting_for_input: true,
            interaction_state: PlotInteractionState::WaitingForChoice,
            last_action_result: None,
            settings: PlotSettings::default(),
            current_chapter: crate::plot_engine::ChapterState {
                summary: "已有摘要；NoName提示：后续重点关注山门危机".to_string(),
                ..crate::plot_engine::ChapterState::new(1, "第一章".to_string())
            },
            chapters: vec![],
            segment_count: 0,
            last_generation_diagnostics: None,
            last_option_generation_source: None,
            last_consistency_risk_score: None,
        };
        let mut result = NoNameDirectorRunResult {
            trace: NoNameTrace {
                trace_id: "trace-summary-duplicate".to_string(),
                session_id: "session-1".to_string(),
                turn_id: "turn-1".to_string(),
                mode: NoNameMode::Assisted,
                graph_path: vec![crate::noname_types::NoNameTraceStage::ApplyProposal],
                capability_calls: vec![],
                proposals: vec![],
                proposal_transition_log: vec![],
                apply_plan_log: vec![],
                apply_execution_log: vec![],
                related_observations: vec![],
                protocol_events: vec![],
                guardrail_result: None,
                apply_result: None,
                fallback_used: false,
                elapsed_ms: 0,
            },
            observation: NoNameDirectorObservation {
                role: NoNameRole::Director,
                action_summary: "选择返回山门".to_string(),
                focus: "山门危机".to_string(),
                rationale: "应优先观察山门危机".to_string(),
                prompt_preview: "prompt".to_string(),
                proposal: crate::noname_types::NoNameProposal {
                    proposal_id: "proposal-summary-duplicate".to_string(),
                    kind: crate::noname_types::NoNameProposalKind::PlotCandidate,
                    producer_role: NoNameRole::Director,
                    title: "Director提案：山门危机".to_string(),
                    summary: "建议优先观察山门危机".to_string(),
                    focus: "山门危机".to_string(),
                    target_segment: crate::noname_types::NoNameTargetSegment::ChapterSummaryTail,
                    intended_effect: "避免重复摘要污染".to_string(),
                    rationale: "应优先观察山门危机".to_string(),
                    suggested_action: Some("进入低风险 apply".to_string()),
                    labels: vec!["director".to_string()],
                    apply_scopes: vec![crate::noname_types::NoNameApplyScope::ChapterSummaryHint],
                    status: crate::noname_types::NoNameProposalStatus::Applied,
                    applyable: true,
                },
            },
            proposal: crate::noname_types::NoNameProposal {
                proposal_id: "proposal-summary-duplicate".to_string(),
                kind: crate::noname_types::NoNameProposalKind::PlotCandidate,
                producer_role: NoNameRole::Director,
                title: "Director提案：山门危机".to_string(),
                summary: "建议优先观察山门危机".to_string(),
                focus: "山门危机".to_string(),
                target_segment: crate::noname_types::NoNameTargetSegment::ChapterSummaryTail,
                intended_effect: "避免重复摘要污染".to_string(),
                rationale: "应优先观察山门危机".to_string(),
                suggested_action: Some("进入低风险 apply".to_string()),
                labels: vec!["director".to_string()],
                apply_scopes: vec![crate::noname_types::NoNameApplyScope::ChapterSummaryHint],
                status: crate::noname_types::NoNameProposalStatus::Applied,
                applyable: true,
            },
            guardrail_result: None,
            related_observations: vec![],
        };

        apply_noname_low_risk_outputs(&mut plot_state, &mut result, false);

        assert_eq!(
            plot_state.current_chapter.summary,
            "已有摘要；NoName提示：后续重点关注山门危机"
        );
        assert!(result
            .trace
            .apply_execution_log
            .iter()
            .any(|item| item.target == "chapter_summary_hint"
                && item.outcome == "skipped_duplicate"));
    }

    #[test]
    fn test_apply_noname_option_bias_hint_skips_duplicate_diagnostics() {
        let existing_hint = "NoName选项偏置：下轮优先围绕山门危机提供行动切入点";
        let mut plot_state = PlotState {
            current_scene: crate::plot_engine::Scene {
                id: "scene-1".to_string(),
                name: "山门".to_string(),
                description: "山门风声渐紧".to_string(),
                location: "sect".to_string(),
                available_options: vec![make_option(0, "返回山门广场")],
            },
            plot_history: vec![],
            is_waiting_for_input: true,
            interaction_state: PlotInteractionState::WaitingForChoice,
            last_action_result: None,
            settings: PlotSettings::default(),
            current_chapter: crate::plot_engine::ChapterState::new(1, "第一章".to_string()),
            chapters: vec![],
            segment_count: 0,
            last_generation_diagnostics: Some(existing_hint.to_string()),
            last_option_generation_source: None,
            last_consistency_risk_score: None,
        };
        let mut result = NoNameDirectorRunResult {
            trace: NoNameTrace {
                trace_id: "trace-option-duplicate".to_string(),
                session_id: "session-1".to_string(),
                turn_id: "turn-1".to_string(),
                mode: NoNameMode::Assisted,
                graph_path: vec![crate::noname_types::NoNameTraceStage::ApplyProposal],
                capability_calls: vec![],
                proposals: vec![],
                proposal_transition_log: vec![],
                apply_plan_log: vec![],
                apply_execution_log: vec![],
                related_observations: vec![],
                protocol_events: vec![],
                guardrail_result: None,
                apply_result: None,
                fallback_used: false,
                elapsed_ms: 0,
            },
            observation: NoNameDirectorObservation {
                role: NoNameRole::Director,
                action_summary: "选择返回山门".to_string(),
                focus: "山门危机".to_string(),
                rationale: "应优先观察山门危机".to_string(),
                prompt_preview: "prompt".to_string(),
                proposal: crate::noname_types::NoNameProposal {
                    proposal_id: "proposal-option-duplicate".to_string(),
                    kind: crate::noname_types::NoNameProposalKind::PlotCandidate,
                    producer_role: NoNameRole::Director,
                    title: "Director提案：山门危机".to_string(),
                    summary: "建议优先观察山门危机".to_string(),
                    focus: "山门危机".to_string(),
                    target_segment: crate::noname_types::NoNameTargetSegment::CurrentTurnTail,
                    intended_effect: "避免重复偏置提示".to_string(),
                    rationale: "应优先观察山门危机".to_string(),
                    suggested_action: Some("进入低风险 apply".to_string()),
                    labels: vec!["director".to_string()],
                    apply_scopes: vec![crate::noname_types::NoNameApplyScope::OptionBiasHint],
                    status: crate::noname_types::NoNameProposalStatus::Applied,
                    applyable: true,
                },
            },
            proposal: crate::noname_types::NoNameProposal {
                proposal_id: "proposal-option-duplicate".to_string(),
                kind: crate::noname_types::NoNameProposalKind::PlotCandidate,
                producer_role: NoNameRole::Director,
                title: "Director提案：山门危机".to_string(),
                summary: "建议优先观察山门危机".to_string(),
                focus: "山门危机".to_string(),
                target_segment: crate::noname_types::NoNameTargetSegment::CurrentTurnTail,
                intended_effect: "避免重复偏置提示".to_string(),
                rationale: "应优先观察山门危机".to_string(),
                suggested_action: Some("进入低风险 apply".to_string()),
                labels: vec!["director".to_string()],
                apply_scopes: vec![crate::noname_types::NoNameApplyScope::OptionBiasHint],
                status: crate::noname_types::NoNameProposalStatus::Applied,
                applyable: true,
            },
            guardrail_result: None,
            related_observations: vec![],
        };

        apply_noname_low_risk_outputs(&mut plot_state, &mut result, false);

        assert_eq!(
            plot_state
                .last_generation_diagnostics
                .as_deref()
                .unwrap_or_default(),
            existing_hint
        );
        assert!(result
            .trace
            .apply_execution_log
            .iter()
            .any(|item| item.target == "option_bias_hint"
                && item.outcome == "skipped_duplicate"));
    }
}
