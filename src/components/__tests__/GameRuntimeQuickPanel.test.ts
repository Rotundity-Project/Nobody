import { mount } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import { ref } from 'vue';
import GameRuntimeView from '../GameRuntimeView.vue';

vi.mock('vue-router', () => ({
  useRouter: () => ({
    push: vi.fn(),
  }),
}));

vi.mock('../../utils/audioSystem', () => ({
  playClick: vi.fn(),
}));

const mockStore = {
  playerCharacter: {
    name: '无名弟子',
    location: 'sect_valley',
    stats: {
      spiritual_root: { element: 'Fire', elements: ['Fire'], grade: 'Double', affinity: 0.6 },
      cultivation_realm: { name: '炼气', level: 1, sub_level: 2 },
      techniques: [],
      combat_power: 88,
      lifespan: { current_age: 18, max_age: 100, realm_bonus: 0 },
    },
    inventory: [],
  },
  currentScene: {
    name: '山门晨雾',
    description: '清晨灵雾缭绕，钟声回荡。',
    location: 'sect_valley',
  },
  plotState: {
    current_chapter: {
      index: 1,
      title: '初入仙门',
      content: ['第一段', '第二段'],
      interaction_count: 1,
    },
    chapters: [],
    settings: {
      min_interactions_per_chapter: 2,
      max_interactions_per_chapter: 4,
    },
    interaction_state: 'waiting_for_choice',
    last_option_generation_source: 'llm_structured',
    last_generation_diagnostics: '',
    last_consistency_risk_score: 2,
  },
  gameState: {
    game_time: { year: 1, month: 1, day: 1, total_days: 1 },
    script: {
      world_setting: {
        locations: [{ id: 'sect_valley', name: '山门', spiritual_energy: 0.7 }],
        techniques: [],
        factions: [],
      },
    },
    world_state: {
      factions: {},
    },
    event_history: [],
  },
  worldRegistry: {
    session_id: 'test-session',
    seed: 1,
    created_at: 1,
    source: 'test',
    tables: {
      characters: [{ character_id: 'player', name: '无名弟子', location_id: 'sect_valley' }],
      map_nodes: [{ location_id: 'sect_valley', name: '山门', description: '仙门入口' }],
      map_edges: [],
      techniques: [{ technique_id: 'tech_1', name: '引气诀', description: '基础吐纳术', required_realm_level: 1 }],
      inventory_items: [{ item_id: 'stone_1', name: '下品灵石', quantity: 12, owner_character_id: 'player' }],
      factions: [{ faction_id: 'sect_1', name: '青岚宗', description: '正道门派', power_level: 3 }],
      story_state: [{ chapter_index: 1, chapter_goal: '入门试炼' }],
      world_facts: [{ fact_id: 'fact_1', subject: '山门', predicate: '现状', object: '戒备森严' }],
    },
  },
  reachableLocationIds: ['sect_valley'],
  mapOverview: [],
  currentScript: null,
  isGameInitialized: true,
  isWaitingForInput: true,
  isPlotInitialized: false,
  availableOptions: [{ id: 1, description: '观察四周', requirements: [], action: {} }],
  error: null,
  lastInitializationDurationMs: null,
  executePlayerAction: vi.fn().mockResolvedValue(undefined),
  clearError: vi.fn(),
  travelToLocation: vi.fn().mockResolvedValue(undefined),
  refreshWorldRegistry: vi.fn().mockResolvedValue(undefined),
  applyWorldRegistryPatch: vi.fn().mockResolvedValue(undefined),
};

vi.mock('../../stores/gameStore', () => ({
  useGameStore: () => mockStore,
}));

vi.mock('../../composables/useInputMode', () => ({
  useInputMode: () => ({
    inputMode: ref('options'),
    freeTextInput: ref(''),
    inputValidation: ref({ valid: true, message: '' }),
    setInputMode: vi.fn(),
  }),
}));

vi.mock('../../composables/useStoryFlow', () => ({
  useStoryFlow: () => ({
    isLoading: ref(false),
    loadingMessage: ref('处理中...'),
    autoAdvanceRunning: ref(false),
    autoAdvanceStopHint: ref(''),
    handleOptionSelect: vi.fn(),
    handleFreeTextSubmit: vi.fn(),
    handleContinue: vi.fn(),
    requestStopAutoAdvance: vi.fn(),
  }),
}));

vi.mock('../../composables/useGameHotkeys', () => ({
  useGameHotkeys: vi.fn(),
}));

describe('GameRuntimeView quick panel', () => {
  it('opens world quick panel from bottom action dock', async () => {
    const wrapper = mount(GameRuntimeView, {
      global: {
        stubs: {
          InkQuickActionDock: {
            template: `
              <div>
                <button data-testid="open-world" @click="$emit('open-world')">open-world</button>
              </div>
            `,
          },
          RuntimeQuickPanelsDialog: {
            name: 'RuntimeQuickPanelsDialog',
            props: ['isOpen', 'activeTab', 'panels'],
            template: `
              <div data-testid="quick-dialog" :data-open="String(isOpen)" :data-tab="activeTab">
                {{ JSON.stringify((panels.find((item) => item.id === 'world') || {}).items || []) }}
              </div>
            `,
          },
          StoryViewport: true,
          GameInteractionPanel: true,
          GameSystemDialogs: true,
          GameInfoCenterDialog: true,
          CharacterInfoModal: true,
          NotificationCenter: true,
        },
      },
    });

    expect(wrapper.get('[data-testid="quick-dialog"]').attributes('data-open')).toBe('false');

    await wrapper.get('[data-testid="open-world"]').trigger('click');

    const quickDialog = wrapper.get('[data-testid="quick-dialog"]');
    expect(quickDialog.attributes('data-open')).toBe('true');
    expect(quickDialog.attributes('data-tab')).toBe('world');
    expect(quickDialog.text()).toContain('山门');
    expect(quickDialog.text()).toContain('戒备森严');
  });
});
