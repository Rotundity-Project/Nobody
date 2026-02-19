import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import StoryViewport from '../StoryViewport.vue';

const buildWrapper = () =>
  mount(StoryViewport, {
    props: {
      hasScene: true,
      chapterTitle: '第一章',
      showRecap: false,
      recapSummary: '',
      paragraphs: ['段落一'],
      optionSourceLabel: '',
      isGameInitialized: true,
    },
    global: {
      stubs: {
        StoryScenePanel: true,
        ScrollToBottomButton: true,
      },
    },
  });

describe('StoryViewport', () => {
  it('renders container with expected classes', () => {
    const wrapper = buildWrapper();
    expect(wrapper.classes()).toContain('relative');
    expect(wrapper.classes()).toContain('flex-1');
  });

  it('exposes scrollToBottom method', () => {
    const wrapper = buildWrapper();
    expect(typeof (wrapper.vm as { scrollToBottom?: unknown }).scrollToBottom).toBe('function');
  });
});
