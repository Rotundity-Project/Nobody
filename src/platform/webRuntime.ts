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
  type PlayerAction,
  type PlayerOption,
  type PlotSettings,
  type PlotState,
  type SaveInfo,
  type Script,
  type WorldRegistry,
} from '../types/game';

type SaveSnapshot = {
  script: Script;
  gameState: GameState;
  plotState: PlotState;
  timestamp: number;
};

type WebRuntimeState = {
  script: Script | null;
  gameState: GameState | null;
  plotState: PlotState | null;
  saves: Record<string, SaveSnapshot>;
  worldRegistry: WorldRegistry | null;
  consistencyPolicy: ConsistencyPolicy;
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
  const paragraph = quickMode
    ? `你迅速执行“${text}”，局势发生了可控变化。`
    : `你选择“${text}”，新的线索逐渐浮现。`;

  plotState.current_chapter.content.push(paragraph);
  plotState.plot_history.push(paragraph);
  plotState.current_chapter.interaction_count += 1;
  plotState.segment_count += 1;
  plotState.last_action_result = createActionResult(paragraph);
  plotState.last_generation_diagnostics = quickMode ? '链路：quick_mode_rule_only（web_mock）' : '链路：web_mock';
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
