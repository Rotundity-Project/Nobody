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

    await wrapper.get('[data-testid="recent-save-btn"]').trigger('click');

    await vi.waitFor(() => {
      expect(playClickMock).toHaveBeenCalled();
      expect(loadGameMock).toHaveBeenCalledWith(2);
      expect(pushMock).toHaveBeenCalledWith('/game');
    });

    expect(wrapper.text()).toContain('最近保存：');
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

    await wrapper.get('[data-testid="refresh-save-btn"]').trigger('click');

    await vi.waitFor(() => {
      expect(listSaveSlotsMock).toHaveBeenCalledTimes(2);
    });
    expect(wrapper.get('[data-testid="recent-save-refresh-label"]').text()).toContain('最近刷新：');
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

    await wrapper.get('[data-testid="quick-volume-60-btn"]').trigger('click');
    expect(setMasterVolumeMock).toHaveBeenCalledWith(0.6);
    expect(wrapper.get('[data-testid="quick-volume-status"]').text()).toContain('当前 60%');
    expect(wrapper.get('[data-testid="quick-volume-60-btn"]').classes()).toContain('border-emerald-400');

    await wrapper.get('[data-testid="quick-mute-btn"]').trigger('click');
    expect(setMasterVolumeMock).toHaveBeenCalledWith(0);
    expect(wrapper.get('[data-testid="quick-volume-status"]').text()).toContain('已静音');
    expect(wrapper.get('[data-testid="quick-mute-btn"]').attributes('aria-pressed')).toBe('true');

    await wrapper.get('[data-testid="quick-mute-btn"]').trigger('click');
    expect(setMasterVolumeMock).toHaveBeenCalledWith(0.6);
    expect(wrapper.get('[data-testid="quick-volume-status"]').text()).toContain('当前 60%');
    expect(wrapper.get('[data-testid="quick-mute-btn"]').attributes('aria-pressed')).toBe('false');

    await wrapper.get('[data-testid="quick-volume-100-btn"]').trigger('click');
    expect(setMasterVolumeMock).toHaveBeenCalledWith(1);
    expect(wrapper.get('[data-testid="quick-volume-status"]').text()).toContain('当前 100%');
    expect(wrapper.get('[data-testid="quick-volume-100-btn"]').classes()).toContain('border-emerald-400');
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

    await wrapper.get('[data-testid="quick-bgm-btn"]').trigger('click');
    expect(setBgmEnabledMock).toHaveBeenCalledWith(false);
    expect(wrapper.get('[data-testid="quick-bgm-btn"]').attributes('aria-pressed')).toBe('false');
    expect(wrapper.get('[data-testid="quick-bgm-btn"]').text()).toContain('BGM 关');

    await wrapper.get('[data-testid="quick-sfx-btn"]').trigger('click');
    expect(setSfxEnabledMock).toHaveBeenCalledWith(false);
    expect(wrapper.get('[data-testid="quick-sfx-btn"]').attributes('aria-pressed')).toBe('false');
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
    expect(wrapper.get('[data-testid="recent-save-refresh-label"]').text()).toMatch(/刚刚|秒前|分钟前/);
    expect(wrapper.get('[data-testid="recent-save-refresh-status"]').text()).toContain('刷新状态：成功');
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

    audioSettingsState.master = 0.3;
    await wrapper.get('[data-testid="open-audio-btn"]').trigger('click');

    expect(wrapper.get('[data-testid="quick-volume-status"]').text()).toContain('当前 30%');
    expect(wrapper.get('[data-testid="quick-volume-30-btn"]').classes()).toContain('border-emerald-400');
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
    expect(wrapper.get('[data-testid="recent-save-refresh-status"]').text()).toContain('刷新状态：失败');

    await wrapper.get('[data-testid="retry-load-saves-btn"]').trigger('click');

    await vi.waitFor(() => {
      expect(listSaveSlotsMock).toHaveBeenCalledTimes(2);
    });
    await vi.waitFor(() => {
      expect(wrapper.text()).not.toContain('读取失败');
    });
    expect(wrapper.get('[data-testid="recent-save-refresh-status"]').text()).toContain('刷新状态：成功');
  });
});
