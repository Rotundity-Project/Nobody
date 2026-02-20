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
      paragraphs: ['段落一', '段落二'],
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

  it('shows reading locator when story paragraphs exist', () => {
    const wrapper = buildWrapper();
    expect(wrapper.text()).toContain('阅读定位');
    expect(wrapper.text()).toContain('段落进度');
    expect(wrapper.find('[data-testid="reading-locator"]').exists()).toBe(true);
  });

  it('exposes scrollToBottom method', () => {
    const wrapper = buildWrapper();
    expect(typeof (wrapper.vm as { scrollToBottom?: unknown }).scrollToBottom).toBe('function');
  });
});
