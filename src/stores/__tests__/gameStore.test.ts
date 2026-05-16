import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { useGameStore } from '../gameStore';
import { ActionType, Element, Grade, ScriptType, type PlotState, type Script } from '../../types/game';

const invokeMock = vi.fn();
const invokeWithTimeoutMock = vi.fn();

vi.mock('../../utils/tauriInvoke', () => ({
  invokeRuntime: (...args: unknown[]) => invokeMock(...args),
  invokeWithTimeout: (...args: unknown[]) => invokeWithTimeoutMock(...args),
}));

const baseScript = (): Script => ({
  id: 'script_1',
  name: 'Test Script',
  script_type: ScriptType.Custom,
  world_setting: {
    cultivation_realms: [],
    spiritual_roots: [],
    techniques: [],
    locations: [],
    factions: [],
  },
  initial_state: {
    player_name: '',
    player_spiritual_root: {
      element: Element.Fire,
      grade: Grade.Double,
      affinity: 0.5,
    },
    starting_location: 'sect',
    starting_age: 16,
  },
});

const basePlotState = (): PlotState => ({
  current_scene: {
    id: 'scene_1',
    name: 'Test Scene',
    description: 'Scene',
    location: 'sect',
    available_options: [],
  },
  plot_history: [],
  is_waiting_for_input: true,
  last_action_result: null,
  settings: {
    recap_enabled: true,
    novel_style: 'xianxia-third-person',
    llm_priority_mode: true,
    llm_strict_mode: false,
    min_interactions_per_chapter: 2,
    max_interactions_per_chapter: 3,
    target_chapter_words_min: 5000,
    target_chapter_words_max: 7000,
  },
  current_chapter: {
    index: 1,
    title: '第一章',
    content: [],
    summary: '',
    interaction_count: 0,
  },
  chapters: [],
  segment_count: 0,
});

const baseGameState = (script: Script) => ({
  script,
  player: {
    id: 'player_1',
    name: script.initial_state.player_name,
    stats: {
      spiritual_root: script.initial_state.player_spiritual_root,
      cultivation_realm: {
        name: '练气',
        level: 1,
        sub_level: 0,
        power_multiplier: 1,
      },
      techniques: [],
      lifespan: {
        current_age: 16,
        max_age: 100,
        realm_bonus: 0,
      },
      combat_power: 10,
    },
    inventory: [],
    location: script.initial_state.starting_location,
  },
  world_state: {
    locations: {},
    factions: {},
    global_events: [],
  },
  game_time: {
    year: 1,
    month: 1,
    day: 1,
    total_days: 1,
  },
  event_history: [],
});

describe('gameStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    invokeMock.mockReset();
    invokeWithTimeoutMock.mockReset();
  });

  it('initializes game and plot state', async () => {
    const script = baseScript();
    const gameState = baseGameState(script);
    const plotState = basePlotState();

    invokeWithTimeoutMock.mockImplementation((command: string) => {
      if (command === 'initialize_game') {
        return Promise.resolve(gameState);
      }
      if (command === 'initialize_plot') {
        return Promise.resolve(plotState);
      }
      return Promise.resolve(null);
    });

    const store = useGameStore();
    await store.initializeGame(script, '  Lin Mo  ');

    expect(store.currentScript?.initial_state.player_name).toBe('Lin Mo');
    expect(store.gameState).toEqual(gameState);
    expect(store.plotState).toEqual(plotState);
    expect(store.isLoading).toBe(false);
    expect(store.error).toBeNull();
  });

  it('executes player action and refreshes state', async () => {
    const script = baseScript();
    const gameState = baseGameState(script);
    const plotState = basePlotState();
    plotState.last_generation_diagnostics = '回退：测试诊断';

    invokeWithTimeoutMock.mockImplementation((command: string) => {
      if (command === 'execute_player_action') {
        return Promise.resolve('ok');
      }
      if (command === 'get_game_state') {
        return Promise.resolve(gameState);
      }
      if (command === 'get_plot_state') {
        return Promise.resolve(plotState);
      }
      return Promise.resolve(null);
    });

    const store = useGameStore();
    await store.executePlayerAction({
      action_type: ActionType.FreeText,
      content: 'test',
      selected_option_id: null,
    });

    expect(invokeWithTimeoutMock).toHaveBeenCalledWith(
      'execute_player_action',
      { action: expect.any(Object) },
      60000,
      expect.any(String),
    );
    expect(store.gameState).toEqual(gameState);
    expect(store.plotState).toEqual(plotState);
    expect(store.error).toBeNull();
    expect(store.isLoading).toBe(false);
  });

  it('loads game and updates plot', async () => {
    const script = baseScript();
    const gameState = baseGameState(script);
    const plotState = basePlotState();

    invokeMock.mockImplementation((command: string) => {
      if (command === 'load_game') {
        return Promise.resolve(gameState);
      }
      if (command === 'get_plot_state') {
        return Promise.resolve(plotState);
      }
      return Promise.resolve(null);
    });

    const store = useGameStore();
    await store.loadGame(1);

    expect(store.gameState).toEqual(gameState);
    expect(store.plotState).toEqual(plotState);
    expect(store.isLoading).toBe(false);
  });

  it('initializes random game with timeout helper', async () => {
    const script = baseScript();
    const gameState = baseGameState(script);
    const plotState = basePlotState();

    invokeWithTimeoutMock.mockImplementation((command: string) => {
      if (command === 'generate_random_script') {
        return Promise.resolve(script);
      }
      if (command === 'initialize_game') {
        return Promise.resolve(gameState);
      }
      if (command === 'initialize_plot') {
        return Promise.resolve(plotState);
      }
      return Promise.resolve(null);
    });

    const store = useGameStore();
    await store.initializeRandomGame();

    expect(invokeWithTimeoutMock).toHaveBeenCalledWith(
      'generate_random_script',
      undefined,
      120000,
      expect.any(String),
    );
    expect(store.currentScript).toEqual(script);
    expect(store.gameState).toEqual(gameState);
    expect(store.plotState).toEqual(plotState);
  });

  it('travels to location and refreshes game/plot state', async () => {
    const script = baseScript();
    const gameState = baseGameState(script);
    const plotState = basePlotState();
    plotState.current_scene.location = 'valley';
    gameState.player.location = 'valley';

    invokeMock.mockImplementation((command: string) => {
      if (command === 'travel_to_location') {
        return Promise.resolve('ok');
      }
      if (command === 'get_game_state') {
        return Promise.resolve(gameState);
      }
      if (command === 'get_plot_state') {
        return Promise.resolve(plotState);
      }
      return Promise.resolve(null);
    });

    const store = useGameStore();
    await store.travelToLocation('valley');

    expect(invokeMock).toHaveBeenCalledWith('travel_to_location', { locationId: 'valley' });
    expect(store.gameState?.player.location).toBe('valley');
    expect(store.plotState?.current_scene.location).toBe('valley');
  });

  it('refreshes world registry snapshot', async () => {
    invokeMock.mockImplementation((command: string) => {
      if (command === 'get_world_registry') {
        return Promise.resolve({
          session_id: 's1',
          seed: 1,
          created_at: 1,
          llm_model: 'm',
          source: 'llm_bootstrap',
          tables: {
            characters: [{ name: 'LinMo' }],
            map_nodes: [],
            map_edges: [],
            techniques: [],
            inventory_items: [],
            factions: [],
            story_state: [],
            world_facts: [],
          },
        });
      }
      return Promise.resolve(null);
    });

    const store = useGameStore();
    await store.refreshWorldRegistry();
    expect(store.worldRegistry?.session_id).toBe('s1');
    expect(store.worldRegistry?.tables.characters.length).toBe(1);
  });

  it('applies world registry patch and refreshes game state', async () => {
    const script = baseScript();
    const gameState = baseGameState(script);
    invokeMock.mockImplementation((command: string) => {
      if (command === 'apply_world_registry_patch') {
        return Promise.resolve({
          session_id: 's2',
          seed: 2,
          created_at: 2,
          llm_model: null,
          source: 'manual_patch',
          tables: {
            characters: [],
            map_nodes: [],
            map_edges: [],
            techniques: [],
            inventory_items: [],
            factions: [],
            story_state: [],
            world_facts: [{ fact_id: 'f1' }],
          },
        });
      }
      if (command === 'get_game_state') {
        return Promise.resolve(gameState);
      }
      return Promise.resolve(null);
    });
    const store = useGameStore();
    await store.applyWorldRegistryPatch({ world_facts: [{ fact_id: 'f1' }] });
    expect(store.worldRegistry?.source).toBe('manual_patch');
    expect(store.gameState?.player.id).toBe('player_1');
  });

  it('formats noname trace debug text with apply preflight details', () => {
    const store = useGameStore();
    store.noNameTraces = [{
      traceId: 'trace-1',
      sessionId: 'session-1',
      turnId: 'turn-1',
      mode: 'assisted',
      graphPath: ['CollectTurnInput', 'ApplyProposal'],
      capabilityCalls: [],
      proposals: [{
        proposalId: 'proposal-1',
        kind: 'plotCandidate',
        producerRole: 'director',
        title: 'Director提案：山门危机',
        summary: '建议优先观察山门危机',
        focus: '山门危机',
        targetSegment: 'current_turn_tail',
        intendedEffect: '为下一轮低风险输出提供导向',
        rationale: '当前章节冲突正在汇聚',
        labels: ['director', 'assisted_ready'],
        applyScopes: ['diagnostics', 'chapterSummaryHint', 'plotAugmentationHint'],
        status: 'ready',
        applyable: true,
      }],
      proposalTransitionLog: ['proposal-1:ready'],
      applyPlanLog: [{
        order: 1,
        target: 'chapter_summary_hint',
        decision: 'apply',
        priority: 200,
        note: '允许执行 chapter_summary_hint',
      }],
      applyExecutionLog: [{
        target: 'chapter_summary_hint',
        outcome: 'applied',
        note: '已写入章节摘要提示',
      }, {
        target: 'plot_augmentation_hint',
        outcome: 'pending_plot_augmentation_consumed',
        note: 'pending plot augmentation consumed after plot_engine generation; count=1',
      }],
      controlledOutputReviews: [{
        requestId: 'controlled-output-proposal-1-plot_text_hint',
        proposalId: 'proposal-1',
        requestedKind: 'sceneAugmentation',
        decision: 'needsReview',
        reason: 'plot text hint requires human review before higher-layer apply',
        normalizedKind: 'sceneAugmentation',
        safeApplyScope: 'plotTextHint',
        policyForbiddenScopes: ['finalPlotState', 'canonWorldFact'],
        requiresHumanReview: true,
        humanReviewDecision: 'approvedForHigherApply',
      }],
      relatedObservations: [{
        role: 'worldCurator',
        actionSummary: '玩家返回山门',
        focus: '山门法阵',
        rationale: '需要补齐世界设定锚点',
        roleGoal: 'Maintain world facts, scene constraints, and canon anchors.',
        sceneFocus: '山门法阵',
        forbiddenScopes: ['Must not decide NPC private intent.'],
        noteTypeHits: ['goal: Hold Gate'],
        sourceStats: [{ source: 'hard_facts', count: 2 }],
        contextTokenBudgetUsed: 42,
        contextSliceStats: [{ section: 'worldFacts', sourceCount: 4, visibleCount: 3 }],
        proposal: {
          proposalId: 'proposal-world-1',
          kind: 'worldPatchProposal',
          producerRole: 'worldCurator',
          title: 'WorldCurator提案：山门法阵',
          summary: '补齐法阵约束',
          focus: '山门法阵',
          targetSegment: 'chapter_summary_tail',
          intendedEffect: '补足世界事实',
          rationale: '设定缺口明显',
          labels: ['worldCurator'],
          applyScopes: ['diagnostics', 'chapterSummaryHint'],
          status: 'observed',
          applyable: false,
        },
      }],
      protocolEvents: [{
        channel: 'agent',
        from: 'director',
        to: 'world_curator',
        kind: 'delegation',
        taskId: 'turn-1-world_curator-observe',
        status: 'running',
        detail: 'turn-1-director-observe',
      }],
      guardrailResult: {
        outcome: 'accept',
        reason: null,
      },
      applyResult: {
        attempted: true,
        outcome: 'preflight_ready',
        reason: '已通过 assisted apply 预检，尚未修改主剧情结果',
      },
      fallbackUsed: false,
      elapsedMs: 12,
    }];

    const text = store.getNoNameTraceDebugText();
    expect(text).toContain('预检：preflight_ready');
    expect(text).toContain('应用生命周期：提案阶段:ready');
    expect(text).toContain('应用阶段核对：1.Human Review=approved');
    expect(text).toContain('安全输出草稿：reviewed:plotTextHint:sceneAugmentation[final=false]');
    expect(text).toContain('安全输出证据：reviewed:plotTextHint[missing=secondGuardrailDecision][warnings=none]');
    expect(text).toContain('应用计划：#1:chapter_summary_hint:apply@200');
    expect(text).toContain('应用执行：chapter_summary_hint:applied');
    expect(text).toContain('剧情增强提示:已消费[raw=plot_augmentation_hint:pending_plot_augmentation_consumed]');
    expect(text).toContain('受控输出复核：sceneAugmentation:plotTextHint:needsReview:human[禁区=finalPlotState/canonWorldFact][人工=approvedForHigherApply]');
    expect(text).toContain('状态迁移：proposal-1:ready');
    expect(text).toContain('提案：Director提案：山门危机 / 山门危机 / ready');
    expect(text).toContain('目标段：current_turn_tail');
    expect(text).toContain('预期效果：为下一轮低风险输出提供导向');
    expect(text).toContain('作用域：diagnostics, chapterSummaryHint');
    expect(text).toContain('剧情增强提示：已消费');
    expect(text).toContain('agent:delegation:running');
    expect(text).toContain('协议摘要：roles=2(director/world_curator), tasks=1, events=1, statuses=running:1, errors=no');
    expect(text).toContain('[来源=hard_facts:2]');
    expect(text).toContain('[token=42]');
    expect(text).toContain('[裁剪=worldFacts:4->3]');
    expect(text).toContain('协作观察：worldCurator:山门法阵');
    expect(text).toContain('[上下文=Maintain world facts, scene constraints, and canon anchors. / 山门法阵]');
    expect(text).toContain('[禁区=Must not decide NPC private intent.]');
    expect(text).toContain('[笔记=goal: Hold Gate]');
  });

  it('marks a NoName controlled output review and updates local trace', async () => {
    const store = useGameStore();
    store.noNameTraces = [{
      traceId: 'trace-1',
      sessionId: 'session-1',
      turnId: 'turn-1',
      mode: 'assisted',
      graphPath: [],
      capabilityCalls: [],
      proposals: [],
      fallbackUsed: false,
      elapsedMs: 0,
      controlledOutputReviews: [{
        requestId: 'review-1',
        requestedKind: 'sceneAugmentation',
        decision: 'needsReview',
        reason: 'requires review',
        safeApplyScope: 'plotTextHint',
        requiresHumanReview: true,
      }],
    }];
    invokeMock.mockResolvedValue({
      ...store.noNameTraces[0],
      controlledOutputReviews: [{
        requestId: 'review-1',
        requestedKind: 'sceneAugmentation',
        decision: 'needsReview',
        reason: 'requires review',
        safeApplyScope: 'plotTextHint',
        requiresHumanReview: true,
        humanReviewDecision: 'rejectedForHigherApply',
      }],
    });

    await store.markNoNameControlledOutputReview({
      traceId: 'trace-1',
      requestId: 'review-1',
      decision: 'rejectedForHigherApply',
    });

    expect(invokeMock).toHaveBeenCalledWith('mark_noname_controlled_output_review', {
      traceId: 'trace-1',
      requestId: 'review-1',
      decision: 'rejectedForHigherApply',
    });
    expect(store.noNameTraces[0].controlledOutputReviews?.[0].humanReviewDecision)
      .toBe('rejectedForHigherApply');
  });

  it('resolves a NoName second guardrail decision and updates local trace', async () => {
    const store = useGameStore();
    store.noNameTraces = [{
      traceId: 'trace-1',
      sessionId: 'session-1',
      turnId: 'turn-1',
      mode: 'assisted',
      graphPath: [],
      capabilityCalls: [],
      proposals: [],
      fallbackUsed: false,
      elapsedMs: 0,
      applyPlanLog: [],
      applyExecutionLog: [],
    }];
    invokeMock.mockResolvedValue({
      ...store.noNameTraces[0],
      applyPlanLog: [{
        order: 1,
        target: 'plot_text_hint',
        decision: 'second_guardrail_allow',
        priority: 350,
        note: 'allowed',
      }],
    });

    await store.resolveNoNameSecondGuardrail({
      traceId: 'trace-1',
      requestId: 'review-1',
      decision: 'allow',
    });

    expect(invokeMock).toHaveBeenCalledWith('resolve_noname_second_guardrail', {
      traceId: 'trace-1',
      requestId: 'review-1',
      decision: 'allow',
    });
    expect(store.noNameTraces[0].applyPlanLog?.[0]?.decision).toBe('second_guardrail_allow');
  });

  it('applies a manual NoName plot text hint with current segment snapshot', async () => {
    const store = useGameStore();
    const plotState = basePlotState();
    plotState.current_chapter.index = 1;
    plotState.current_chapter.content = ['你看见山门风声渐紧。'];
    plotState.plot_history = ['你看见山门风声渐紧。'];
    store.plotState = plotState;
    store.noNameTraces = [{
      traceId: 'trace-1',
      sessionId: 'session-1',
      turnId: 'turn-1',
      mode: 'assisted',
      graphPath: [],
      capabilityCalls: [],
      proposals: [],
      fallbackUsed: false,
      elapsedMs: 0,
    }];
    invokeMock.mockResolvedValue({
      trace: {
        ...store.noNameTraces[0],
        applyExecutionLog: [{
          target: 'plot_text_hint',
          outcome: 'manual_plot_text_applied',
          note: 'applied',
        }],
      },
      plotState: {
        ...plotState,
        current_chapter: {
          ...plotState.current_chapter,
          content: ['你看见山门风声渐紧。\n\n【NoName】重点关注：山门危机'],
        },
      },
    });

    await store.applyNoNameManualPlotTextHint({
      traceId: 'trace-1',
      requestId: 'review-1',
    });

    expect(invokeMock).toHaveBeenCalledWith('apply_noname_reviewed_output', {
      traceId: 'trace-1',
      requestId: 'review-1',
      scope: 'plotTextHint',
      chapterIndex: 1,
      segmentIndex: 0,
      expectedSegmentText: '你看见山门风声渐紧。',
    });
    expect(store.plotState?.current_chapter.content[0]).toContain('【NoName】重点关注');
    expect(store.noNameTraces[0].applyExecutionLog?.[0]?.outcome)
      .toBe('manual_plot_text_applied');
  });

  it('applies a manual NoName chapter summary hint with current summary snapshot', async () => {
    const store = useGameStore();
    const plotState = basePlotState();
    plotState.current_chapter.index = 2;
    plotState.current_chapter.summary = '已有摘要';
    store.plotState = plotState;
    store.noNameTraces = [{
      traceId: 'trace-summary',
      sessionId: 'session-1',
      turnId: 'turn-1',
      mode: 'assisted',
      graphPath: [],
      capabilityCalls: [],
      proposals: [],
      fallbackUsed: false,
      elapsedMs: 0,
    }];
    invokeMock.mockResolvedValue({
      trace: {
        ...store.noNameTraces[0],
        applyExecutionLog: [{
          target: 'chapter_summary_hint',
          outcome: 'manual_chapter_summary_hint_applied',
          note: 'applied',
        }],
      },
      plotState: {
        ...plotState,
        current_chapter: {
          ...plotState.current_chapter,
          summary: '已有摘要; NoName summary hint: 山门危机',
        },
      },
    });

    await store.applyNoNameManualChapterSummaryHint({
      traceId: 'trace-summary',
      requestId: 'review-summary',
    });

    expect(invokeMock).toHaveBeenCalledWith('apply_noname_reviewed_output', {
      traceId: 'trace-summary',
      requestId: 'review-summary',
      scope: 'chapterSummaryHint',
      chapterIndex: 2,
      expectedSummary: '已有摘要',
    });
    expect(store.plotState?.current_chapter.summary).toContain('NoName summary hint');
    expect(store.noNameTraces[0].applyExecutionLog?.[0]?.outcome)
      .toBe('manual_chapter_summary_hint_applied');
  });

  it('applies a manual NoName option bias hint with current diagnostics snapshot', async () => {
    const store = useGameStore();
    const plotState = basePlotState();
    plotState.current_chapter.index = 3;
    plotState.last_generation_diagnostics = 'base diagnostics';
    store.plotState = plotState;
    store.noNameTraces = [{
      traceId: 'trace-option',
      sessionId: 'session-1',
      turnId: 'turn-1',
      mode: 'assisted',
      graphPath: [],
      capabilityCalls: [],
      proposals: [],
      fallbackUsed: false,
      elapsedMs: 0,
    }];
    invokeMock.mockResolvedValue({
      trace: {
        ...store.noNameTraces[0],
        applyExecutionLog: [{
          target: 'option_bias_hint',
          outcome: 'manual_option_bias_hint_applied',
          note: 'applied',
        }],
      },
      plotState: {
        ...plotState,
        last_generation_diagnostics: 'base diagnostics; NoName option bias: focus',
      },
    });

    await store.applyNoNameManualOptionBiasHint({
      traceId: 'trace-option',
      requestId: 'review-option',
    });

    expect(invokeMock).toHaveBeenCalledWith('apply_noname_reviewed_output', {
      traceId: 'trace-option',
      requestId: 'review-option',
      scope: 'optionBiasHint',
      chapterIndex: 3,
      expectedGenerationDiagnostics: 'base diagnostics',
    });
    expect(store.plotState?.last_generation_diagnostics).toContain('NoName option bias');
    expect(store.noNameTraces[0].applyExecutionLog?.[0]?.outcome)
      .toBe('manual_option_bias_hint_applied');
  });

  it('shows a friendly stale snapshot error for manual NoName plot text apply', async () => {
    const store = useGameStore();
    const plotState = basePlotState();
    plotState.current_chapter.index = 1;
    plotState.current_chapter.content = ['你看见山门风声渐紧。'];
    store.plotState = plotState;
    invokeMock.mockRejectedValue(new Error('segment snapshot mismatch; refusing stale manual apply'));

    await expect(store.applyNoNameManualPlotTextHint({
      traceId: 'trace-1',
      requestId: 'review-1',
    })).rejects.toThrow('当前剧情段落已变化');

    expect(store.error).toBe('NoName 人工写入已取消：当前剧情段落已变化，请重新打开调试台确认差异预览。');
  });
});
