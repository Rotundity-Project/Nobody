import { mount } from '@vue/test-utils';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import MainMenu from '../MainMenu.vue';

const pushMock = vi.fn();
const playClickMock = vi.fn();
const listSaveSlotsMock = vi.fn();
const loadGameMock = vi.fn();

vi.mock('vue-router', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
}));

vi.mock('../../utils/audioSystem', () => ({
  playClick: () => playClickMock(),
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
    listSaveSlotsMock.mockReset();
    loadGameMock.mockReset();
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
  });
});
