import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { useGameStore } from '../gameStore';
import { ActionType, Element, Grade, ScriptType, type PlotState, type Script } from '../../types/game';

const invokeMock = vi.fn();
const invokeWithTimeoutMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock('../../utils/tauriInvoke', () => ({
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

    invokeMock.mockImplementation((command: string) => {
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
      140000,
      expect.any(String),
    );
    expect(store.gameState).toEqual(gameState);
    expect(store.plotState).toEqual(plotState);
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

    invokeWithTimeoutMock.mockResolvedValue(script);
    invokeMock.mockImplementation((command: string) => {
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
});
