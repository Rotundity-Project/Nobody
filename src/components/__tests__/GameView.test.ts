import { mount } from '@vue/test-utils';
import { reactive } from 'vue';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import GameView from '../GameView.vue';
import type { PlayerOption } from '../../types/game';

const pushMock = vi.fn();
const playClickMock = vi.fn();
const executePlayerActionMock = vi.fn();
const clearErrorMock = vi.fn();
const createOptionActionMock = vi.fn();
const createContinueActionMock = vi.fn();
const createFreeTextActionMock = vi.fn();
const validateFreeTextInputMock = vi.fn();
const invokeWithTimeoutMock = vi.fn();
const getStorySettingsMock = vi.fn();
const saveStorySettingsMock = vi.fn();

vi.mock('vue-router', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
}));

vi.mock('../CharacterPanel.vue', () => ({
  default: { name: 'CharacterPanel', template: '<div />' },
}));

vi.mock('../AudioControlPanel.vue', () => ({
  default: { name: 'AudioControlPanel', template: '<div />' },
}));

vi.mock('../LLMConfigDialog.vue', () => ({
  default: { name: 'LLMConfigDialog', template: '<div />' },
}));

vi.mock('../SaveLoadDialog.vue', () => ({
  default: { name: 'SaveLoadDialog', template: '<div />' },
}));

vi.mock('../StorySettingsDialog.vue', () => ({
  default: { name: 'StorySettingsDialog', template: '<div />' },
}));

vi.mock('../LoadingIndicator.vue', () => ({
  default: { name: 'LoadingIndicator', template: '<div />' },
}));

vi.mock('../VirtualStoryList.vue', () => ({
  default: { name: 'VirtualStoryList', template: '<div />' },
}));

vi.mock('../../utils/audioSystem', () => ({
  playClick: () => playClickMock(),
}));

vi.mock('../../utils/playerInput', () => ({
  createOptionAction: (...args: unknown[]) => createOptionActionMock(...args),
  createContinueAction: () => createContinueActionMock(),
  createFreeTextAction: (...args: unknown[]) => createFreeTextActionMock(...args),
  validateFreeTextInput: (...args: unknown[]) => validateFreeTextInputMock(...args),
}));

vi.mock('../../utils/storySettings', () => ({
  getStorySettings: () => getStorySettingsMock(),
  saveStorySettings: (...args: unknown[]) => saveStorySettingsMock(...args),
}));

vi.mock('../../utils/tauriInvoke', () => ({
  invokeWithTimeout: (...args: unknown[]) => invokeWithTimeoutMock(...args),
}));

const buildStore = (overrides: Record<string, unknown> = {}) =>
  reactive({
    playerCharacter: null,
    currentScene: {
      name: '第一章',
      description: 'test',
      available_options: [],
    },
    plotState: {
      current_chapter: {
        title: '第一章',
        content: ['段落一'],
      },
      chapters: [],
    },
    currentScript: null,
    gameState: null,
    isGameInitialized: true,
    isWaitingForInput: true,
    isPlotInitialized: false,
    availableOptions: [] as PlayerOption[],
    error: null as string | null,
    executePlayerAction: executePlayerActionMock,
    clearError: clearErrorMock,
    ...overrides,
  });

const flushPromises = async () => {
  await Promise.resolve();
  await Promise.resolve();
};

let storeRef = buildStore();
vi.mock('../../stores/gameStore', () => ({
  useGameStore: () => storeRef,
}));

describe('GameView', () => {
  beforeEach(() => {
    pushMock.mockReset();
    playClickMock.mockReset();
    executePlayerActionMock.mockReset();
    clearErrorMock.mockReset();
    createOptionActionMock.mockReset();
    createContinueActionMock.mockReset();
    createFreeTextActionMock.mockReset();
    validateFreeTextInputMock.mockReset();
    invokeWithTimeoutMock.mockReset();
    getStorySettingsMock.mockReset();
    saveStorySettingsMock.mockReset();
    getStorySettingsMock.mockReturnValue({
      recap_enabled: false,
      novel_style: 'xianxia-third-person',
      min_interactions_per_chapter: 2,
      max_interactions_per_chapter: 3,
      target_chapter_words_min: 5000,
      target_chapter_words_max: 7000,
    });
    validateFreeTextInputMock.mockReturnValue({ valid: true, message: '' });
    storeRef = buildStore();
  });

  it('renders options and handles option selection', async () => {
    const optionAction = { action_type: 'SelectedOption', content: 'opt', selected_option_id: 0 };
    createOptionActionMock.mockReturnValue(optionAction);
    executePlayerActionMock.mockResolvedValue(undefined);

    storeRef = buildStore({
      availableOptions: [{ id: 0, description: '选项一', requirements: [], action: {} }],
    });

    const wrapper = mount(GameView);
    const optionButton = wrapper
      .findAll('button')
      .find((btn) => btn.text().includes('选项一'));
    expect(optionButton).toBeTruthy();
    await optionButton!.trigger('click');
    await flushPromises();

    expect(playClickMock).toHaveBeenCalled();
    expect(executePlayerActionMock).toHaveBeenCalledWith(optionAction);
  });

  it('shows continue button when not waiting for input', async () => {
    const continueAction = { action_type: 'FreeText', content: 'continue', selected_option_id: null };
    createContinueActionMock.mockReturnValue(continueAction);
    executePlayerActionMock.mockResolvedValue(undefined);

    storeRef = buildStore({
      isWaitingForInput: false,
    });

    const wrapper = mount(GameView);
    const continueButton = wrapper
      .findAll('button')
      .find((btn) => btn.text() === '继续写');
    expect(continueButton).toBeTruthy();
    await continueButton!.trigger('click');
    await flushPromises();

    expect(executePlayerActionMock).toHaveBeenCalledWith(continueAction);
  });

  it('submits free text input', async () => {
    const freeAction = { action_type: 'FreeText', content: 'hello', selected_option_id: null };
    createFreeTextActionMock.mockReturnValue(freeAction);
    executePlayerActionMock.mockResolvedValue(undefined);

    storeRef = buildStore({
      availableOptions: [{ id: 0, description: '选项一', requirements: [], action: {} }],
    });

    const wrapper = mount(GameView);
    const freeTextTab = wrapper
      .findAll('button')
      .find((btn) => btn.text() === '自由输入');
    expect(freeTextTab).toBeTruthy();
    await freeTextTab!.trigger('click');

    const textarea = wrapper.find('textarea');
    await textarea.setValue('hello');
    const submitButton = wrapper
      .findAll('button')
      .find((btn) => btn.text() === '提交自由输入');
    expect(submitButton).toBeTruthy();
    await submitButton!.trigger('click');
    await flushPromises();

    expect(executePlayerActionMock).toHaveBeenCalledWith(freeAction);
  });
});
