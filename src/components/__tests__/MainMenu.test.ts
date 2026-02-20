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

const AudioStub = { name: 'AudioControlPanel', template: '<div />' };
const LlmStub = { name: 'LLMConfigDialog', props: ['isOpen'], template: '<div />' };
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
    expect(wrapper.find('[role="group"][aria-labelledby="save-actions-heading"]').exists()).toBe(true);
    const saveActionsGroup = wrapper.get('[role="group"][aria-labelledby="save-actions-heading"]');
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
    expect(wrapper.get('[role="group"][aria-labelledby="save-actions-heading"]').attributes('aria-describedby'))
      .toContain('loading-save-hint');
    expect(wrapper.get('[data-testid="refresh-save-btn"]').attributes('aria-describedby'))
      .toBe('recent-save-refresh-status');
    expect(wrapper.get('[data-testid="recent-save-refresh-status"]').text()).toContain('刷新状态：刷新中');
    expect(wrapper.get('[data-testid="recent-save-refresh-status"]').classes()).toContain('text-sky-300');

    resolveSlots([]);
    await vi.waitFor(() => {
      expect(wrapper.get('[data-testid="recent-save-card"]').attributes('aria-busy')).toBe('false');
    });
  });

  it('applies quick volume preset and mute toggle', async () => {
    const wrapper = mount(MainMenu, {
      global: {
        stubs: {
          AudioControlPanel: AudioStub,
          LLMConfigDialog: LlmStub,
          SaveLoadDialog: SaveLoadStub,
        },
      },
    });
    const quickVolumeGroup = wrapper.find('[role="group"][aria-label*="快捷音量预设"]');
    expect(quickVolumeGroup.exists()).toBe(true);
    expect(wrapper.find('#quick-audio-heading').exists()).toBe(true);
    expect(quickVolumeGroup.attributes('aria-labelledby')).toBe('quick-audio-heading');
    expect(quickVolumeGroup.attributes('aria-describedby')).toBe('quick-volume-status');
    expect(quickVolumeGroup.attributes('aria-label')).toContain('当前音量 55%');
    expect(quickVolumeGroup.attributes('aria-label')).toContain('BGM 开');
    expect(quickVolumeGroup.attributes('aria-label')).toContain('音效 开');

    await wrapper.get('[data-testid="quick-volume-60-btn"]').trigger('click');
    expect(setMasterVolumeMock).toHaveBeenCalledWith(0.6);
    expect(wrapper.get('[data-testid="quick-volume-status"]').text()).toContain('当前 60%');
    expect(wrapper.get('[data-testid="quick-mute-btn"]').attributes('aria-label')).toContain('静音，当前音量 60%');
    expect(wrapper.get('[data-testid="quick-volume-60-btn"]').classes()).toContain('border-emerald-400');
    expect(wrapper.get('[data-testid="quick-volume-60-btn"]').attributes('aria-label')).toBe('音量预设 60%，当前已选中');
    expect(wrapper.get('[data-testid="quick-volume-60-btn"]').attributes('aria-describedby')).toBe('quick-volume-status');
    expect(wrapper.get('[data-testid="quick-mute-btn"]').attributes('aria-describedby')).toBe('quick-volume-status');
    expect(wrapper.get('[data-testid="quick-volume-60-btn"]').attributes('aria-pressed')).toBe('true');
    expect(wrapper.get('[data-testid="quick-volume-30-btn"]').attributes('aria-describedby')).toBe('quick-volume-status');
    expect(wrapper.get('[data-testid="quick-volume-30-btn"]').attributes('aria-label')).toBe('音量预设 30%，点击设置');
    expect(wrapper.get('[data-testid="quick-volume-30-btn"]').attributes('aria-pressed')).toBe('false');

    await wrapper.get('[data-testid="quick-mute-btn"]').trigger('click');
    expect(setMasterVolumeMock).toHaveBeenCalledWith(0);
    expect(wrapper.get('[data-testid="quick-volume-status"]').text()).toContain('已静音');
    expect(quickVolumeGroup.attributes('aria-label')).toContain('当前静音');
    expect(wrapper.get('[data-testid="quick-mute-btn"]').attributes('aria-label')).toContain('恢复音量，恢复到 60%');
    expect(wrapper.get('[data-testid="quick-mute-btn"]').attributes('aria-pressed')).toBe('true');

    await wrapper.get('[data-testid="quick-mute-btn"]').trigger('click');
    expect(setMasterVolumeMock).toHaveBeenCalledWith(0.6);
    expect(wrapper.get('[data-testid="quick-volume-status"]').text()).toContain('当前 60%');
    expect(wrapper.get('[data-testid="quick-mute-btn"]').attributes('aria-label')).toContain('静音，当前音量 60%');
    expect(wrapper.get('[data-testid="quick-mute-btn"]').attributes('aria-pressed')).toBe('false');

    await wrapper.get('[data-testid="quick-volume-100-btn"]').trigger('click');
    expect(setMasterVolumeMock).toHaveBeenCalledWith(1);
    expect(wrapper.get('[data-testid="quick-volume-status"]').text()).toContain('当前 100%');
    expect(quickVolumeGroup.attributes('aria-label')).toContain('当前音量 100%');
    expect(wrapper.get('[data-testid="quick-volume-100-btn"]').classes()).toContain('border-emerald-400');
    expect(wrapper.get('[data-testid="quick-volume-100-btn"]').attributes('aria-label')).toBe('音量预设 100%，当前已选中');
    expect(wrapper.get('[data-testid="quick-volume-100-btn"]').attributes('aria-describedby')).toBe('quick-volume-status');
    expect(wrapper.get('[data-testid="quick-volume-100-btn"]').attributes('aria-pressed')).toBe('true');
    expect(wrapper.get('[data-testid="quick-volume-60-btn"]').attributes('aria-label')).toBe('音量预设 60%，点击设置');
    expect(wrapper.get('[data-testid="quick-volume-60-btn"]').attributes('aria-pressed')).toBe('false');
  });

  it('toggles quick bgm and sfx buttons', async () => {
    const wrapper = mount(MainMenu, {
      global: {
        stubs: {
          AudioControlPanel: AudioStub,
          LLMConfigDialog: LlmStub,
          SaveLoadDialog: SaveLoadStub,
        },
      },
    });

    expect(wrapper.get('[data-testid="quick-bgm-btn"]').text()).toContain('BGM 开');
    expect(wrapper.get('[data-testid="quick-sfx-btn"]').text()).toContain('音效 开');
    expect(wrapper.get('[data-testid="quick-bgm-btn"]').attributes('aria-label')).toBe('关闭 BGM，当前已开启');
    expect(wrapper.get('[data-testid="quick-sfx-btn"]').attributes('aria-label')).toBe('关闭音效，当前已开启');
    expect(wrapper.get('[data-testid="quick-bgm-btn"]').attributes('aria-describedby')).toBe('quick-volume-status');
    expect(wrapper.get('[data-testid="quick-sfx-btn"]').attributes('aria-describedby')).toBe('quick-volume-status');

    await wrapper.get('[data-testid="quick-bgm-btn"]').trigger('click');
    expect(setBgmEnabledMock).toHaveBeenCalledWith(false);
    expect(wrapper.get('[data-testid="quick-bgm-btn"]').attributes('aria-pressed')).toBe('false');
    expect(wrapper.get('[data-testid="quick-bgm-btn"]').attributes('aria-label')).toBe('开启 BGM，当前已关闭');
    expect(wrapper.get('[data-testid="quick-bgm-btn"]').text()).toContain('BGM 关');

    await wrapper.get('[data-testid="quick-sfx-btn"]').trigger('click');
    expect(setSfxEnabledMock).toHaveBeenCalledWith(false);
    expect(wrapper.get('[data-testid="quick-sfx-btn"]').attributes('aria-pressed')).toBe('false');
    expect(wrapper.get('[data-testid="quick-sfx-btn"]').attributes('aria-label')).toBe('开启音效，当前已关闭');
    expect(wrapper.get('[data-testid="quick-sfx-btn"]').text()).toContain('音效 关');
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
    expect(audioToggle.attributes('aria-label')).toContain('BGM 开');
    expect(audioToggle.attributes('aria-label')).toContain('音效 开');
    expect(audioToggle.attributes('aria-expanded')).toBe('false');
    expect(wrapper.find('#main-menu-audio-panel').exists()).toBe(false);

    audioSettingsState.master = 0.3;
    await audioToggle.trigger('click');

    expect(wrapper.get('[data-testid="quick-volume-status"]').text()).toContain('当前 30%');
    expect(wrapper.get('[data-testid="quick-volume-30-btn"]').classes()).toContain('border-emerald-400');
    expect(wrapper.get('[data-testid="quick-volume-30-btn"]').attributes('aria-pressed')).toBe('true');
    expect(wrapper.get('[data-testid="quick-volume-60-btn"]').attributes('aria-pressed')).toBe('false');
    expect(audioToggle.attributes('aria-label')).toContain('收起音量控制');
    expect(audioToggle.attributes('aria-label')).toContain('当前音量 30%');
    expect(audioToggle.attributes('aria-expanded')).toBe('true');
    expect(wrapper.find('#main-menu-audio-panel').exists()).toBe(true);
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
    expect(wrapper.get('[role="group"][aria-labelledby="save-actions-heading"]').attributes('aria-describedby'))
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
