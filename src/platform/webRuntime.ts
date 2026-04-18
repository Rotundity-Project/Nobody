import {
  ActionType,
  Element,
  EventImportance,
  Grade,
  ScriptType,
  type ActionResult,
  type ConsistencyPolicy,
  type GameState,
  type MapLocationOverview,
  type NoNameApplyScope,
  type NoNameHumanReviewMarkPayload,
  type NoNameMode,
  type NoNameSecondGuardrailResolvePayload,
  type NoNameTargetSegment,
  type NoNameTrace,
  type PlayerAction,
  type PlayerOption,
  type PlotSettings,
  type PlotState,
  type SaveInfo,
  type Script,
  type WorldRegistry,
} from '../types/game';

type WebNoNameMode = NoNameMode;

type SaveSnapshot = {
  script: Script;
  gameState: GameState;
  plotState: PlotState;
  timestamp: number;
  noNameMode: WebNoNameMode;
};

type WebRuntimeState = {
  script: Script | null;
  gameState: GameState | null;
  plotState: PlotState | null;
  saves: Record<string, SaveSnapshot>;
  worldRegistry: WorldRegistry | null;
  consistencyPolicy: ConsistencyPolicy;
  noNameTraces: NoNameTrace[];
  noNameMode: WebNoNameMode;
};

const STORAGE_KEY = 'nobody_web_runtime_v1';

const defaultPlotSettings = (): PlotSettings => ({
  recap_enabled: true,
  novel_style: 'xianxia-third-person',
  llm_priority_mode: true,
  llm_strict_mode: false,
  min_interactions_per_chapter: 2,
  max_interactions_per_chapter: 4,
  target_chapter_words_min: 800,
  target_chapter_words_max: 1600,
});

const defaultConsistencyPolicy = (): ConsistencyPolicy => ({
  recent_window: 3,
  cross_chapter_window: 3,
  duplicate_recent_threshold: 0.88,
  duplicate_cross_chapter_threshold: 0.88,
  weight_warning: 5,
  weight_critical: 12,
  code_weights: {
    duplicate_segment: 8,
    duplicate_cross_chapter: 10,
    waiting_without_options: 12,
  },
});

const initialState = (): WebRuntimeState => ({
  script: null,
  gameState: null,
  plotState: null,
  saves: {},
  worldRegistry: null,
  consistencyPolicy: defaultConsistencyPolicy(),
  noNameTraces: [],
  noNameMode: 'observeOnly',
});

let runtimeState: WebRuntimeState = loadState();

function loadState(): WebRuntimeState {
  if (typeof localStorage === 'undefined') {
    return initialState();
  }
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return initialState();
    const parsed = JSON.parse(raw) as Partial<WebRuntimeState>;
    return {
      script: parsed.script ?? null,
      gameState: parsed.gameState ?? null,
      plotState: parsed.plotState ?? null,
      saves: parsed.saves ?? {},
      worldRegistry: parsed.worldRegistry ?? null,
      consistencyPolicy: parsed.consistencyPolicy ?? defaultConsistencyPolicy(),
      noNameTraces: parsed.noNameTraces ?? [],
      noNameMode: parsed.noNameMode ?? 'observeOnly',
    };
  } catch {
    return initialState();
  }
}

function persistState() {
  if (typeof localStorage === 'undefined') {
    return;
  }
  localStorage.setItem(STORAGE_KEY, JSON.stringify(runtimeState));
}

function ensureStateReady() {
  if (!runtimeState.gameState || !runtimeState.plotState || !runtimeState.script) {
    throw new Error('Web 模式下尚未初始化游戏，请先开始新游戏。');
  }
}

function buildDefaultScript(): Script {
  return {
    id: `web_script_${Date.now()}`,
    name: 'Web 试炼剧本',
    script_type: ScriptType.Custom,
    world_setting: {
      cultivation_realms: [
        { name: '练气', level: 1, sub_level: 0, power_multiplier: 1 },
        { name: '筑基', level: 2, sub_level: 0, power_multiplier: 1.8 },
      ],
      spiritual_roots: [
        { element: Element.Fire, elements: [Element.Fire], grade: Grade.Double, affinity: 0.72 },
      ],
      techniques: [
        {
          id: 'breath_core',
          name: '吐纳诀',
          description: '最基础的灵气吐纳法门。',
          required_realm_level: 1,
          element: null,
        },
      ],
      locations: [
        { id: 'sect_valley', name: '宗门外谷', description: '山风平稳，灵气充足。', spiritual_energy: 1.1 },
        { id: 'stone_forest', name: '乱石林', description: '怪石密布，偶有灵兽。', spiritual_energy: 1.3 },
        { id: 'market', name: '云岚坊市', description: '散修往来，消息繁杂。', spiritual_energy: 0.9 },
      ],
      factions: [{ id: 'qingyun', name: '青云宗', description: '正道宗门。', power_level: 72 }],
    },
    initial_state: {
      player_name: '无名弟子',
      player_spiritual_root: {
        element: Element.Fire,
        elements: [Element.Fire],
        grade: Grade.Double,
        affinity: 0.72,
      },
      starting_location: 'sect_valley',
      starting_age: 16,
    },
  };
}

function buildGameState(script: Script): GameState {
  const locations = Object.fromEntries(script.world_setting.locations.map((loc) => [loc.id, loc]));
  const factions = Object.fromEntries(script.world_setting.factions.map((f) => [f.id, f]));
  return {
    script,
    player: {
      id: 'player',
      name: script.initial_state.player_name,
      stats: {
        spiritual_root: script.initial_state.player_spiritual_root,
        cultivation_realm: script.world_setting.cultivation_realms[0] ?? {
          name: '练气',
          level: 1,
          sub_level: 0,
          power_multiplier: 1,
        },
        techniques: ['吐纳诀'],
        lifespan: {
          current_age: script.initial_state.starting_age,
          max_age: 120,
          realm_bonus: 0,
        },
        combat_power: 100,
      },
      inventory: ['基础灵石袋'],
      location: script.initial_state.starting_location,
      combat_status: {
        injury_level: 0,
        reputation: 0,
        enmity: 0,
        qi_deviation: 0,
      },
      social_profile: {
        sect_affinity: 10,
        mentor_bond: 5,
        vendetta: 0,
        favor: 5,
        camp_stance: 'neutral',
      },
    },
    world_state: {
      locations,
      factions,
      global_events: [],
    },
    game_time: {
      year: 1,
      month: 1,
      day: 1,
      total_days: 1,
    },
    event_history: [],
  };
}

function buildInitialOptions(): PlayerOption[] {
  return [
    {
      id: 0,
      description: '在原地打坐，稳固气息',
      requirements: [],
      action: { Custom: { description: '打坐' } },
    },
    {
      id: 1,
      description: '前往乱石林探索',
      requirements: [],
      action: { Custom: { description: '探索' } },
    },
    {
      id: 2,
      description: '去坊市打听消息',
      requirements: [],
      action: { Custom: { description: '打听' } },
    },
  ];
}

function buildPlotState(gameState: GameState): PlotState {
  const currentLocation = gameState.world_state.locations[gameState.player.location];
  const opening = `你踏入${currentLocation?.name ?? '陌生地界'}，灵气在经脉中缓缓流动。`;
  return {
    current_scene: {
      id: 'scene_1',
      name: currentLocation?.name ?? '初始场景',
      description: currentLocation?.description ?? '一切刚刚开始。',
      location: gameState.player.location,
      available_options: buildInitialOptions(),
    },
    plot_history: [opening],
    is_waiting_for_input: true,
    interaction_state: 'waiting_for_choice',
    last_action_result: null,
    last_generation_diagnostics: '链路：web_mock',
    last_option_generation_source: 'web_mock_rule',
    last_consistency_risk_score: 0,
    settings: defaultPlotSettings(),
    current_chapter: {
      index: 1,
      title: '第一章·入门',
      content: [opening],
      summary: '你开始了修行旅程。',
      interaction_count: 0,
      status: 'in_progress',
    },
    chapters: [],
    segment_count: 1,
  };
}

function refreshWorldRegistry() {
  if (!runtimeState.gameState) {
    runtimeState.worldRegistry = null;
    return;
  }
  runtimeState.worldRegistry = {
    session_id: 'web-session',
    seed: 20260304,
    created_at: Date.now(),
    llm_model: 'web-mock-rule-engine',
    source: 'web_mock',
    tables: {
      characters: [{ id: runtimeState.gameState.player.id, name: runtimeState.gameState.player.name }],
      map_nodes: runtimeState.gameState.script.world_setting.locations.map((loc) => ({
        id: loc.id,
        name: loc.name,
      })),
      map_edges: [],
      techniques: runtimeState.gameState.script.world_setting.techniques.map((t) => ({ id: t.id, name: t.name })),
      inventory_items: runtimeState.gameState.player.inventory.map((name) => ({ name })),
      factions: runtimeState.gameState.script.world_setting.factions.map((f) => ({ id: f.id, name: f.name })),
      story_state: [{ chapter: runtimeState.plotState?.current_chapter.title ?? '第一章' }],
      world_facts: [{ key: 'runtime', value: 'web_mock' }],
    },
  };
}

function createActionResult(description: string): ActionResult {
  return {
    success: true,
    description,
    stat_changes: [],
    events: [],
  };
}

function pickWebNoNameTargetSegment(action: PlayerAction): NoNameTargetSegment {
  if (action.action_type === ActionType.FreeText) {
    return 'current_turn_head';
  }
  if (typeof action.selected_option_id === 'number' && action.selected_option_id % 2 === 0) {
    return 'chapter_summary_head';
  }
  return 'current_turn_tail';
}

function deriveWebNoNameApplyScopes(targetSegment: NoNameTargetSegment): NoNameApplyScope[] {
  const baseScopes: NoNameApplyScope[] = ['diagnostics', 'chapterSummaryHint', 'optionBiasHint'];
  if (
    targetSegment === 'current_turn_head' ||
    targetSegment === 'current_turn_tail'
  ) {
    return [...baseScopes, 'plotTextHint'];
  }
  return baseScopes;
}

function applyWebNoNameSummaryHint(summary: string, focus: string, targetSegment: NoNameTargetSegment): string {
  const hint = `NoName提示：后续重点关注${focus}`;
  if (!summary.trim()) {
    return hint;
  }
  if (targetSegment === 'chapter_summary_head') {
    return `${hint}；${summary}`;
  }
  if (summary.includes(focus)) {
    return summary;
  }
  return `${summary}；${hint}`;
}

function appendWebNoNameTrace(
  text: string,
  paragraph: string,
  targetSegment: NoNameTargetSegment,
  applyScopes: NoNameApplyScope[],
) {
  const traceId = `web-trace-${Date.now()}`;
  const proposalId = `proposal-${traceId}`;

  if (runtimeState.noNameMode === 'disabled') {
    return;
  }

  if (runtimeState.noNameMode === 'assisted') {
    runtimeState.noNameTraces.push({
      traceId,
      sessionId: 'web-session',
      turnId: `turn-${Date.now()}`,
      mode: 'assisted',
      graphPath: [
        'CollectTurnInput',
        'BuildContextBundle',
        'PlanTurn',
        'ValidateProposal',
        'ApplyProposal',
        'PersistTrace',
      ],
      capabilityCalls: [],
      proposals: [
        {
          proposalId,
          kind: 'plot_candidate',
          producerRole: 'director',
          title: `Director提案：${text}`,
          summary: `建议优先推进“${text}”`,
          focus: text,
          targetSegment,
          intendedEffect: '为低风险输出提供稳定导向',
          rationale: 'web mock: assisted apply',
          suggestedAction: '进入低风险 apply',
          labels: ['director', 'assisted_ready', 'apply_scope_diagnostics'],
          applyScopes,
          status: 'applied',
          applyable: true,
        },
      ],
      proposalTransitionLog: [
        `${proposalId}:ready`,
        `${proposalId}:apply_preflight:ready`,
        ...applyScopes
          .filter((scope) => scope !== 'plotTextHint')
          .map((scope) => `${proposalId}:applied:${scope}`),
        ...(applyScopes.includes('plotTextHint')
          ? [`${proposalId}:controlled_output:plot_text_hint:needs_review`]
          : []),
      ],
      applyPlanLog: applyScopes
        .map((scope) => ({
        target: scope,
        decision: scope === 'plotTextHint' ? 'needs_review' : 'apply',
        priority: scope === 'plotTextHint' ? 300 : scope === 'chapterSummaryHint' ? 200 : scope === 'optionBiasHint' ? 100 : 50,
        note: scope === 'plotTextHint'
          ? 'web mock plotTextHint 需要人工复核'
          : `web mock 允许执行 ${scope}`,
      }))
        .sort((left, right) => right.priority - left.priority)
        .map((item, index) => ({ ...item, order: index + 1 })),
      applyExecutionLog: applyScopes
        .filter((scope) => scope !== 'plotTextHint')
        .map((scope) => ({
        target: scope,
        outcome: 'applied',
        note: scope === 'chapterSummaryHint'
            ? `已补充章节摘要提示，聚焦“${text}”`
            : scope === 'optionBiasHint'
              ? `已补充下轮选项偏置提示，聚焦“${text}”`
              : `已补充诊断提示，聚焦“${text}”`,
      })),
      controlledOutputReviews: applyScopes.map((scope) => ({
        requestId: `controlled-output-${proposalId}-${scope === 'plotTextHint' ? 'plot_text_hint' : scope}`,
        requestedKind: scope === 'plotTextHint'
          ? 'sceneAugmentation'
          : scope === 'chapterSummaryHint'
            ? 'recapNote'
            : scope === 'optionBiasHint'
              ? 'intermediateNarrativeHint'
              : 'narrativeNote',
        decision: scope === 'plotTextHint' ? 'needsReview' : 'allow',
        reason: scope === 'plotTextHint'
          ? 'plot text hint requires human review before higher-layer apply'
          : 'controlled output stays within allowed boundary',
        normalizedKind: scope === 'plotTextHint'
          ? 'sceneAugmentation'
          : scope === 'chapterSummaryHint'
            ? 'recapNote'
            : scope === 'optionBiasHint'
              ? 'intermediateNarrativeHint'
              : 'narrativeNote',
        safeApplyScope: scope,
        policyForbiddenScopes: ['finalPlotState', 'canonWorldFact'],
        requiresHumanReview: scope === 'plotTextHint',
      })),
      guardrailResult: { outcome: 'accept' },
      applyResult: {
        attempted: true,
        outcome: 'applied_summary_and_option_bias',
        reason: `web mock 已将提案应用到低风险输出层，plotTextHint 等待人工复核，target=${targetSegment}，scopes=${applyScopes.join(',')}，正文为：${paragraph}`,
      },
      fallbackUsed: false,
      elapsedMs: 0,
    });
  } else {
    runtimeState.noNameTraces.push({
      traceId,
      sessionId: 'web-session',
      turnId: `turn-${Date.now()}`,
      mode: 'observeOnly',
      graphPath: ['CollectTurnInput', 'BuildContextBundle', 'PlanTurn', 'PersistTrace'],
      capabilityCalls: [],
      proposals: [
        {
          proposalId,
          kind: 'plot_candidate',
          producerRole: 'director',
          title: `Director提案：${text}`,
          summary: `建议优先观察“${text}”`,
          focus: text,
          targetSegment,
          intendedEffect: '维持观察链路，不直接改写主剧情',
          rationale: 'web mock: observe-only',
          suggestedAction: '保持 observe-only',
          labels: ['director', 'observe_only'],
          applyScopes,
          status: 'observed',
          applyable: false,
        },
      ],
      proposalTransitionLog: [`${proposalId}:observed`],
      applyPlanLog: applyScopes
        .map((scope) => ({
        target: scope,
        decision: 'skip',
        priority: scope === 'plotTextHint' ? 300 : scope === 'chapterSummaryHint' ? 200 : scope === 'optionBiasHint' ? 100 : 50,
        note: `web mock observe-only：${scope} 仅记录不执行`,
      }))
        .sort((left, right) => right.priority - left.priority)
        .map((item, index) => ({ ...item, order: index + 1 })),
      applyExecutionLog: [],
      guardrailResult: { outcome: 'accept' },
      applyResult: { attempted: false, outcome: 'skipped_observe_only' },
      fallbackUsed: false,
      elapsedMs: 0,
    });
  }

  if (runtimeState.noNameTraces.length > 8) {
    runtimeState.noNameTraces = runtimeState.noNameTraces.slice(-8);
  }
}

function markWebNoNameControlledOutputReview(payload: NoNameHumanReviewMarkPayload): NoNameTrace {
  const trace = runtimeState.noNameTraces.find((item) => item.traceId === payload.traceId);
  if (!trace) {
    throw new Error(`NoName trace not found: ${payload.traceId}`);
  }
  const review = (trace.controlledOutputReviews ?? [])
    .find((item) => item.requestId === payload.requestId);
  if (!review) {
    throw new Error(`NoName controlled output review not found: ${payload.requestId}`);
  }
  if (review.decision !== 'needsReview' || !review.requiresHumanReview) {
    throw new Error(`NoName controlled output review does not require human review: ${payload.requestId}`);
  }

  review.humanReviewDecision = payload.decision;
  review.humanReviewedAt = Math.floor(Date.now() / 1000);
  review.humanReviewNote = payload.decision === 'approvedForHigherApply'
    ? '人工确认可进入高层 apply 设计，仍需后端二次 guardrail'
    : payload.decision === 'rejectedForHigherApply'
      ? '人工确认暂不应用，保持当前安全边界'
      : '人工复核已重置为待确认，未触发高层 apply';
  trace.proposalTransitionLog = [
    ...(trace.proposalTransitionLog ?? []),
    `${payload.requestId}:human_review:${payload.decision}`,
  ];
  const target = review.safeApplyScope ?? 'controlled_output';
  const nextOrder = Math.max(0, ...(trace.applyPlanLog ?? []).map((item) => item.order)) + 1;
  const priority = review.safeApplyScope === 'plotTextHint'
    ? 325
    : review.safeApplyScope === 'chapterSummaryHint'
      ? 225
      : review.safeApplyScope === 'optionBiasHint'
        ? 125
        : 35;

  if (payload.decision === 'pending') {
    trace.applyPlanLog = [
      ...(trace.applyPlanLog ?? []),
      {
        order: nextOrder,
        target,
        decision: 'review_intent_pending',
        priority,
        note: '人工复核已重置为待确认，未进入二次 guardrail',
      },
    ];
    trace.applyExecutionLog = [
      ...(trace.applyExecutionLog ?? []),
      {
        target,
        outcome: 'human_review_pending',
        note: '等待人工确认，不触发高层 apply',
      },
    ];
    trace.proposalTransitionLog.push(`${payload.requestId}:apply_intent:pending`);
    return trace;
  }

  if (payload.decision === 'rejectedForHigherApply') {
    trace.applyPlanLog = [
      ...(trace.applyPlanLog ?? []),
      {
        order: nextOrder,
        target,
        decision: 'reject',
        priority,
        note: '人工复核拒绝进入高层 apply，保持安全边界',
      },
    ];
    trace.applyExecutionLog = [
      ...(trace.applyExecutionLog ?? []),
      {
        target,
        outcome: 'rejected_by_human_review',
        note: '开发者选择暂不应用，未触发二次 guardrail',
      },
    ];
    trace.proposalTransitionLog.push(`${payload.requestId}:apply_intent:rejected_by_human_review`);
    return trace;
  }

  const proposal = trace.proposals
    .slice()
    .reverse()
    .find((item) => payload.requestId.includes(item.proposalId))
    ?? trace.proposals[trace.proposals.length - 1];
  const secondGuardrailRejected = trace.mode !== 'assisted'
    || review.safeApplyScope !== 'plotTextHint'
    || !proposal
    || proposal.status !== 'applied'
    || !proposal.applyable
    || !['current_turn_head', 'current_turn_tail'].includes(proposal.targetSegment);
  if (secondGuardrailRejected) {
    trace.applyPlanLog = [
      ...(trace.applyPlanLog ?? []),
      {
        order: nextOrder,
        target,
        decision: 'second_guardrail_reject',
        priority,
        note: 'web mock 二次 guardrail 拒绝高层 apply intent',
      },
    ];
    trace.applyExecutionLog = [
      ...(trace.applyExecutionLog ?? []),
      {
        target,
        outcome: 'second_guardrail_rejected',
        note: 'web mock 未满足高层 apply intent 条件',
      },
    ];
    trace.proposalTransitionLog.push(`${payload.requestId}:apply_intent:second_guardrail_rejected`);
    return trace;
  }

  trace.applyPlanLog = [
    ...(trace.applyPlanLog ?? []),
    {
      order: nextOrder,
      target,
      decision: 'review_intent_ready',
      priority,
      note: '人工确认已通过，已排入二次 guardrail / apply planner，未写入正文',
    },
  ];
  trace.applyExecutionLog = [
    ...(trace.applyExecutionLog ?? []),
    {
      target,
      outcome: 'awaiting_second_guardrail',
      note: '高层 apply intent 已记录，等待后续二次 guardrail 决策',
    },
  ];
  trace.proposalTransitionLog.push(`${payload.requestId}:apply_intent:awaiting_second_guardrail`);
  return trace;
}

function resolveWebNoNameSecondGuardrail(payload: NoNameSecondGuardrailResolvePayload): NoNameTrace {
  const trace = runtimeState.noNameTraces.find((item) => item.traceId === payload.traceId);
  if (!trace) {
    throw new Error(`NoName trace not found: ${payload.traceId}`);
  }
  const review = (trace.controlledOutputReviews ?? [])
    .find((item) => item.requestId === payload.requestId);
  if (!review) {
    throw new Error(`NoName controlled output review not found: ${payload.requestId}`);
  }
  if (review.humanReviewDecision !== 'approvedForHigherApply') {
    throw new Error(`NoName controlled output review is not approved for second guardrail: ${payload.requestId}`);
  }

  const target = review.safeApplyScope ?? 'controlled_output';
  const nextOrder = Math.max(0, ...(trace.applyPlanLog ?? []).map((item) => item.order)) + 1;
  const priority = review.safeApplyScope === 'plotTextHint'
    ? 350
    : review.safeApplyScope === 'chapterSummaryHint'
      ? 250
      : review.safeApplyScope === 'optionBiasHint'
        ? 150
        : 60;
  const isWaiting = (trace.proposalTransitionLog ?? [])
    .includes(`${payload.requestId}:apply_intent:awaiting_second_guardrail`)
    || (trace.applyExecutionLog ?? []).some((item) => item.outcome === 'awaiting_second_guardrail');
  const proposal = trace.proposals
    .slice()
    .reverse()
    .find((item) => payload.requestId.includes(item.proposalId))
    ?? trace.proposals[trace.proposals.length - 1];
  const revalidateRejected = !isWaiting
    || trace.mode !== 'assisted'
    || review.safeApplyScope !== 'plotTextHint'
    || !proposal
    || proposal.status !== 'applied'
    || !proposal.applyable
    || !['current_turn_head', 'current_turn_tail'].includes(proposal.targetSegment);

  const finalDecision = revalidateRejected ? 'reject' : payload.decision;
  const planDecision = finalDecision === 'allow'
    ? 'second_guardrail_allow'
    : finalDecision === 'fallback'
      ? 'second_guardrail_fallback'
      : 'second_guardrail_reject';
  const outcome = finalDecision === 'allow'
    ? 'second_guardrail_allowed'
    : finalDecision === 'fallback'
      ? 'second_guardrail_fallback'
      : 'second_guardrail_rejected';
  const note = revalidateRejected
    ? 'web mock 二次 guardrail 复核未通过'
    : finalDecision === 'allow'
      ? '二次 guardrail 允许进入后续人工 apply 命令；当前不写正文'
      : finalDecision === 'fallback'
        ? '二次 guardrail 要求回退经典链路'
        : '二次 guardrail 人工拒绝高层 apply';

  trace.applyPlanLog = [
    ...(trace.applyPlanLog ?? []),
    {
      order: nextOrder,
      target,
      decision: planDecision,
      priority,
      note,
    },
  ];
  trace.applyExecutionLog = [
    ...(trace.applyExecutionLog ?? []),
    {
      target,
      outcome,
      note: finalDecision === 'allow'
        ? '已允许进入下一步人工 apply，但未改写剧情正文'
        : note,
    },
  ];
  trace.proposalTransitionLog = [
    ...(trace.proposalTransitionLog ?? []),
    `${payload.requestId}:second_guardrail:${finalDecision}`,
  ];
  trace.applyResult = {
    attempted: true,
    outcome: planDecision,
    reason: note,
  };
  if (finalDecision === 'fallback') {
    trace.fallbackUsed = true;
    trace.graphPath = [...trace.graphPath, 'ApplyFallback'];
  }
  return trace;
}

function applyWebNoNameManualPlotTextHint(args: {
  traceId?: unknown;
  requestId?: unknown;
  chapterIndex?: unknown;
  segmentIndex?: unknown;
  expectedSegmentText?: unknown;
}) {
  const traceId = String(args.traceId ?? '');
  const requestId = String(args.requestId ?? '');
  const chapterIndex = Number(args.chapterIndex);
  const segmentIndex = Number(args.segmentIndex);
  const expectedSegmentText = String(args.expectedSegmentText ?? '');
  const trace = runtimeState.noNameTraces.find((item) => item.traceId === traceId);
  if (!trace) {
    throw new Error(`NoName trace not found: ${traceId}`);
  }
  const review = (trace.controlledOutputReviews ?? []).find((item) => item.requestId === requestId);
  if (!review) {
    throw new Error(`NoName controlled output review not found: ${requestId}`);
  }
  if (review.humanReviewDecision !== 'approvedForHigherApply') {
    throw new Error(`NoName controlled output review is not approved for manual apply: ${requestId}`);
  }
  if (review.safeApplyScope !== 'plotTextHint') {
    throw new Error('manual apply currently only supports plotTextHint');
  }
  const hasSecondGuardrailAllow = (trace.proposalTransitionLog ?? [])
    .includes(`${requestId}:second_guardrail:allow`)
    || (trace.applyExecutionLog ?? []).some((item) => (
      item.target === 'plotTextHint' && item.outcome === 'second_guardrail_allowed'
    ));
  if (!hasSecondGuardrailAllow) {
    throw new Error('second guardrail has not allowed this review');
  }
  const plotState = runtimeState.plotState;
  if (!plotState) {
    throw new Error('Web 模式下剧情未初始化。');
  }
  if (plotState.current_chapter.index !== chapterIndex) {
    throw new Error(`chapter mismatch: expected ${chapterIndex}, current ${plotState.current_chapter.index}`);
  }
  if (plotState.current_chapter.content[segmentIndex] !== expectedSegmentText) {
    throw new Error('segment snapshot mismatch; refusing stale manual apply');
  }
  if (expectedSegmentText.includes('【NoName】') || expectedSegmentText.includes('NoName提示')) {
    throw new Error('segment already contains a NoName marker');
  }
  const proposal = trace.proposals
    .slice()
    .reverse()
    .find((item) => requestId.includes(item.proposalId))
    ?? trace.proposals[trace.proposals.length - 1];
  if (!proposal) {
    throw new Error('NoName proposal not found for manual apply');
  }
  const hint = `【NoName】重点关注：${proposal.focus}`;
  const updatedSegment = proposal.targetSegment === 'current_turn_head'
    ? `${hint}\n\n${expectedSegmentText.trim()}`
    : `${expectedSegmentText.trim()}\n\n${hint}`;
  plotState.current_chapter.content[segmentIndex] = updatedSegment;
  const historyIndex = plotState.plot_history.lastIndexOf(expectedSegmentText);
  if (historyIndex < 0) {
    throw new Error('plot history snapshot mismatch; refusing partial manual apply');
  }
  plotState.plot_history[historyIndex] = updatedSegment;
  plotState.current_scene.description = plotState.current_chapter.content.join('\n\n');
  if (plotState.last_action_result?.description === expectedSegmentText) {
    plotState.last_action_result.description = updatedSegment;
  }

  const nextOrder = Math.max(0, ...(trace.applyPlanLog ?? []).map((item) => item.order)) + 1;
  trace.applyPlanLog = [
    ...(trace.applyPlanLog ?? []),
    {
      order: nextOrder,
      target: 'plotTextHint',
      decision: 'manual_apply',
      priority: 375,
      note: `显式人工 apply 已确认 chapter=${chapterIndex} segment=${segmentIndex}`,
    },
  ];
  trace.applyExecutionLog = [
    ...(trace.applyExecutionLog ?? []),
    {
      target: 'plotTextHint',
      outcome: 'manual_plot_text_applied',
      note: `已由显式人工命令写入正文提示，聚焦“${proposal.focus}”`,
    },
  ];
  trace.proposalTransitionLog = [
    ...(trace.proposalTransitionLog ?? []),
    `${requestId}:manual_apply:plot_text_hint`,
  ];
  trace.applyResult = {
    attempted: true,
    outcome: 'manual_plot_text_applied',
    reason: '显式人工 apply 已写入正文提示',
  };
  return { trace, plotState };
}

function applyWebNoNameManualChapterSummaryHint(args: {
  traceId?: unknown;
  requestId?: unknown;
  chapterIndex?: unknown;
  expectedSummary?: unknown;
}) {
  const traceId = String(args.traceId ?? '');
  const requestId = String(args.requestId ?? '');
  const chapterIndex = Number(args.chapterIndex);
  const expectedSummary = String(args.expectedSummary ?? '');
  const trace = runtimeState.noNameTraces.find((item) => item.traceId === traceId);
  if (!trace) {
    throw new Error(`NoName trace not found: ${traceId}`);
  }
  const review = (trace.controlledOutputReviews ?? []).find((item) => item.requestId === requestId);
  if (!review) {
    throw new Error(`NoName controlled output review not found: ${requestId}`);
  }
  if (review.humanReviewDecision !== 'approvedForHigherApply') {
    throw new Error(`NoName controlled output review is not approved for manual apply: ${requestId}`);
  }
  if (review.safeApplyScope !== 'chapterSummaryHint') {
    throw new Error('manual apply scope mismatch: expected chapterSummaryHint');
  }
  const hasSecondGuardrailAllow = (trace.proposalTransitionLog ?? [])
    .includes(`${requestId}:second_guardrail:allow`)
    || (trace.applyExecutionLog ?? []).some((item) => (
      item.target === 'chapterSummaryHint' && item.outcome === 'second_guardrail_allowed'
    ));
  if (!hasSecondGuardrailAllow) {
    throw new Error('second guardrail has not allowed this review');
  }
  if ((trace.applyExecutionLog ?? []).some((item) => (
    item.target === 'chapterSummaryHint' && item.outcome === 'manual_chapter_summary_hint_applied'
  ))) {
    throw new Error('manual chapterSummaryHint has already been applied for this trace');
  }
  const plotState = runtimeState.plotState;
  if (!plotState) {
    throw new Error('Web runtime plot state is not initialized');
  }
  if (plotState.current_chapter.index !== chapterIndex) {
    throw new Error(`chapter mismatch: expected ${chapterIndex}, current ${plotState.current_chapter.index}`);
  }
  if (plotState.current_chapter.summary !== expectedSummary) {
    throw new Error('summary snapshot mismatch; refusing stale manual apply');
  }
  const proposal = trace.proposals
    .slice()
    .reverse()
    .find((item) => requestId.includes(item.proposalId))
    ?? trace.proposals[trace.proposals.length - 1];
  if (!proposal) {
    throw new Error('NoName proposal not found for manual apply');
  }
  const focus = proposal.focus.trim();
  const hint = `NoName summary hint: ${focus}`;
  if (focus && (expectedSummary.includes(focus) || expectedSummary.includes(hint))) {
    throw new Error('chapter summary already contains this NoName hint');
  }
  const currentSummary = expectedSummary.trim();
  const updatedSummary = !currentSummary
    ? hint
    : proposal.targetSegment === 'chapter_summary_head'
      ? `${hint}; ${currentSummary}`
      : `${currentSummary}; ${hint}`;

  plotState.current_chapter.summary = updatedSummary;
  const chapter = plotState.chapters.find((item) => item.index === chapterIndex);
  if (chapter) {
    chapter.summary = updatedSummary;
  }

  const nextOrder = Math.max(0, ...(trace.applyPlanLog ?? []).map((item) => item.order)) + 1;
  trace.applyPlanLog = [
    ...(trace.applyPlanLog ?? []),
    {
      order: nextOrder,
      target: 'chapterSummaryHint',
      decision: 'manual_apply',
      priority: 275,
      note: `manual apply confirmed for chapter=${chapterIndex} scope=chapterSummaryHint`,
    },
  ];
  trace.applyExecutionLog = [
    ...(trace.applyExecutionLog ?? []),
    {
      target: 'chapterSummaryHint',
      outcome: 'manual_chapter_summary_hint_applied',
      note: `manual chapter summary hint applied for focus=${proposal.focus}`,
    },
  ];
  trace.proposalTransitionLog = [
    ...(trace.proposalTransitionLog ?? []),
    `${requestId}:manual_apply:chapter_summary_hint`,
  ];
  trace.applyResult = {
    attempted: true,
    outcome: 'manual_chapter_summary_hint_applied',
    reason: 'manual chapter summary hint applied',
  };
  return { trace, plotState };
}

function applyWebNoNameManualOptionBiasHint(args: {
  traceId?: unknown;
  requestId?: unknown;
  chapterIndex?: unknown;
  expectedGenerationDiagnostics?: unknown;
}) {
  const traceId = String(args.traceId ?? '');
  const requestId = String(args.requestId ?? '');
  const chapterIndex = Number(args.chapterIndex);
  const expectedGenerationDiagnostics = String(args.expectedGenerationDiagnostics ?? '');
  const trace = runtimeState.noNameTraces.find((item) => item.traceId === traceId);
  if (!trace) {
    throw new Error(`NoName trace not found: ${traceId}`);
  }
  const review = (trace.controlledOutputReviews ?? []).find((item) => item.requestId === requestId);
  if (!review) {
    throw new Error(`NoName controlled output review not found: ${requestId}`);
  }
  if (review.humanReviewDecision !== 'approvedForHigherApply') {
    throw new Error(`NoName controlled output review is not approved for manual apply: ${requestId}`);
  }
  if (review.safeApplyScope !== 'optionBiasHint') {
    throw new Error('manual apply scope mismatch: expected optionBiasHint');
  }
  const hasSecondGuardrailAllow = (trace.proposalTransitionLog ?? [])
    .includes(`${requestId}:second_guardrail:allow`)
    || (trace.applyExecutionLog ?? []).some((item) => (
      item.target === 'optionBiasHint' && item.outcome === 'second_guardrail_allowed'
    ));
  if (!hasSecondGuardrailAllow) {
    throw new Error('second guardrail has not allowed this review');
  }
  if ((trace.applyExecutionLog ?? []).some((item) => (
    item.target === 'optionBiasHint' && item.outcome === 'manual_option_bias_hint_applied'
  ))) {
    throw new Error('manual optionBiasHint has already been applied for this trace');
  }
  const plotState = runtimeState.plotState;
  if (!plotState) {
    throw new Error('Web runtime plot state is not initialized');
  }
  if (plotState.current_chapter.index !== chapterIndex) {
    throw new Error(`chapter mismatch: expected ${chapterIndex}, current ${plotState.current_chapter.index}`);
  }
  if (!plotState.is_waiting_for_input) {
    throw new Error('optionBiasHint manual apply requires waiting-for-input state');
  }
  const currentDiagnostics = plotState.last_generation_diagnostics ?? '';
  if (currentDiagnostics !== expectedGenerationDiagnostics) {
    throw new Error('diagnostics snapshot mismatch; refusing stale manual apply');
  }
  const proposal = trace.proposals
    .slice()
    .reverse()
    .find((item) => requestId.includes(item.proposalId))
    ?? trace.proposals[trace.proposals.length - 1];
  if (!proposal) {
    throw new Error('NoName proposal not found for manual apply');
  }
  const hint = `NoName option bias: next turn should prioritize actions around ${proposal.focus.trim()}`;
  if (currentDiagnostics.includes(hint)) {
    throw new Error('diagnostics already contains this NoName option bias hint');
  }
  plotState.last_generation_diagnostics = currentDiagnostics.trim()
    ? `${currentDiagnostics}; ${hint}`
    : hint;

  const nextOrder = Math.max(0, ...(trace.applyPlanLog ?? []).map((item) => item.order)) + 1;
  trace.applyPlanLog = [
    ...(trace.applyPlanLog ?? []),
    {
      order: nextOrder,
      target: 'optionBiasHint',
      decision: 'manual_apply',
      priority: 175,
      note: `manual apply confirmed for chapter=${chapterIndex} scope=optionBiasHint`,
    },
  ];
  trace.applyExecutionLog = [
    ...(trace.applyExecutionLog ?? []),
    {
      target: 'optionBiasHint',
      outcome: 'manual_option_bias_hint_applied',
      note: `manual option bias hint applied for focus=${proposal.focus}`,
    },
  ];
  trace.proposalTransitionLog = [
    ...(trace.proposalTransitionLog ?? []),
    `${requestId}:manual_apply:option_bias_hint`,
  ];
  trace.applyResult = {
    attempted: true,
    outcome: 'manual_option_bias_hint_applied',
    reason: 'manual option bias hint applied',
  };
  return { trace, plotState };
}

function resolveActionText(action: PlayerAction, options: PlayerOption[]): string {
  if (action.action_type === ActionType.FreeText) {
    return action.content.trim() || '你静静观察局势变化。';
  }
  const matched = options.find((opt) => opt.id === action.selected_option_id);
  return matched?.description ?? '你做出了一个谨慎选择。';
}

function updateAfterAction(action: PlayerAction, quickMode: boolean) {
  ensureStateReady();
  const gameState = runtimeState.gameState as GameState;
  const plotState = runtimeState.plotState as PlotState;
  const text = resolveActionText(action, plotState.current_scene.available_options);
  const targetSegment = pickWebNoNameTargetSegment(action);
  const applyScopes = deriveWebNoNameApplyScopes(targetSegment);
  const baseParagraph = quickMode
    ? `你迅速执行“${text}”，局势发生了可控变化。`
    : `你选择“${text}”，新的线索逐渐浮现。`;
  const paragraph = baseParagraph;

  plotState.current_chapter.content.push(paragraph);
  plotState.plot_history.push(paragraph);
  plotState.current_chapter.interaction_count += 1;
  plotState.segment_count += 1;
  plotState.last_action_result = createActionResult(paragraph);
  plotState.last_generation_diagnostics = quickMode ? '链路：quick_mode_rule_only（web_mock）' : '链路：web_mock';
  if (runtimeState.noNameMode === 'observeOnly') {
    plotState.last_generation_diagnostics += `；NoName.observeOnly：focus=${text}；target_segment=${targetSegment}；apply=skipped_observe_only`;
  }
  if (runtimeState.noNameMode === 'assisted') {
    const applyOutcome = 'applied_summary_and_option_bias';
    plotState.last_generation_diagnostics += `；NoName.assisted：focus=${text}；target_segment=${targetSegment}；scopes=${applyScopes.join(',')}；apply=${applyOutcome}`;
    if (applyScopes.includes('chapterSummaryHint')) {
      plotState.current_chapter.summary = applyWebNoNameSummaryHint(
        plotState.current_chapter.summary,
        text,
        targetSegment,
      );
    }
  }
  plotState.current_scene.available_options = buildInitialOptions().map((opt) => ({
    ...opt,
    id: opt.id,
    description: `${opt.description}（第${plotState.current_chapter.interaction_count + 1}轮）`,
  }));

  gameState.event_history.push({
    id: gameState.event_history.length + 1,
    timestamp: Date.now(),
    event_type: 'player_action',
    description: paragraph,
    importance: EventImportance.Normal,
  });
  gameState.game_time.total_days += 1;
  gameState.game_time.day += 1;
  gameState.player.stats.combat_power += 2;
  appendWebNoNameTrace(text, paragraph, targetSegment, applyScopes);
}

function listReachable(): string[] {
  ensureStateReady();
  const gameState = runtimeState.gameState as GameState;
  return gameState.script.world_setting.locations.map((loc) => loc.id);
}

function listOverview(): MapLocationOverview[] {
  ensureStateReady();
  const gameState = runtimeState.gameState as GameState;
  const current = gameState.script.world_setting.locations.find((loc) => loc.id === gameState.player.location);
  const currentEnergy = current?.spiritual_energy ?? 1;
  return gameState.script.world_setting.locations.map((loc) => ({
    location_id: loc.id,
    name: loc.name,
    spiritual_energy: loc.spiritual_energy,
    energy_gap: Number((loc.spiritual_energy - currentEnergy).toFixed(2)),
    reachable: true,
    risk_tier: loc.spiritual_energy > 1.25 ? 'high' : loc.spiritual_energy > 1 ? 'medium' : 'low',
    estimated_steps: 1,
    suggested_path: [gameState.player.location, loc.id],
  }));
}

function saveSlot(slotId: number) {
  ensureStateReady();
  runtimeState.saves[String(slotId)] = {
    script: runtimeState.script as Script,
    gameState: runtimeState.gameState as GameState,
    plotState: runtimeState.plotState as PlotState,
    timestamp: Date.now(),
    noNameMode: runtimeState.noNameMode,
  };
}

function loadSlot(slotId: number): GameState {
  const snapshot = runtimeState.saves[String(slotId)];
  if (!snapshot) {
    throw new Error(`槽位 ${slotId} 没有存档。`);
  }
  runtimeState.script = snapshot.script;
  runtimeState.gameState = snapshot.gameState;
  runtimeState.plotState = snapshot.plotState;
  runtimeState.noNameMode = snapshot.noNameMode ?? 'observeOnly';
  refreshWorldRegistry();
  return snapshot.gameState;
}

function listSlots(): SaveInfo[] {
  return Object.entries(runtimeState.saves)
    .map(([slotId, snapshot]) => ({
      slot_id: Number(slotId),
      version: 'web-mock-v1',
      timestamp: snapshot.timestamp,
      player_name: snapshot.gameState.player.name,
      player_age: snapshot.gameState.player.stats.lifespan.current_age,
      realm: snapshot.gameState.player.stats.cultivation_realm.name,
      location: snapshot.gameState.player.location,
      game_time: `第${snapshot.gameState.game_time.total_days}天`,
      noname_mode: snapshot.noNameMode,
    }))
    .sort((a, b) => b.timestamp - a.timestamp);
}

function buildNovel(title: string) {
  ensureStateReady();
  const plot = runtimeState.plotState as PlotState;
  const chapterContent = plot.current_chapter.content.join('\n\n');
  return {
    title,
    chapters: [
      {
        index: 1,
        title: plot.current_chapter.title,
        content: chapterContent,
        source_event_ids: (runtimeState.gameState as GameState).event_history.map((ev) => ev.id),
      },
    ],
    toc: [
      {
        index: 1,
        title: plot.current_chapter.title,
        summary: plot.current_chapter.summary || chapterContent.slice(0, 36),
        source_event_count: (runtimeState.gameState as GameState).event_history.length,
      },
    ],
    total_events: (runtimeState.gameState as GameState).event_history.length,
  };
}

function triggerTextDownload(filename: string, content: string) {
  if (typeof document === 'undefined') {
    return;
  }
  const blob = new Blob([content], { type: 'text/plain;charset=utf-8' });
  const href = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = href;
  anchor.download = filename;
  anchor.click();
  URL.revokeObjectURL(href);
}

export async function invokeWebRuntime<T>(
  command: string,
  args: Record<string, unknown> = {},
): Promise<T> {
  switch (command) {
    case 'load_script':
    case 'generate_random_script': {
      return buildDefaultScript() as T;
    }
    case 'initialize_game': {
      const script = (args.script as Script | undefined) ?? buildDefaultScript();
      runtimeState.script = script;
      runtimeState.gameState = buildGameState(script);
      refreshWorldRegistry();
      persistState();
      return runtimeState.gameState as T;
    }
    case 'initialize_plot': {
      if (!runtimeState.gameState) {
        throw new Error('Web 模式下尚未初始化世界状态，请先开始新游戏。');
      }
      runtimeState.plotState = buildPlotState(runtimeState.gameState as GameState);
      persistState();
      return runtimeState.plotState as T;
    }
    case 'execute_player_action': {
      updateAfterAction(args.action as PlayerAction, Boolean(args.quickMode));
      persistState();
      return 'ok' as T;
    }
    case 'get_game_state':
      ensureStateReady();
      return runtimeState.gameState as T;
    case 'get_plot_state':
      ensureStateReady();
      return runtimeState.plotState as T;
    case 'rehydrate_last_quick_mode_segment':
      return false as T;
    case 'get_player_options':
      ensureStateReady();
      return (runtimeState.plotState as PlotState).current_scene.available_options as T;
    case 'travel_to_location': {
      ensureStateReady();
      const locationId = String(args.locationId ?? '');
      const gameState = runtimeState.gameState as GameState;
      const loc = gameState.script.world_setting.locations.find((item) => item.id === locationId);
      if (!loc) {
        throw new Error(`目标地点不存在：${locationId}`);
      }
      gameState.player.location = locationId;
      const plotState = runtimeState.plotState as PlotState;
      plotState.current_scene.location = locationId;
      plotState.current_scene.name = loc.name;
      plotState.current_scene.description = loc.description;
      plotState.current_chapter.content.push(`你抵达了${loc.name}。`);
      plotState.last_generation_diagnostics = '链路：web_mock_travel';
      persistState();
      return 'ok' as T;
    }
    case 'get_reachable_locations':
      return listReachable() as T;
    case 'get_map_overview':
      return listOverview() as T;
    case 'save_game':
      saveSlot(Number(args.slotId ?? 0));
      persistState();
      return undefined as T;
    case 'load_game': {
      const loaded = loadSlot(Number(args.slotId ?? 0));
      persistState();
      return loaded as T;
    }
    case 'list_save_slots':
      return listSlots() as T;
    case 'get_world_registry':
      refreshWorldRegistry();
      persistState();
      return runtimeState.worldRegistry as T;
    case 'get_noname_recent_traces':
      return runtimeState.noNameTraces as T;
    case 'clear_noname_recent_traces':
      runtimeState.noNameTraces = [];
      persistState();
      return undefined as T;
    case 'mark_noname_controlled_output_review': {
      const trace = markWebNoNameControlledOutputReview(args as unknown as NoNameHumanReviewMarkPayload);
      persistState();
      return trace as T;
    }
    case 'resolve_noname_second_guardrail': {
      const trace = resolveWebNoNameSecondGuardrail(args as unknown as NoNameSecondGuardrailResolvePayload);
      persistState();
      return trace as T;
    }
    case 'apply_noname_manual_plot_text_hint': {
      const result = applyWebNoNameManualPlotTextHint(args);
      persistState();
      return result as T;
    }
    case 'apply_noname_reviewed_output': {
      const result = args.scope === 'chapterSummaryHint'
        ? applyWebNoNameManualChapterSummaryHint(args)
        : args.scope === 'optionBiasHint'
          ? applyWebNoNameManualOptionBiasHint(args)
          : args.scope === 'plotTextHint'
            ? applyWebNoNameManualPlotTextHint(args)
            : (() => {
              throw new Error(`Web mock reviewed apply currently does not support ${String(args.scope)}`);
            })();
      persistState();
      return result as T;
    }
    case 'get_noname_mode':
      return runtimeState.noNameMode as T;
    case 'set_noname_mode':
      runtimeState.noNameMode = (args.mode as WebRuntimeState['noNameMode']) ?? 'observeOnly';
      persistState();
      return runtimeState.noNameMode as T;
    case 'apply_world_registry_patch': {
      refreshWorldRegistry();
      if (runtimeState.worldRegistry && args.patch && typeof args.patch === 'object') {
        runtimeState.worldRegistry = {
          ...runtimeState.worldRegistry,
          tables: {
            ...runtimeState.worldRegistry.tables,
            ...(args.patch as { tables?: WorldRegistry['tables'] }).tables,
          },
        };
      }
      persistState();
      return runtimeState.worldRegistry as T;
    }
    case 'summarize_generation_diagnostics': {
      const diagnostics = Array.isArray(args.diagnostics) ? args.diagnostics : [];
      return {
        sampleCount: diagnostics.length,
        totalP50Ms: 1500,
        totalP95Ms: 2400,
        totalP99Ms: 3200,
        plotGenP95Ms: 1800,
        optionGenP95Ms: 900,
      } as T;
    }
    case 'summarize_generation_failures': {
      const diagnostics = Array.isArray(args.diagnostics) ? args.diagnostics : [];
      return {
        sampleCount: diagnostics.length,
        structuredOkCount: diagnostics.length,
        plainOkCount: 0,
        skeletonOkCount: 0,
        microOkCount: 0,
        presetFallbackCount: 0,
        turnUpdateFallbackCount: 0,
        optionLlmBlockedCount: 0,
        topReasons: [],
      } as T;
    }
    case 'update_plot_settings': {
      ensureStateReady();
      const plot = runtimeState.plotState as PlotState;
      plot.settings = { ...plot.settings, ...(args.settings as Partial<PlotSettings>) };
      persistState();
      return undefined as T;
    }
    case 'get_consistency_policy':
      return runtimeState.consistencyPolicy as T;
    case 'update_consistency_policy':
      runtimeState.consistencyPolicy = (args.policy as ConsistencyPolicy) ?? runtimeState.consistencyPolicy;
      persistState();
      return runtimeState.consistencyPolicy as T;
    case 'reset_consistency_policy':
      runtimeState.consistencyPolicy = defaultConsistencyPolicy();
      persistState();
      return runtimeState.consistencyPolicy as T;
    case 'get_llm_config_status':
      return { configured: false, provider: 'web-mock', model: 'web-mock-rule-engine' } as T;
    case 'set_llm_config':
    case 'clear_llm_config':
      return 'Web 模式下无需设置 LLM，使用内置规则引擎。' as T;
    case 'test_llm_connection':
      return 'Web 模式规则引擎在线。' as T;
    case 'parse_novel_characters':
      return ['韩立', '林墨', '叶青'] as T;
    case 'load_existing_novel':
      return buildDefaultScript() as T;
    case 'generate_novel': {
      const title = String(args.title ?? '修仙旅程记录');
      return buildNovel(title) as T;
    }
    case 'export_novel': {
      const novel = args.novel as {
        title: string;
        chapters: Array<{ title: string; content: string }>;
      };
      const content = [
        novel.title,
        '',
        ...novel.chapters.flatMap((chapter) => [chapter.title, chapter.content, '']),
      ].join('\n');
      const outputPath = String(args.outputPath ?? `${novel.title}.txt`);
      triggerTextDownload(outputPath, content);
      return undefined as T;
    }
    default:
      throw new Error(`Web 模式暂不支持命令：${command}`);
  }
}
