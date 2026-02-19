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

vi.mock('../InfoTabsDialog.vue', () => ({
  default: {
    name: 'InfoTabsDialog',
    props: ['debugRiskScore'],
    template: '<div data-testid="info-tabs-dialog">{{ debugRiskScore }}</div>',
  },
}));

vi.mock('../ConsistencySettingsDialog.vue', () => ({
  default: { name: 'ConsistencySettingsDialog', template: '<div />' },
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

  it('auto advances when backend keeps no-input state', async () => {
    const continueAction = { action_type: 'FreeText', content: 'continue', selected_option_id: null };
    createContinueActionMock.mockReturnValue(continueAction);

    let callCount = 0;
    executePlayerActionMock.mockImplementation(async () => {
      callCount += 1;
      if (callCount < 3) {
        storeRef.isWaitingForInput = false;
        storeRef.availableOptions = [];
        storeRef.plotState = {
          current_chapter: {
            title: 'chapter',
            content: ['paragraph', `p${callCount}`],
          },
          chapters: [],
          segment_count: callCount,
          last_option_generation_source: 'llm_structured',
        } as any;
      } else {
        storeRef.isWaitingForInput = true;
        storeRef.availableOptions = [{ id: 0, description: '选项A', requirements: [], action: {} }];
        storeRef.plotState = {
          current_chapter: {
            title: 'chapter',
            content: ['paragraph', 'p3'],
          },
          chapters: [],
          segment_count: 3,
          last_option_generation_source: 'llm_structured',
        } as any;
      }
    });

    storeRef = buildStore({
      isWaitingForInput: false,
      availableOptions: [],
    });

    const wrapper = mount(GameView);
    const continueButton = wrapper
      .findAll('button')
      .find((btn) => /(继续|推进)/.test(btn.text()));
    expect(continueButton).toBeTruthy();
    await continueButton!.trigger('click');
    await flushPromises();

    expect(executePlayerActionMock).toHaveBeenCalledTimes(3);
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

  it('shows continue action when option source says no input required', async () => {
    const continueAction = { action_type: 'FreeText', content: 'continue', selected_option_id: null };
    createContinueActionMock.mockReturnValue(continueAction);
    executePlayerActionMock.mockImplementation(async () => {
      storeRef.plotState = {
        current_chapter: {
          title: 'chapter',
          content: ['paragraph'],
        },
        chapters: [],
        last_option_generation_source: 'llm_structured',
      } as any;
      storeRef.availableOptions = [{ id: 0, description: '选项一', requirements: [], action: {} }];
      storeRef.isWaitingForInput = true;
    });

    storeRef = buildStore({
      isWaitingForInput: true,
      availableOptions: [],
      plotState: {
        current_chapter: {
          title: 'chapter',
          content: ['paragraph'],
        },
        chapters: [],
        last_option_generation_source: 'not_waiting_for_input',
      } as any,
    });

    const wrapper = mount(GameView);
    const continueButton = wrapper
      .findAll('button')
      .find((btn) => btn.text().includes('继续'));
    expect(continueButton).toBeTruthy();
    await continueButton!.trigger('click');
    await flushPromises();

    expect(executePlayerActionMock).toHaveBeenCalledWith(continueAction);
    expect(executePlayerActionMock).toHaveBeenCalledTimes(1);
  });

  it('auto advances multiple times when no input is required repeatedly', async () => {
    const continueAction = { action_type: 'FreeText', content: 'continue', selected_option_id: null };
    createContinueActionMock.mockReturnValue(continueAction);

    let callCount = 0;
    executePlayerActionMock.mockImplementation(async () => {
      callCount += 1;
      if (callCount < 3) {
        storeRef.plotState = {
          current_chapter: {
            title: 'chapter',
            content: ['paragraph', `p${callCount}`],
          },
          chapters: [],
          segment_count: callCount,
          last_option_generation_source: 'not_waiting_for_input',
        } as any;
        storeRef.availableOptions = [];
        storeRef.isWaitingForInput = true;
      } else {
        storeRef.plotState = {
          current_chapter: {
            title: 'chapter',
            content: ['paragraph', 'p3'],
          },
          chapters: [],
          segment_count: 3,
          last_option_generation_source: 'llm_structured',
        } as any;
        storeRef.availableOptions = [{ id: 0, description: '选项一', requirements: [], action: {} }];
        storeRef.isWaitingForInput = true;
      }
    });

    storeRef = buildStore({
      isWaitingForInput: true,
      availableOptions: [],
      plotState: {
        current_chapter: {
          title: 'chapter',
          content: ['paragraph'],
        },
        chapters: [],
        last_option_generation_source: 'not_waiting_for_input',
      } as any,
    });

    const wrapper = mount(GameView);
    const continueButton = wrapper
      .findAll('button')
      .find((btn) => btn.text().includes('继续'));
    expect(continueButton).toBeTruthy();
    await continueButton!.trigger('click');
    await flushPromises();

    expect(executePlayerActionMock).toHaveBeenCalledTimes(3);
  });

  it('loads consistency policy when opening consistency settings', async () => {
    invokeWithTimeoutMock.mockResolvedValue({
      recent_window: 3,
      cross_chapter_window: 3,
      duplicate_recent_threshold: 0.92,
      duplicate_cross_chapter_threshold: 0.88,
      weight_warning: 5,
      weight_critical: 12,
      code_weights: {},
    });

    const wrapper = mount(GameView);
    const settingsBtn = wrapper
      .findAll('button')
      .find((btn) => btn.text().includes('一致性设置'));
    expect(settingsBtn).toBeTruthy();
    await settingsBtn!.trigger('click');
    await flushPromises();

    expect(invokeWithTimeoutMock).toHaveBeenCalledWith(
      'get_consistency_policy',
      undefined,
      8000,
      '读取一致性策略超时',
    );
  });

  it('shows structured consistency risk score in debug panel', () => {
    storeRef = buildStore({
      plotState: {
        current_chapter: {
          title: 'chapter',
          content: ['paragraph'],
          index: 1,
        },
        chapters: [],
        segment_count: 1,
        last_option_generation_source: 'llm_structured',
        last_generation_diagnostics: 'diag',
        last_consistency_risk_score: 23,
      } as any,
    });

    const wrapper = mount(GameView);
    const dialog = wrapper.get('[data-testid="info-tabs-dialog"]');
    expect(dialog.text()).toContain('23');
  });
});

