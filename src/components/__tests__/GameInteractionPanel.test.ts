import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import GameInteractionPanel from '../GameInteractionPanel.vue';
import type { PlayerOption } from '../../types/game';

const defaultOptions: PlayerOption[] = [{ id: 1, description: '选项一', requirements: [], action: {} }];

const buildProps = (overrides: Record<string, unknown> = {}) => ({
  shouldShowInputPanel: true,
  error: null,
  isNoInputAdvanceState: false,
  availableOptions: defaultOptions,
  inputMode: 'options' as const,
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
  ...overrides,
});

describe('GameInteractionPanel', () => {
  it('renders input branch and forwards mode switch and option select', async () => {
    const wrapper = mount(GameInteractionPanel, {
      props: buildProps(),
      global: {
        stubs: {
          InputStatusNotice: true,
          InputModeTabs: {
            template: '<button class="mode-btn" @click="$emit(\'switch-mode\', \'freeText\')">mode</button>',
          },
          OptionListPanel: {
            template: '<button class="option-btn" @click="$emit(\'select\', { id: 9, description: \'x\', requirements: [], action: {} })">option</button>',
          },
          FreeTextInputPanel: true,
          ContinueActionPanel: true,
          LoadingStatePanel: true,
          UiPanel: { template: '<div><slot /></div>' },
        },
      },
    });

    await wrapper.find('.mode-btn').trigger('click');
    await wrapper.find('.option-btn').trigger('click');
    expect(wrapper.emitted('switch-mode')?.[0]).toEqual(['freeText']);
    expect(wrapper.emitted('select-option')?.[0]?.[0]).toMatchObject({ id: 9 });
  });

  it('renders loading branch', () => {
    const wrapper = mount(GameInteractionPanel, {
      props: buildProps({
        shouldShowInputPanel: false,
        isLoading: true,
      }),
      global: {
        stubs: {
          LoadingStatePanel: {
            props: ['message'],
            template: '<div class="loading">{{ message }}</div>',
          },
          UiPanel: { template: '<div><slot /></div>' },
        },
      },
    });
    expect(wrapper.find('.loading').exists()).toBe(true);
  });

  it('renders no-input continue branch and emits continue', async () => {
    const wrapper = mount(GameInteractionPanel, {
      props: buildProps({
        shouldShowInputPanel: false,
        isNoInputAdvanceState: true,
      }),
      global: {
        stubs: {
          ContinueActionPanel: {
            template: '<button class="continue-btn" @click="$emit(\'continue\')">go</button>',
          },
          UiPanel: { template: '<div><slot /></div>' },
        },
      },
    });

    await wrapper.find('.continue-btn').trigger('click');
    expect(wrapper.emitted('continue')).toBeTruthy();
  });
});
