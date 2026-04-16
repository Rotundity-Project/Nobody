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
        applyScopes: ['diagnostics', 'chapterSummaryHint'],
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
      }],
      relatedObservations: [{
        role: 'worldCurator',
        actionSummary: '玩家返回山门',
        focus: '山门法阵',
        rationale: '需要补齐世界设定锚点',
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
    expect(text).toContain('应用计划：#1:chapter_summary_hint:apply@200');
    expect(text).toContain('应用执行：chapter_summary_hint:applied');
    expect(text).toContain('状态迁移：proposal-1:ready');
    expect(text).toContain('提案：Director提案：山门危机 / 山门危机 / ready');
    expect(text).toContain('目标段：current_turn_tail');
    expect(text).toContain('预期效果：为下一轮低风险输出提供导向');
    expect(text).toContain('作用域：diagnostics, chapterSummaryHint');
    expect(text).toContain('agent:delegation:running');
    expect(text).toContain('协作观察：worldCurator:山门法阵');
  });
});
