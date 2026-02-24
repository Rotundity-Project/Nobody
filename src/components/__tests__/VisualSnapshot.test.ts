import { shallowMount } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import MainMenu from '../MainMenu.vue';
import ScriptSelector from '../ScriptSelector.vue';
import GameRuntimeView from '../GameRuntimeView.vue';
import InfoTabsDialog from '../InfoTabsDialog.vue';
import CharacterInfoModal from '../CharacterInfoModal.vue';

const normalizeSnapshotHtml = (html: string): string =>
  html
    .replace(/>\s+([^<]*?)\s+</g, (_, text: string) => `>${text.trim()}<`)
    .replace(/>\s+</g, '><')
    .replace(/\s{2,}/g, ' ')
    .trim();

vi.mock('vue-router', () => ({
  useRouter: () => ({
    push: vi.fn(),
  }),
}));

vi.mock('../../stores/gameStore', () => ({
  useGameStore: () => ({
    playerCharacter: {
      name: '无名弟子',
      location: 'sect_valley',
      stats: {
        cultivation_realm: { name: '炼气', level: 1, sub_level: 2 },
        combat_power: 123,
      },
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
      segment_count: 2,
      last_option_generation_source: 'llm_structured',
      interaction_state: 'waiting_for_choice',
      last_generation_diagnostics: '',
      last_consistency_risk_score: 2,
    },
    gameState: {
      script: {
        world_setting: {
          locations: [{ id: 'sect_valley', name: '山门', spiritual_energy: 0.7 }],
        },
      },
      event_history: [],
    },
    reachableLocationIds: ['sect_valley'],
    mapOverview: [],
    currentScript: null,
    isGameInitialized: true,
    isWaitingForInput: true,
    isPlotInitialized: false,
    availableOptions: [{ id: 1, description: '观察四周', requirements: [], action: {} }],
    error: null,
    executePlayerAction: vi.fn().mockResolvedValue(undefined),
    clearError: vi.fn(),
    travelToLocation: vi.fn().mockResolvedValue(undefined),
  }),
}));

describe('visual snapshots', () => {
  const mountWithTheme = (
    component: unknown,
    theme: 'theme-scroll' | 'theme-night',
    options: Record<string, unknown> = {},
  ) =>
    shallowMount(
      {
        components: { TestedComponent: component },
        template: `<div class="${theme}"><TestedComponent /></div>`,
      },
      options as never,
    );

  it('MainMenu snapshot', () => {
    const wrapper = shallowMount(MainMenu, {
      global: {
        stubs: {
          LLMConfigDialog: true,
        },
      },
    });
    expect(normalizeSnapshotHtml(wrapper.html())).toMatchSnapshot();
  });

  it('MainMenu snapshot (theme-night)', () => {
    const wrapper = mountWithTheme(MainMenu, 'theme-night', {
      global: {
        stubs: {
          LLMConfigDialog: true,
        },
      },
    });
    expect(normalizeSnapshotHtml(wrapper.html())).toMatchSnapshot();
  });

  it('ScriptSelector snapshot', () => {
    const wrapper = shallowMount(ScriptSelector, {
      global: {
        stubs: {
          LoadingIndicator: true,
          StatusBanner: true,
          UiPanel: true,
        },
      },
    });
    expect(normalizeSnapshotHtml(wrapper.html())).toMatchSnapshot();
  });

  it('ScriptSelector snapshot (theme-night)', () => {
    const wrapper = mountWithTheme(ScriptSelector, 'theme-night', {
      global: {
        stubs: {
          LoadingIndicator: true,
          StatusBanner: true,
          UiPanel: true,
        },
      },
    });
    expect(normalizeSnapshotHtml(wrapper.html())).toMatchSnapshot();
  });

  it('GameRuntimeView snapshot', () => {
    const wrapper = shallowMount(GameRuntimeView, {
      global: {
        stubs: {
          InkTopBar: true,
          PoemStatusSlip: true,
          InkStoryStage: true,
          InkQuickActionDock: true,
          GameSystemDialogs: true,
          GameInfoCenterDialog: true,
          CharacterInfoModal: true,
          NotificationCenter: true,
        },
      },
    });
    expect(normalizeSnapshotHtml(wrapper.html())).toMatchSnapshot();
  });

  it('GameRuntimeView snapshot (theme-night)', () => {
    const wrapper = mountWithTheme(GameRuntimeView, 'theme-night', {
      global: {
        stubs: {
          InkTopBar: true,
          PoemStatusSlip: true,
          InkStoryStage: true,
          InkQuickActionDock: true,
          GameSystemDialogs: true,
          GameInfoCenterDialog: true,
          CharacterInfoModal: true,
          NotificationCenter: true,
        },
      },
    });
    expect(normalizeSnapshotHtml(wrapper.html())).toMatchSnapshot();
  });

  it('InfoTabsDialog drawer snapshot', () => {
    const wrapper = shallowMount(InfoTabsDialog, {
      props: {
        isOpen: true,
        playerName: '无名弟子',
        playerRealm: '炼气 (1-2)',
        playerCombatPower: '123',
        playerLocation: '山门',
        chapterProgress: '1 / 初入仙门',
        chapterInteraction: '1 / 2-4',
        segmentCount: 2,
        isWaitingForInput: true,
        worldLocations: [{ id: 'sect_valley', name: '山门', spiritual_energy: 0.7 }],
        reachableLocationIds: ['sect_valley'],
        mapOverview: [],
        recentCombatExplanations: [],
        currentLocationId: 'sect_valley',
        currentLocationLabel: '山门',
        isTraveling: false,
        isGameRunning: true,
        eventCount: 0,
        isDevMode: true,
        debugChapter: '1 / 初入仙门',
        debugOptionSource: 'llm_structured',
        debugRiskScore: 2,
        debugDiagnostics: '',
        systemError: null,
      },
      global: {
        stubs: {
          UiButton: true,
          UiPanel: true,
          NovelExporter: true,
          StatusBanner: true,
        },
      },
    });
    expect(normalizeSnapshotHtml(wrapper.html())).toMatchSnapshot();
  });

  it('CharacterInfoModal snapshot', () => {
    const wrapper = shallowMount(CharacterInfoModal, {
      props: {
        isOpen: true,
        character: {
          id: 'player_1',
          name: '花尊',
          stats: {
            cultivation_realm: { name: '炼气', level: 1, sub_level: 2 },
            combat_power: 88,
            spiritual_root: {
              element: 'Metal',
              elements: ['Metal'],
              grade: 'Heavenly',
              affinity: 0.85,
            },
            lifespan: {
              current_age: 17,
              max_age: 120,
              realm_bonus: 10,
            },
            techniques: ['清风诀'],
          },
          location: '宗门外谷',
          inventory: [],
          personality_tags: ['谨慎'],
        },
      },
      global: {
        stubs: {
          CharacterPanel: true,
        },
      },
    });
    expect(normalizeSnapshotHtml(wrapper.html())).toMatchSnapshot();
  });
});
