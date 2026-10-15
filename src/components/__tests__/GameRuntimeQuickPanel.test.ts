import { mount } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import { ref } from 'vue';
import GameRuntimeView from '../GameRuntimeView.vue';
import RuntimeQuickPanelsDialog from '../RuntimeQuickPanelsDialog.vue';

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
    loadingProgress: ref(null),
    loadingProgressText: ref(''),
    loadingElapsedMs: ref(null),
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
  it('opens the matching panel from every bottom action dock button', async () => {
    const wrapper = mount(GameRuntimeView, {
      global: {
        stubs: {
          InkQuickActionDock: {
            template: `
              <div>
                <button data-testid="open-character" @click="$emit('open-character')">open-character</button>
                <button data-testid="open-info" @click="$emit('open-info')">open-info</button>
                <button data-testid="open-backpack" @click="$emit('open-backpack')">open-backpack</button>
                <button data-testid="open-techniques" @click="$emit('open-techniques')">open-techniques</button>
                <button data-testid="open-factions" @click="$emit('open-factions')">open-factions</button>
                <button data-testid="open-world" @click="$emit('open-world')">open-world</button>
                <button data-testid="open-save" @click="$emit('open-save')">open-save</button>
                <button data-testid="open-load" @click="$emit('open-load')">open-load</button>
              </div>
            `,
          },
          GameSystemDialogs: {
            props: ['showSaveDialog', 'showLoadDialog'],
            template: `
              <div
                data-testid="system-dialogs"
                :data-save-open="String(showSaveDialog)"
                :data-load-open="String(showLoadDialog)"
              />
            `,
          },
          GameInfoCenterDialog: {
            props: ['isOpen'],
            template: '<div data-testid="info-dialog" :data-open="String(isOpen)" />',
          },
          CharacterInfoModal: {
            props: ['isOpen', 'character'],
            template: '<div data-testid="character-dialog" :data-open="String(isOpen)">{{ character?.name }}</div>',
          },
          RuntimeQuickPanelsDialog: {
            name: 'RuntimeQuickPanelsDialog',
            props: ['isOpen', 'activeTab', 'panels'],
            template: `
              <div data-testid="quick-dialog" :data-open="String(isOpen)" :data-tab="activeTab">
                {{ JSON.stringify((panels.find((item) => item.id === activeTab) || {}).items || []) }}
              </div>
            `,
          },
          StoryViewport: true,
          NotificationCenter: true,
        },
      },
    });

    expect(wrapper.get('[data-testid="quick-dialog"]').attributes('data-open')).toBe('false');

    await wrapper.get('[data-testid="open-character"]').trigger('click');
    expect(wrapper.get('[data-testid="character-dialog"]').attributes('data-open')).toBe('true');
    expect(wrapper.get('[data-testid="character-dialog"]').text()).toContain('无名弟子');

    await wrapper.get('[data-testid="open-info"]').trigger('click');
    expect(wrapper.get('[data-testid="info-dialog"]').attributes('data-open')).toBe('true');

    await wrapper.get('[data-testid="open-backpack"]').trigger('click');
    let quickDialog = wrapper.get('[data-testid="quick-dialog"]');
    expect(quickDialog.attributes('data-open')).toBe('true');
    expect(quickDialog.attributes('data-tab')).toBe('backpack');
    expect(quickDialog.text()).toContain('下品灵石');

    await wrapper.get('[data-testid="open-techniques"]').trigger('click');
    quickDialog = wrapper.get('[data-testid="quick-dialog"]');
    expect(quickDialog.attributes('data-open')).toBe('true');
    expect(quickDialog.attributes('data-tab')).toBe('techniques');
    expect(quickDialog.text()).toContain('引气诀');

    await wrapper.get('[data-testid="open-factions"]').trigger('click');
    quickDialog = wrapper.get('[data-testid="quick-dialog"]');
    expect(quickDialog.attributes('data-open')).toBe('true');
    expect(quickDialog.attributes('data-tab')).toBe('factions');
    expect(quickDialog.text()).toContain('青岚宗');

    await wrapper.get('[data-testid="open-world"]').trigger('click');

    quickDialog = wrapper.get('[data-testid="quick-dialog"]');
    expect(quickDialog.attributes('data-open')).toBe('true');
    expect(quickDialog.attributes('data-tab')).toBe('world');
    expect(quickDialog.text()).toContain('山门');
    expect(quickDialog.text()).toContain('戒备森严');

    await wrapper.get('[data-testid="open-save"]').trigger('click');
    expect(wrapper.get('[data-testid="system-dialogs"]').attributes('data-save-open')).toBe('true');

    await wrapper.get('[data-testid="open-load"]').trigger('click');
    expect(wrapper.get('[data-testid="system-dialogs"]').attributes('data-load-open')).toBe('true');
  });

  it('does not apply shell stacking styles to overlay component roots', () => {
    const wrapper = mount(GameRuntimeView, {
      global: {
        stubs: {
          GameRuntimeTopBar: true,
          GameRuntimeLeftStatusPanels: true,
          GameRuntimeWorldRegistryPanel: true,
          GameRuntimeMainHeader: true,
          StoryViewport: true,
          GameRuntimeInteractionCard: true,
          GameRuntimeBottomBar: true,
          GameSystemDialogs: true,
          GameInfoCenterDialog: true,
          CharacterInfoModal: true,
          RuntimeQuickPanelsDialog,
          NoNameDebugConsole: true,
          NotificationCenter: true,
        },
      },
    });

    const shellContent = wrapper.get('.runtime-shell-content').element;
    expect(shellContent.parentElement).toBe(wrapper.get('.game-shell').element);
    expect(wrapper.find('.game-shell > .runtime-shell-content').exists()).toBe(true);

    const quickDialog = wrapper.getComponent(RuntimeQuickPanelsDialog);
    quickDialog.vm.$emit('update:active-tab', 'world');
    quickDialog.vm.$emit('close');

    expect(wrapper.find('.runtime-quick-overlay').exists()).toBe(false);
  });
});
