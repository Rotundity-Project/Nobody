import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import GameSystemDialogs from '../GameSystemDialogs.vue';

const baseProps = {
  showSaveDialog: true,
  showLoadDialog: true,
  showShortcutsDialog: true,
  showLLMDialog: true,
  showStorySettings: true,
  showConsistencySettings: true,
  storySettings: {
    recap_enabled: true,
    novel_style: 'xianxia-third-person',
    llm_priority_mode: true,
    llm_strict_mode: false,
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
};

describe('GameSystemDialogs', () => {
  it('forwards save/load and close events', async () => {
    const wrapper = mount(GameSystemDialogs, {
      props: baseProps,
      global: {
        stubs: {
          SaveLoadDialog: {
            props: ['mode'],
            template: `
              <button
                :class="'slot-' + mode"
                @click="$emit(mode === 'save' ? 'saved' : 'loaded', 3)"
              >
                {{ mode }}
              </button>
            `,
          },
          KeyboardShortcutsDialog: {
            template: '<button class="close-shortcuts" @click="$emit(\'close\')" />',
          },
          LLMConfigDialog: {
            template: '<button class="close-llm" @click="$emit(\'close\')" />',
          },
          StorySettingsDialog: true,
          ConsistencySettingsDialog: true,
        },
      },
    });

    await wrapper.find('.slot-save').trigger('click');
    await wrapper.find('.slot-load').trigger('click');
    await wrapper.find('.close-shortcuts').trigger('click');
    await wrapper.find('.close-llm').trigger('click');

    expect(wrapper.emitted('saved')?.[0]).toEqual([3]);
    expect(wrapper.emitted('loaded')?.[0]).toEqual([3]);
    expect(wrapper.emitted('close-shortcuts')).toBeTruthy();
    expect(wrapper.emitted('close-llm')).toBeTruthy();
  });

  it('forwards story and consistency actions', async () => {
    const wrapper = mount(GameSystemDialogs, {
      props: baseProps,
      global: {
        stubs: {
          SaveLoadDialog: true,
          KeyboardShortcutsDialog: true,
          LLMConfigDialog: true,
          StorySettingsDialog: {
            template: '<button class="save-story" @click="$emit(\'save\', { recap_enabled: false })" />',
          },
          ConsistencySettingsDialog: {
            template: `
              <div>
                <button class="save-consistency" @click="$emit('save', { recent_window: 2 })" />
                <button class="reset-consistency" @click="$emit('reset')" />
              </div>
            `,
          },
        },
      },
    });

    await wrapper.find('.save-story').trigger('click');
    await wrapper.find('.save-consistency').trigger('click');
    await wrapper.find('.reset-consistency').trigger('click');

    expect(wrapper.emitted('save-story-settings')).toBeTruthy();
    expect(wrapper.emitted('save-consistency')).toBeTruthy();
    expect(wrapper.emitted('reset-consistency')).toBeTruthy();
  });
});
