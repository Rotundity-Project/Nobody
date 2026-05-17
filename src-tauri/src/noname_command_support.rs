use crate::entity_store::EntityStore;
use crate::game_state::GameState;
use crate::memory_layers::MemoryLayers;
use crate::noname_apply::apply_noname_low_risk_outputs;
use crate::noname_context_builder::build_context_packet;
use crate::noname_context_types::NoNameContextBuildInput;
use crate::noname_guardrails::{NoNameDirectorGuardrailInput, NoNameGuardrailResult};
use crate::noname_memory_manager::NoNameMemoryManager;
use crate::noname_memory_types::NoNameWorkingMemoryItem;
use crate::noname_roles::NoNameDirectorObservation;
use crate::noname_runtime::{NoNameDirectorRunResult, NoNameRuntime, NoNameTurnInput};
use crate::noname_trace::NoNameTrace;
use crate::noname_types::{NoNameApplyScope, NoNameMode, NoNameRole};
use crate::numerical_system::ActionResult;
use crate::plot_engine::{PlayerAction, PlotState};

pub fn noname_mode_label(mode: NoNameMode) -> &'static str {
    match mode {
        NoNameMode::Disabled => "disabled",
        NoNameMode::ObserveOnly => "observe_only",
        NoNameMode::Assisted => "assisted",
    }
}

pub fn build_noname_action_summary(action: &PlayerAction, plot_state: &PlotState) -> String {
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

pub fn append_noname_observation_diagnostics(
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

pub fn build_noname_director_context_input(
    action_summary: &str,
    game_state: &GameState,
    plot_state: &PlotState,
) -> NoNameContextBuildInput {
    NoNameContextBuildInput {
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
    }
}

pub fn build_noname_director_guardrail_input(
    action_summary: &str,
    game_state: &GameState,
    plot_state: &PlotState,
) -> NoNameDirectorGuardrailInput {
    NoNameDirectorGuardrailInput {
        plot_state: plot_state.clone(),
        action_result: plot_state
            .last_action_result
            .clone()
            .unwrap_or(ActionResult {
                success: true,
                description: action_summary.to_string(),
                stat_changes: Vec::new(),
                events: Vec::new(),
            }),
        player_name: game_state.player.name.clone(),
        player_realm_level: game_state.player.stats.cultivation_realm.level,
        player_combat_power: game_state.player.stats.combat_power,
    }
}

pub fn run_noname_director_observe_turn(
    runtime: &mut NoNameRuntime,
    entity_store: &EntityStore,
    memory_layers: Option<&MemoryLayers>,
    action_summary: &str,
    game_state: &GameState,
    plot_state: &PlotState,
    timestamp: u64,
) -> Result<NoNameDirectorRunResult, String> {
    let mut memory_manager = NoNameMemoryManager::new();
    if let Some(memory) = memory_layers {
        memory_manager.ingest_legacy_layers(memory);
    }
    memory_manager.push_working_memory(
        NoNameWorkingMemoryItem {
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

    let context_input = build_noname_director_context_input(action_summary, game_state, plot_state);
    let context_packet = build_context_packet(entity_store, &memory_manager, &context_input);
    let guardrail_input =
        build_noname_director_guardrail_input(action_summary, game_state, plot_state);

    runtime
        .run_director_observe_turn(
            NoNameTurnInput {
                trace_id: format!("noname-trace-{}", timestamp),
                session_id: "active-run".to_string(),
                turn_id: format!("turn-{}", timestamp),
                caller_role: NoNameRole::Director,
            },
            action_summary,
            &context_packet,
            Some(&guardrail_input),
        )
        .map_err(|e| e.to_string())
}

pub fn apply_noname_turn_outputs(
    plot_state: &mut PlotState,
    noname_result: &mut NoNameDirectorRunResult,
    plot_text_applied: bool,
    pending_plot_augmentation_trace: Option<(String, String)>,
) {
    if let Some((outcome, note)) = pending_plot_augmentation_trace {
        noname_result.trace.record_apply_execution(
            NoNameApplyScope::PlotAugmentationHint.as_str(),
            outcome.as_str(),
            Some(note),
        );
        noname_result.trace.record_proposal_transition(format!(
            "{}:pending_plot_augmentation:{}",
            noname_result.proposal.proposal_id, outcome
        ));
    }
    apply_noname_low_risk_outputs(
        plot_state,
        &mut noname_result.trace,
        &noname_result.proposal,
        plot_text_applied,
    );
    append_noname_observation_diagnostics(
        &mut plot_state.last_generation_diagnostics,
        noname_result.trace.mode,
        &noname_result.observation,
        noname_result.guardrail_result.as_ref(),
        &noname_result.trace,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_engine::GameEngine;
    use crate::models::{CultivationRealm, Element, Grade, SpiritualRoot};
    use crate::noname_guardrails::NoNameGuardrailResult;
    use crate::noname_roles::NoNameDirectorObservation;
    use crate::noname_trace::{NoNameApplyTraceResult, NoNameTrace};
    use crate::noname_types::{
        NoNameApplyScope, NoNameMode, NoNameProposal, NoNameProposalKind, NoNameProposalStatus,
        NoNameRole, NoNameTargetSegment,
    };
    use crate::numerical_system::Action;
    use crate::plot_engine::{
        ActionType, ChapterState, PlayerAction, PlayerOption, PlotInteractionState, PlotSettings,
        PlotState, Scene,
    };
    use crate::script::{InitialState, Location, Script, ScriptType, WorldSetting};

    fn make_option(id: usize, description: &str) -> PlayerOption {
        PlayerOption {
            id,
            description: description.to_string(),
            requirements: Vec::new(),
            action: Action::Custom {
                description: description.to_string(),
            },
        }
    }

    fn make_plot_state() -> PlotState {
        let mut state = PlotState {
            current_scene: Scene {
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
            current_chapter: ChapterState::new(1, "第一章".to_string()),
            chapters: vec![],
            segment_count: 0,
            last_generation_diagnostics: None,
            last_option_generation_source: None,
            last_consistency_risk_score: None,
            pending_plot_augmentation_hints: Vec::new(),
        };
        state.current_chapter.content = vec![
            "第一段".to_string(),
            "第二段".to_string(),
            "第三段".to_string(),
        ];
        state
    }

    fn make_game_state() -> GameState {
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
        let script = Script::new(
            "world-1".to_string(),
            "测试世界".to_string(),
            ScriptType::Custom,
            world_setting,
            initial_state,
        );
        GameEngine::new()
            .initialize_game(script)
            .expect("test game should initialize")
    }

    fn make_observation() -> NoNameDirectorObservation {
        NoNameDirectorObservation {
            role: NoNameRole::Director,
            action_summary: "选择返回山门".to_string(),
            focus: "山门危机".to_string(),
            rationale: "应优先观察山门危机".to_string(),
            prompt_preview: "prompt".to_string(),
            role_goal: None,
            scene_focus: None,
            forbidden_scopes: Vec::new(),
            note_type_hits: Vec::new(),
            source_stats: Vec::new(),
            context_token_budget_used: None,
            context_slice_stats: Vec::new(),
            proposal: NoNameProposal {
                proposal_id: "proposal-1".to_string(),
                kind: NoNameProposalKind::PlotCandidate,
                producer_role: NoNameRole::Director,
                title: "Director提案：山门危机".to_string(),
                summary: "建议优先观察山门危机".to_string(),
                focus: "山门危机".to_string(),
                target_segment: NoNameTargetSegment::CurrentTurnTail,
                intended_effect: "维持低风险观察导向".to_string(),
                rationale: "应优先观察山门危机".to_string(),
                suggested_action: Some("保持 observe-only".to_string()),
                labels: vec!["director".to_string()],
                apply_scopes: vec![NoNameApplyScope::Diagnostics],
                status: NoNameProposalStatus::Observed,
                applyable: false,
            },
        }
    }

    fn make_director_run_result() -> NoNameDirectorRunResult {
        let observation = make_observation();
        let mut proposal = observation.proposal.clone();
        proposal.status = NoNameProposalStatus::Applied;
        proposal.applyable = true;
        proposal.apply_scopes = vec![
            NoNameApplyScope::ChapterSummaryHint,
            NoNameApplyScope::OptionBiasHint,
        ];
        NoNameDirectorRunResult {
            trace: NoNameTrace::empty("trace-outputs", "session-1", "turn-1", NoNameMode::Assisted),
            observation: NoNameDirectorObservation {
                proposal: proposal.clone(),
                ..observation
            },
            proposal,
            guardrail_result: Some(NoNameGuardrailResult::accept()),
            related_observations: Vec::new(),
        }
    }

    #[test]
    fn action_summary_prefers_selected_option_text() {
        let action = PlayerAction {
            action_type: ActionType::SelectedOption,
            content: String::new(),
            selected_option_id: Some(0),
            meta: None,
        };

        let summary = build_noname_action_summary(&action, &make_plot_state());

        assert_eq!(summary, "选择选项: 返回山门广场");
    }

    #[test]
    fn action_summary_uses_free_text_when_no_option_matches() {
        let action = PlayerAction {
            action_type: ActionType::FreeText,
            content: "  观察灵气流动  ".to_string(),
            selected_option_id: None,
            meta: None,
        };

        let summary = build_noname_action_summary(&action, &make_plot_state());

        assert_eq!(summary, "自由输入: 观察灵气流动");
    }

    #[test]
    fn diagnostics_append_mode_guardrail_apply_and_proposal_details() {
        let mut diagnostics = Some("原始诊断".to_string());
        let guardrail = NoNameGuardrailResult::accept();
        let mut trace =
            NoNameTrace::empty("trace-1", "session-1", "turn-1", NoNameMode::ObserveOnly);
        trace.apply_result = Some(NoNameApplyTraceResult {
            attempted: false,
            outcome: "skipped_observe_only".to_string(),
            reason: Some("当前模式仅观察，不尝试 assisted apply".to_string()),
        });

        append_noname_observation_diagnostics(
            &mut diagnostics,
            NoNameMode::ObserveOnly,
            &make_observation(),
            Some(&guardrail),
            &trace,
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
    fn director_context_input_keeps_recent_lines_and_player_identity() {
        let action_summary = "选择选项: 返回山门广场";
        let game_state = make_game_state();
        let plot_state = make_plot_state();

        let input = build_noname_director_context_input(action_summary, &game_state, &plot_state);

        assert_eq!(input.role, NoNameRole::Director);
        assert_eq!(input.world_id, "world-1");
        assert_eq!(input.run_id, "active-run");
        assert_eq!(input.scene_id, "scene-1");
        assert_eq!(input.character_ids, vec![game_state.player.id]);
        assert_eq!(input.map_node_id, Some(game_state.player.location));
        assert_eq!(input.player_intent.as_deref(), Some(action_summary));
        assert_eq!(
            input.recent_context_lines,
            vec![
                "第一段".to_string(),
                "第二段".to_string(),
                "第三段".to_string()
            ]
        );
        assert_eq!(input.token_budget, 320);
        assert_eq!(input.per_section_limit, 4);
    }

    #[test]
    fn director_guardrail_input_uses_last_action_or_safe_fallback() {
        let game_state = make_game_state();
        let plot_state = make_plot_state();

        let fallback =
            build_noname_director_guardrail_input("自由输入: 观察灵气", &game_state, &plot_state);

        assert_eq!(fallback.player_name, game_state.player.name);
        assert_eq!(
            fallback.player_realm_level,
            game_state.player.stats.cultivation_realm.level
        );
        assert_eq!(
            fallback.player_combat_power,
            game_state.player.stats.combat_power
        );
        assert_eq!(fallback.action_result.description, "自由输入: 观察灵气");
        assert!(fallback.action_result.success);

        let mut plot_state_with_result = plot_state.clone();
        plot_state_with_result.last_action_result = Some(ActionResult {
            success: false,
            description: "山门阵法拒绝回应".to_string(),
            stat_changes: Vec::new(),
            events: Vec::new(),
        });

        let from_result = build_noname_director_guardrail_input(
            "自由输入: 观察灵气",
            &game_state,
            &plot_state_with_result,
        );

        assert_eq!(from_result.action_result.description, "山门阵法拒绝回应");
        assert!(!from_result.action_result.success);
    }

    #[test]
    fn turn_outputs_apply_low_risk_outputs_and_pending_trace_marker() {
        let mut plot_state = make_plot_state();
        let mut result = make_director_run_result();

        apply_noname_turn_outputs(
            &mut plot_state,
            &mut result,
            false,
            Some((
                "pending_plot_augmentation_consumed".to_string(),
                "NoName pending plot augmentation consumed=1".to_string(),
            )),
        );

        assert!(plot_state
            .current_chapter
            .summary
            .contains("NoName提示：后续重点关注山门危机"));
        let diagnostics = plot_state
            .last_generation_diagnostics
            .as_deref()
            .unwrap_or_default();
        assert!(diagnostics.contains("NoName选项偏置"));
        assert!(diagnostics.contains("NoName.assisted"));
        assert!(result.trace.apply_execution_log.iter().any(|entry| {
            entry.target == "plot_augmentation_hint"
                && entry.outcome == "pending_plot_augmentation_consumed"
        }));
        assert!(result.trace.proposal_transition_log.iter().any(|entry| {
            entry == "proposal-1:pending_plot_augmentation:pending_plot_augmentation_consumed"
        }));
        assert_eq!(
            result
                .trace
                .apply_result
                .as_ref()
                .map(|entry| entry.outcome.as_str()),
            Some("applied_scoped_outputs")
        );
    }
}
