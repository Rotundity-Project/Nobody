import { mount, shallowMount } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import MainMenu from '../MainMenu.vue';
import ScriptSelector from '../ScriptSelector.vue';
import GameRuntimeView from '../GameRuntimeView.vue';
import InfoTabsDialog from '../InfoTabsDialog.vue';
import CharacterInfoModal from '../CharacterInfoModal.vue';
import StoryViewport from '../StoryViewport.vue';
import GameInteractionPanel from '../GameInteractionPanel.vue';
import InkQuickActionDock from '../InkQuickActionDock.vue';
import RuntimeQuickPanelsDialog from '../RuntimeQuickPanelsDialog.vue';
import GameSystemDialogs from '../GameSystemDialogs.vue';
import GameRuntimeTopBar from '../GameRuntimeTopBar.vue';
import GameRuntimeBottomBar from '../GameRuntimeBottomBar.vue';
import NotificationCenter from '../NotificationCenter.vue';
import { Element, Grade } from '../../types/game';

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

const withStoredUiTheme = (
  theme: 'theme-scroll' | 'theme-night',
  run: () => void,
) => {
  const key = 'nobody_ui_theme';
  const previous = window.localStorage.getItem(key);
  window.localStorage.setItem(key, theme);
  try {
    run();
  } finally {
    if (previous === null) {
      window.localStorage.removeItem(key);
    } else {
      window.localStorage.setItem(key, previous);
    }
  }
};

describe('visual snapshots', () => {
  const mountWithTheme = (
    component: unknown,
    theme: 'theme-scroll' | 'theme-night',
    options: Record<string, unknown> = {},
  ) => {
    const { props, ...rest } = options;
    return mount(
      {
        components: { TestedComponent: component },
        data: () => ({
          childProps: props ?? {},
        }),
        template: `<div class="${theme}"><TestedComponent v-bind="childProps" /></div>`,
      },
      rest as never,
    );
  };

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
    withStoredUiTheme('theme-night', () => {
      const wrapper = shallowMount(MainMenu, {
        global: {
          stubs: {
            LLMConfigDialog: true,
          },
        },
      });
      expect(normalizeSnapshotHtml(wrapper.html())).toMatchSnapshot();
    });
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
    withStoredUiTheme('theme-night', () => {
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
  });

  it('GameRuntimeView snapshot', () => {
    const wrapper = shallowMount(GameRuntimeView, {
      global: {
        stubs: {
          GameRuntimeTopBar: true,
          GameRuntimeBottomBar: true,
          GameRuntimeLeftStatusPanels: true,
          GameRuntimeMainHeader: true,
          GameRuntimeWorldRegistryPanel: true,
          GameRuntimeInteractionCard: true,
          StoryViewport: true,
          GameSystemDialogs: true,
          GameInfoCenterDialog: true,
          CharacterInfoModal: true,
          RuntimeQuickPanelsDialog: true,
          NotificationCenter: true,
        },
      },
    });
    expect(normalizeSnapshotHtml(wrapper.html())).toMatchSnapshot();
  });

  it('GameRuntimeView snapshot (theme-night)', () => {
    withStoredUiTheme('theme-night', () => {
      const wrapper = shallowMount(GameRuntimeView, {
        global: {
          stubs: {
            GameRuntimeTopBar: true,
            GameRuntimeBottomBar: true,
            GameRuntimeLeftStatusPanels: true,
            GameRuntimeMainHeader: true,
            GameRuntimeWorldRegistryPanel: true,
            GameRuntimeInteractionCard: true,
            StoryViewport: true,
            GameSystemDialogs: true,
            GameInfoCenterDialog: true,
            CharacterInfoModal: true,
            RuntimeQuickPanelsDialog: true,
            NotificationCenter: true,
          },
        },
      });
      expect(normalizeSnapshotHtml(wrapper.html())).toMatchSnapshot();
    });
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
            cultivation_realm: { name: '炼气', level: 1, sub_level: 2, power_multiplier: 1.1 },
            combat_power: 88,
            spiritual_root: {
              element: Element.Metal,
              elements: [Element.Metal],
              grade: Grade.Heavenly,
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

  it('StoryViewport snapshot', () => {
    const wrapper = shallowMount(StoryViewport, {
      props: {
        hasScene: true,
        chapterTitle: '初入仙门',
        showRecap: true,
        recapSummary: '上一章回顾',
        paragraphs: ['第一段', '第二段', '第三段'],
        optionSourceLabel: '模型结构化',
        isGameInitialized: true,
      },
      global: {
        stubs: {
          ScrollToBottomButton: true,
          StoryScenePanel: true,
        },
      },
    });
    expect(normalizeSnapshotHtml(wrapper.html())).toMatchSnapshot();
  });

  it('GameInteractionPanel snapshot', () => {
    const wrapper = shallowMount(GameInteractionPanel, {
      props: {
        shouldShowInputPanel: true,
        error: null,
        isNoInputAdvanceState: false,
        availableOptions: [{ id: 1, description: '观察四周', requirements: [], action: {} }],
        inputMode: 'options',
        isLoading: false,
        freeTextInput: '',
        inputValidation: { valid: true, message: '' },
        isGameInitialized: true,
        isWaitingForInput: true,
        loadingMessage: '处理中...',
        loadingProgress: null,
        loadingProgressText: '',
        canStopAutoAdvance: false,
        autoAdvanceStopHint: '',
      },
      global: {
        stubs: {
          UiPanel: true,
          InputStatusNotice: true,
          InputModeTabs: true,
          OptionListPanel: true,
          FreeTextInputPanel: true,
          ContinueActionPanel: true,
          LoadingStatePanel: true,
        },
      },
    });
    expect(normalizeSnapshotHtml(wrapper.html())).toMatchSnapshot();
  });

  it('InkQuickActionDock snapshot', () => {
    const wrapper = shallowMount(InkQuickActionDock, {
      props: {
        isGameInitialized: true,
      },
    });
    expect(normalizeSnapshotHtml(wrapper.html())).toMatchSnapshot();
  });

  it('RuntimeQuickPanelsDialog snapshot', () => {
    const wrapper = shallowMount(RuntimeQuickPanelsDialog, {
      props: {
        isOpen: true,
        activeTab: 'world',
        panels: [
          {
            id: 'world',
            label: '世界',
            title: '世界快照',
            subtitle: '本轮世界状态索引',
            emptyText: '暂无世界快照。',
            items: [
              {
                id: 'world-1',
                title: '地图：山门',
                description: '云雾缭绕',
                meta: '灵气：中',
                badge: '当前',
                featured: true,
              },
            ],
          },
        ],
      },
    });
    expect(normalizeSnapshotHtml(wrapper.html())).toMatchSnapshot();
  });

  it('GameSystemDialogs snapshot', () => {
    const wrapper = shallowMount(GameSystemDialogs, {
      props: {
        showSaveDialog: true,
        showLoadDialog: false,
        showShortcutsDialog: false,
        showLLMDialog: false,
        showStorySettings: true,
        showConsistencySettings: false,
        uiTheme: 'theme-scroll',
        storySettings: {
          recap_enabled: true,
          novel_style: 'xianxia-third-person',
          llm_priority_mode: true,
          llm_strict_mode: true,
          min_interactions_per_chapter: 2,
          max_interactions_per_chapter: 3,
          target_chapter_words_min: 5000,
          target_chapter_words_max: 7000,
        },
        consistencyPolicy: {
          recent_window: 3,
          cross_chapter_window: 3,
          duplicate_recent_threshold: 0.92,
          duplicate_cross_chapter_threshold: 0.88,
          weight_warning: 5,
          weight_critical: 12,
          code_weights: {},
        },
      },
      global: {
        stubs: {
          SaveLoadDialog: true,
          KeyboardShortcutsDialog: true,
          LLMConfigDialog: true,
          StorySettingsDialog: true,
          ConsistencySettingsDialog: true,
        },
      },
    });
    expect(normalizeSnapshotHtml(wrapper.html())).toMatchSnapshot();
  });

  it('GameRuntimeTopBar snapshot', () => {
    const wrapper = shallowMount(GameRuntimeTopBar, {
      props: {
        chapterIndexLabel: '第一章',
        chapterNameLabel: '初入仙门',
        interactionStateToneClass: 'runtime-state-cool',
        interactionStateLabel: '等待输入',
        gameTimeLabel: '第1年 · 第1月 · 第2日',
        spiritStoneLabel: '18',
        characterCreationDurationLabel: '12.4s',
      },
    });
    expect(normalizeSnapshotHtml(wrapper.html())).toMatchSnapshot();
  });

  it('GameRuntimeTopBar snapshot (theme-night)', () => {
    const wrapper = mountWithTheme(GameRuntimeTopBar, 'theme-night', {
      props: {
        chapterIndexLabel: '第一章',
        chapterNameLabel: '初入仙门',
        interactionStateToneClass: 'runtime-state-cool',
        interactionStateLabel: '等待输入',
        gameTimeLabel: '第1年 · 第1月 · 第2日',
        spiritStoneLabel: '18',
        characterCreationDurationLabel: '12.4s',
      },
    });
    expect(normalizeSnapshotHtml(wrapper.html())).toMatchSnapshot();
  });

  it('GameRuntimeBottomBar snapshot', () => {
    const wrapper = shallowMount(GameRuntimeBottomBar, {
      props: {
        isGameInitialized: true,
        activeThemeLabel: '浅色',
      },
      global: {
        stubs: {
          InkQuickActionDock: true,
        },
      },
    });
    expect(normalizeSnapshotHtml(wrapper.html())).toMatchSnapshot();
  });

  it('GameRuntimeBottomBar snapshot (theme-night)', () => {
    const wrapper = mountWithTheme(GameRuntimeBottomBar, 'theme-night', {
      props: {
        isGameInitialized: true,
        activeThemeLabel: '深色',
      },
      global: {
        stubs: {
          InkQuickActionDock: true,
        },
      },
    });
    expect(normalizeSnapshotHtml(wrapper.html())).toMatchSnapshot();
  });

  it('NotificationCenter snapshot', () => {
    const wrapper = shallowMount(NotificationCenter, {
      props: {
        notifications: [
          { id: 'n1', kind: 'error', title: '网络异常', message: '已自动重试', priority: 'banner' },
          { id: 'n2', kind: 'info', title: '已保存', message: '保存到槽位 1', priority: 'toast' },
        ],
      },
      global: {
        stubs: {
          StatusBanner: true,
        },
      },
    });
    expect(normalizeSnapshotHtml(wrapper.html())).toMatchSnapshot();
  });

  it('NotificationCenter snapshot (theme-night)', () => {
    const wrapper = mountWithTheme(NotificationCenter, 'theme-night', {
      props: {
        notifications: [
          { id: 'n1', kind: 'error', title: '网络异常', message: '已自动重试', priority: 'banner' },
          { id: 'n2', kind: 'info', title: '已保存', message: '保存到槽位 1', priority: 'toast' },
        ],
      },
      global: {
        stubs: {
          StatusBanner: true,
        },
      },
    });
    expect(normalizeSnapshotHtml(wrapper.html())).toMatchSnapshot();
  });
});


