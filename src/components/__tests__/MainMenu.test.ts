import { mount } from '@vue/test-utils';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import MainMenu from '../MainMenu.vue';

const pushMock = vi.fn();
const playClickMock = vi.fn();

vi.mock('vue-router', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
}));

vi.mock('../../utils/audioSystem', () => ({
  playClick: () => playClickMock(),
}));

const AudioStub = { name: 'AudioControlPanel', template: '<div />' };
const LlmStub = { name: 'LLMConfigDialog', props: ['isOpen'], template: '<div />' };

describe('MainMenu', () => {
  beforeEach(() => {
    pushMock.mockReset();
    playClickMock.mockReset();
  });

  it('navigates to script select on new game', async () => {
    const wrapper = mount(MainMenu, {
      global: {
        stubs: {
          AudioControlPanel: AudioStub,
          LLMConfigDialog: LlmStub,
        },
      },
    });

    const newGame = wrapper
      .findAll('button')
      .find((btn) => btn.text() === '新游戏');
    expect(newGame).toBeTruthy();
    await newGame!.trigger('click');

    expect(playClickMock).toHaveBeenCalled();
    expect(pushMock).toHaveBeenCalledWith('/script-select');
  });

  it('opens LLM config dialog', async () => {
    const wrapper = mount(MainMenu, {
      global: {
        stubs: {
          AudioControlPanel: AudioStub,
          LLMConfigDialog: LlmStub,
        },
      },
    });

    const settings = wrapper
      .findAll('button')
      .find((btn) => btn.text() === 'LLM 设置');
    expect(settings).toBeTruthy();
    await settings!.trigger('click');

    const dialog = wrapper.findComponent(LlmStub);
    expect(dialog.exists()).toBe(true);
    expect(dialog.props('isOpen')).toBe(true);
  });
});
