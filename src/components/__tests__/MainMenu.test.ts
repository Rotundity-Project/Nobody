import { mount } from '@vue/test-utils';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import MainMenu from '../MainMenu.vue';

const pushMock = vi.fn();
const playClickMock = vi.fn();
const setMasterVolumeMock = vi.fn();
const setBgmEnabledMock = vi.fn();
const setSfxEnabledMock = vi.fn();
const listSaveSlotsMock = vi.fn();
const loadGameMock = vi.fn();
const audioSettingsState = {
  master: 0.55,
  bgmEnabled: true,
  sfxEnabled: true,
};

vi.mock('vue-router', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
}));

vi.mock('../../utils/audioSystem', () => ({
  getAudioSettings: () => ({ ...audioSettingsState }),
  playClick: () => playClickMock(),
  setMasterVolume: (value: number) => setMasterVolumeMock(value),
  setBgmEnabled: (enabled: boolean) => setBgmEnabledMock(enabled),
  setSfxEnabled: (enabled: boolean) => setSfxEnabledMock(enabled),
}));

vi.mock('../../stores/gameStore', () => ({
  useGameStore: () => ({
    listSaveSlots: listSaveSlotsMock,
    loadGame: loadGameMock,
  }),
}));

vi.mock('../../platform/runtimeEnv', () => ({
  isTauriRuntime: () => false,
}));

const AudioStub = { name: 'AudioControlPanel', template: '<div />' };
const LlmStub = { name: 'LLMConfigDialog', props: ['isOpen', 'inline'], template: '<div />' };
const SaveLoadStub = {
  name: 'SaveLoadDialog',
  props: ['isOpen', 'mode'],
  template: '<div />',
};

describe('MainMenu', () => {
  beforeEach(() => {
    pushMock.mockReset();
    playClickMock.mockReset();
    setMasterVolumeMock.mockReset();
    setBgmEnabledMock.mockReset();
    setSfxEnabledMock.mockReset();
    listSaveSlotsMock.mockReset();
    loadGameMock.mockReset();
    audioSettingsState.master = 0.55;
    audioSettingsState.bgmEnabled = true;
    audioSettingsState.sfxEnabled = true;
    listSaveSlotsMock.mockResolvedValue([]);
    loadGameMock.mockResolvedValue(undefined);
    window.localStorage.removeItem('nobody_web_onboarding_seen_v1');
  });

  it('shows web onboarding once and persists dismiss state', async () => {
    const wrapper = mount(MainMenu, {
      global: {
        stubs: {
          AudioControlPanel: AudioStub,
          LLMConfigDialog: LlmStub,
          SaveLoadDialog: SaveLoadStub,
        },
      },
    });

    await wrapper.vm.$nextTick();
    expect(wrapper.find('[data-testid="web-onboarding-banner"]').exists()).toBe(true);
    await wrapper.get('[data-testid="web-onboarding-dismiss-btn"]').trigger('click');
    expect(wrapper.find('[data-testid="web-onboarding-banner"]').exists()).toBe(false);
    expect(window.localStorage.getItem('nobody_web_onboarding_seen_v1')).toBe('1');
  });

  it('navigates to script select on new game', async () => {
    const wrapper = mount(MainMenu, {
      global: {
        stubs: {
          AudioControlPanel: AudioStub,
          LLMConfigDialog: LlmStub,
          SaveLoadDialog: SaveLoadStub,
        },
      },
    });

    await wrapper.get('[data-testid="new-game-btn"]').trigger('click');

    expect(playClickMock).toHaveBeenCalled();
    expect(pushMock).toHaveBeenCalledWith('/script-select');
    expect(wrapper.get('[data-testid="new-game-btn"]').attributes('type')).toBe('button');
  });

  it('opens LLM config dialog', async () => {
    const wrapper = mount(MainMenu, {
      global: {
        stubs: {
          AudioControlPanel: AudioStub,
          LLMConfigDialog: LlmStub,
          SaveLoadDialog: SaveLoadStub,
        },
      },
    });

    await wrapper.get('[data-testid="llm-settings-btn"]').trigger('click');

    const dialog = wrapper.findComponent(LlmStub);
    expect(dialog.exists()).toBe(true);
    expect(dialog.props('isOpen')).toBe(true);
  });

  it('updates theme class when ui theme event is dispatched', async () => {
    window.localStorage.setItem('nobody_ui_theme', 'theme-scroll');
    const wrapper = mount(MainMenu, {
      global: {
        stubs: {
          AudioControlPanel: AudioStub,
          LLMConfigDialog: LlmStub,
          SaveLoadDialog: SaveLoadStub,
        },
      },
    });

    expect(wrapper.classes()).toContain('theme-scroll');
    window.dispatchEvent(new CustomEvent('nobody:ui-theme-changed', { detail: 'theme-night' }));
    await wrapper.vm.$nextTick();
    expect(wrapper.classes()).toContain('theme-night');
    expect(wrapper.find('[data-testid="ui-theme-status"]').exists()).toBe(true);
    expect(wrapper.text()).toContain('界面主题已切换');
    expect(wrapper.text()).toContain('当前为深色风格（已自动同步）');
  });

  it('loads latest save and navigates to game', async () => {
    listSaveSlotsMock.mockResolvedValue([
      {
        slot_id: 2,
        timestamp: 1700000000,
        player_name: '无名弟子',
        realm: '炼气',
        location: 'sect_valley',
        game_time: '第1年1月',
      },
    ]);

    const wrapper = mount(MainMenu, {
      global: {
        stubs: {
          AudioControlPanel: AudioStub,
          LLMConfigDialog: LlmStub,
          SaveLoadDialog: SaveLoadStub,
        },
      },
    });

    await vi.waitFor(() => {
      expect(listSaveSlotsMock).toHaveBeenCalled();
    });

    await vi.waitFor(() => {
      expect(wrapper.get('[data-testid="recent-save-btn"]').attributes('disabled')).toBeUndefined();
    });
    expect(wrapper.get('[data-testid="recent-save-btn"]').attributes('aria-label'))
      .toContain('槽位 2');
    expect(wrapper.get('[data-testid="recent-save-btn"]').attributes('aria-label'))
      .toContain('无名弟子');
    expect(wrapper.get('[data-testid="recent-save-btn"]').attributes('aria-label'))
      .toContain('炼气');
    expect(wrapper.get('[data-testid="recent-save-btn"]').attributes('aria-describedby'))
      .toBe('latest-save-summary');

    await wrapper.get('[data-testid="recent-save-btn"]').trigger('click');

    await vi.waitFor(() => {
      expect(playClickMock).toHaveBeenCalled();
      expect(loadGameMock).toHaveBeenCalledWith(2);
      expect(pushMock).toHaveBeenCalledWith('/game');
    });

    expect(wrapper.text()).toContain('最近保存：');
    expect(wrapper.text()).toMatch(/秒前|分钟前|小时前|天前|时间未知/);
    expect(wrapper.get('[data-testid="latest-save-summary"]').attributes('aria-live')).toBe('polite');
    expect(wrapper.get('[data-testid="latest-save-summary"]').attributes('aria-atomic')).toBe('true');
  });

  it('hides location tag when latest save location is unknown', async () => {
    listSaveSlotsMock.mockResolvedValue([
      {
        slot_id: 3,
        timestamp: 1700000000,
        player_name: '无名弟子',
        realm: '炼气',
        location: '',
        game_time: '第1年2月',
      },
    ]);

    const wrapper = mount(MainMenu, {
      global: {
        stubs: {
          AudioControlPanel: AudioStub,
          LLMConfigDialog: LlmStub,
          SaveLoadDialog: SaveLoadStub,
        },
      },
    });

    await vi.waitFor(() => {
      expect(listSaveSlotsMock).toHaveBeenCalled();
    });
    await vi.waitFor(() => {
      expect(wrapper.get('[data-testid="refresh-save-btn"]').text()).toBe('刷新存档');
    });

    expect(wrapper.text()).not.toContain('位置：未知');
    expect(wrapper.text()).toContain('时间：第1年2月');
  });

  it('uses fallback labels when latest save player and realm are empty', async () => {
    listSaveSlotsMock.mockResolvedValue([
      {
        slot_id: 7,
        timestamp: 1700000000,
        player_name: '  ',
        realm: '',
        location: 'sect_valley',
        game_time: '   ',
      },
    ]);

    const wrapper = mount(MainMenu, {
      global: {
        stubs: {
          AudioControlPanel: AudioStub,
          LLMConfigDialog: LlmStub,
          SaveLoadDialog: SaveLoadStub,
        },
      },
    });

    await vi.waitFor(() => {
      expect(wrapper.find('[data-testid="latest-save-summary"]').exists()).toBe(true);
    });

    const summary = wrapper.get('[data-testid="latest-save-summary"]').text();
    expect(summary).toContain('槽位 7 · 未命名角色 · 境界未知');
    expect(summary).toContain('时间：时间未知');
  });

  it('shows concise unknown text when latest save timestamp is invalid', async () => {
    listSaveSlotsMock.mockResolvedValue([
      {
        slot_id: 9,
        timestamp: Number.NaN,
        player_name: '无名弟子',
        realm: '炼气',
        location: 'sect_valley',
        game_time: '第3年1月',
      },
    ]);

    const wrapper = mount(MainMenu, {
      global: {
        stubs: {
          AudioControlPanel: AudioStub,
          LLMConfigDialog: LlmStub,
          SaveLoadDialog: SaveLoadStub,
        },
      },
    });

    await vi.waitFor(() => {
      expect(wrapper.find('[data-testid="latest-save-summary"]').exists()).toBe(true);
    });

    const summary = wrapper.get('[data-testid="latest-save-summary"]').text();
    expect(summary).toContain('最近保存：时间未知');
    expect(summary).not.toContain('未知（时间未知）');
  });

  it('refreshes save slots when refresh button is clicked', async () => {
    const wrapper = mount(MainMenu, {
      global: {
        stubs: {
          AudioControlPanel: AudioStub,
          LLMConfigDialog: LlmStub,
          SaveLoadDialog: SaveLoadStub,
        },
      },
    });

    await vi.waitFor(() => {
      expect(listSaveSlotsMock).toHaveBeenCalledTimes(1);
    });

    await vi.waitFor(() => {
      expect(wrapper.get('[data-testid="refresh-save-btn"]').text()).toBe('刷新存档');
    });
    expect(wrapper.find('#save-actions-heading').exists()).toBe(true);
    expect(wrapper.find('[data-testid="save-actions-group"]').exists()).toBe(true);
    const saveActionsGroup = wrapper.get('[data-testid="save-actions-group"]');
    expect(saveActionsGroup.attributes('aria-describedby')).toContain('no-save-hint');
    expect(saveActionsGroup.attributes('aria-describedby')).toContain('recent-save-refresh-label');
    expect(saveActionsGroup.attributes('aria-describedby')).toContain('recent-save-refresh-status');
    expect(wrapper.get('[data-testid="recent-save-btn"]').attributes('aria-label'))
      .toBe('继续最近存档，当前没有可用存档');
    expect(wrapper.get('[data-testid="recent-save-btn"]').attributes('aria-describedby'))
      .toBe('no-save-hint');
    expect(wrapper.get('[data-testid="refresh-save-btn"]').attributes('aria-controls')).toBe('recent-save-card');
    expect(wrapper.get('[data-testid="refresh-save-btn"]').attributes('aria-label'))
      .toContain('最近一次刷新成功');
    expect(wrapper.get('[data-testid="refresh-save-btn"]').attributes('aria-label'))
      .toContain('时间');
    expect(wrapper.get('[data-testid="refresh-save-btn"]').attributes('aria-describedby'))
      .toContain('recent-save-refresh-label');
    expect(wrapper.get('[data-testid="refresh-save-btn"]').attributes('aria-describedby'))
      .toContain('recent-save-refresh-status');
    expect(wrapper.get('[data-testid="recent-save-card"]').attributes('id')).toBe('recent-save-card');

    await wrapper.get('[data-testid="refresh-save-btn"]').trigger('click');

    await vi.waitFor(() => {
      expect(listSaveSlotsMock).toHaveBeenCalledTimes(2);
    });
    expect(wrapper.get('[data-testid="recent-save-refresh-label"]').text()).toContain('最近刷新：');
  });

  it('marks recent save card as busy while loading slots', async () => {
    let resolveSlots!: (value: any[]) => void;
    const pendingSlots = new Promise<any[]>((resolve) => {
      resolveSlots = resolve;
    });
    listSaveSlotsMock.mockImplementationOnce(() => pendingSlots);

    const wrapper = mount(MainMenu, {
      global: {
        stubs: {
          AudioControlPanel: AudioStub,
          LLMConfigDialog: LlmStub,
          SaveLoadDialog: SaveLoadStub,
        },
      },
    });

    await vi.waitFor(() => {
      expect(wrapper.get('[data-testid="recent-save-card"]').attributes('aria-busy')).toBe('true');
    });
    expect(wrapper.get('[data-testid="loading-save-hint"]').attributes('role')).toBe('status');
    expect(wrapper.get('[data-testid="loading-save-hint"]').attributes('aria-live')).toBe('polite');
    expect(wrapper.get('[data-testid="loading-save-hint"]').attributes('aria-atomic')).toBe('true');
    expect(wrapper.get('[data-testid="save-actions-group"]').attributes('aria-describedby'))
      .toContain('loading-save-hint');
    expect(wrapper.get('[data-testid="refresh-save-btn"]').attributes('aria-describedby'))
      .toBe('recent-save-refresh-status');
    expect(wrapper.get('[data-testid="recent-save-refresh-status"]').text()).toContain('刷新状态：刷新中');
    expect(wrapper.get('[data-testid="recent-save-refresh-status"]').classes()).toContain('menu-status-loading');

    resolveSlots([]);
    await vi.waitFor(() => {
      expect(wrapper.get('[data-testid="recent-save-card"]').attributes('aria-busy')).toBe('false');
    });
  });

  it('shows refresh label after initial fetch', async () => {
    const wrapper = mount(MainMenu, {
      global: {
        stubs: {
          AudioControlPanel: AudioStub,
          LLMConfigDialog: LlmStub,
          SaveLoadDialog: SaveLoadStub,
        },
      },
    });

    await vi.waitFor(() => {
      expect(listSaveSlotsMock).toHaveBeenCalledTimes(1);
    });

    await vi.waitFor(() => {
      expect(wrapper.get('[data-testid="refresh-save-btn"]').text()).toBe('刷新存档');
    });

    expect(wrapper.get('[data-testid="recent-save-refresh-label"]').text()).toContain('最近刷新：');
    expect(wrapper.get('[data-testid="recent-save-refresh-label"]').attributes('id')).toBe('recent-save-refresh-label');
    expect(wrapper.get('[data-testid="recent-save-refresh-label"]').attributes('role')).toBe('status');
    expect(wrapper.get('[data-testid="recent-save-refresh-label"]').attributes('aria-live')).toBe('polite');
    expect(wrapper.get('[data-testid="recent-save-refresh-label"]').attributes('aria-atomic')).toBe('true');
    expect(wrapper.get('[data-testid="recent-save-refresh-label"]').text()).toMatch(/刚刚|秒前|分钟前/);
    expect(wrapper.get('[data-testid="recent-save-refresh-status"]').text()).toContain('刷新状态：成功');
    expect(wrapper.get('[data-testid="recent-save-refresh-status"]').attributes('id')).toBe('recent-save-refresh-status');
    expect(wrapper.get('[data-testid="recent-save-refresh-status"]').attributes('role')).toBe('status');
    expect(wrapper.get('[data-testid="recent-save-refresh-status"]').attributes('aria-live')).toBe('polite');
    expect(wrapper.get('[data-testid="recent-save-refresh-status"]').attributes('aria-atomic')).toBe('true');
    expect(wrapper.get('[data-testid="refresh-save-btn"]').attributes('aria-describedby'))
      .toContain('recent-save-refresh-status');
    expect(wrapper.get('[data-testid="no-save-hint"]').attributes('role')).toBe('status');
    expect(wrapper.get('[data-testid="no-save-hint"]').attributes('aria-live')).toBe('polite');
    expect(wrapper.get('[data-testid="no-save-hint"]').attributes('aria-atomic')).toBe('true');
    expect(wrapper.get('[data-testid="no-save-hint"]').text()).toContain('暂无可用存档');
  });

  it('syncs quick volume status from audio settings when opening audio panel', async () => {
    const wrapper = mount(MainMenu, {
      global: {
        stubs: {
          AudioControlPanel: AudioStub,
          LLMConfigDialog: LlmStub,
          SaveLoadDialog: SaveLoadStub,
        },
      },
    });

    expect(wrapper.get('[data-testid="quick-volume-status"]').text()).toContain('当前 55%');
    expect(wrapper.get('[data-testid="quick-volume-status"]').attributes('id')).toBe('quick-volume-status');
    expect(wrapper.get('[data-testid="quick-volume-status"]').attributes('role')).toBe('status');
    expect(wrapper.get('[data-testid="quick-volume-status"]').attributes('aria-live')).toBe('polite');
    expect(wrapper.get('[data-testid="quick-volume-status"]').attributes('aria-atomic')).toBe('true');
    const audioToggle = wrapper.get('[data-testid="open-audio-btn"]');
    expect(audioToggle.attributes('aria-describedby')).toBe('quick-volume-status');
    expect(audioToggle.attributes('aria-label')).toContain('展开音量控制');
    expect(audioToggle.attributes('aria-label')).toContain('当前音量 55%');
    expect(audioToggle.attributes('aria-expanded')).toBe('false');
    expect(wrapper.find('#main-menu-audio-panel').exists()).toBe(false);

    audioSettingsState.master = 0.3;
    await audioToggle.trigger('click');

    expect(wrapper.get('[data-testid="quick-volume-status"]').text()).toContain('当前 30%');
    expect(audioToggle.attributes('aria-label')).toContain('收起音量控制');
    expect(audioToggle.attributes('aria-label')).toContain('当前音量 30%');
    expect(audioToggle.attributes('aria-expanded')).toBe('true');
    expect(wrapper.find('#main-menu-audio-panel').exists()).toBe(true);
    expect(wrapper.find('[data-testid="quick-audio-group"]').exists()).toBe(false);
  });

  it('shows retry button when load saves failed and retries successfully', async () => {
    listSaveSlotsMock
      .mockRejectedValueOnce(new Error('读取失败'))
      .mockResolvedValueOnce([]);

    const wrapper = mount(MainMenu, {
      global: {
        stubs: {
          AudioControlPanel: AudioStub,
          LLMConfigDialog: LlmStub,
          SaveLoadDialog: SaveLoadStub,
        },
      },
    });

    await vi.waitFor(() => {
      expect(wrapper.text()).toContain('读取失败');
    });
    expect(wrapper.find('[data-testid="retry-load-saves-btn"]').exists()).toBe(true);
    expect(wrapper.get('[data-testid="recent-save-error"]').attributes('role')).toBe('alert');
    expect(wrapper.get('[data-testid="recent-save-error"]').attributes('aria-live')).toBe('assertive');
    expect(wrapper.get('[data-testid="recent-save-error"]').attributes('aria-atomic')).toBe('true');
    const retryButton = wrapper.get('[data-testid="retry-load-saves-btn"]');
    expect(retryButton.attributes('aria-label')).toContain('最近错误：读取失败');
    expect(retryButton.attributes('aria-describedby')).toContain('recent-save-error');
    expect(retryButton.attributes('aria-describedby')).toContain('recent-save-refresh-label');
    expect(retryButton.attributes('aria-describedby')).toContain('recent-save-refresh-status');
    expect(wrapper.get('[data-testid="recent-save-refresh-status"]').text()).toContain('刷新状态：失败');
    expect(wrapper.get('[data-testid="recent-save-refresh-status"]').attributes('role')).toBe('alert');
    expect(wrapper.get('[data-testid="recent-save-refresh-status"]').attributes('aria-live')).toBe('assertive');
    expect(wrapper.get('[data-testid="refresh-save-btn"]').attributes('aria-label'))
      .toContain('最近一次刷新失败');
    expect(wrapper.get('[data-testid="refresh-save-btn"]').attributes('aria-label'))
      .toContain('时间');
    expect(wrapper.get('[data-testid="refresh-save-btn"]').attributes('aria-describedby'))
      .toContain('recent-save-error');
    expect(wrapper.get('[data-testid="save-actions-group"]').attributes('aria-describedby'))
      .toContain('recent-save-error');
    expect(wrapper.get('[data-testid="recent-save-btn"]').attributes('aria-describedby'))
      .toBe('recent-save-error no-save-hint');

    await wrapper.get('[data-testid="retry-load-saves-btn"]').trigger('click');

    await vi.waitFor(() => {
      expect(listSaveSlotsMock).toHaveBeenCalledTimes(2);
    });
    await vi.waitFor(() => {
      expect(wrapper.text()).not.toContain('读取失败');
    });
    expect(wrapper.get('[data-testid="recent-save-refresh-status"]').text()).toContain('刷新状态：成功');
  });

  it('hides success refresh status after a short duration', async () => {
    vi.useFakeTimers();
    try {
      const wrapper = mount(MainMenu, {
        global: {
          stubs: {
            AudioControlPanel: AudioStub,
            LLMConfigDialog: LlmStub,
            SaveLoadDialog: SaveLoadStub,
          },
        },
      });

      await vi.waitFor(() => {
        expect(listSaveSlotsMock).toHaveBeenCalledTimes(1);
      });

      await vi.waitFor(() => {
        expect(wrapper.get('[data-testid="recent-save-refresh-status"]').text()).toContain('刷新状态：成功');
      });

      vi.advanceTimersByTime(9000);
      await wrapper.vm.$nextTick();

      expect(wrapper.find('[data-testid="recent-save-refresh-status"]').exists()).toBe(false);
      expect(wrapper.get('[data-testid="refresh-save-btn"]').attributes('aria-describedby'))
        .toBe('recent-save-refresh-label');
    } finally {
      vi.useRealTimers();
    }
  });
});
