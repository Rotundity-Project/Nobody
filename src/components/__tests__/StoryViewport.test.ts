import { mount } from '@vue/test-utils';
import { beforeEach, describe, expect, it } from 'vitest';
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
  beforeEach(() => {
    window.localStorage.removeItem('nobody_reading_locator_expanded');
  });

  it('renders container with expected classes', () => {
    const wrapper = buildWrapper();
    expect(wrapper.classes()).toContain('relative');
    expect(wrapper.classes()).toContain('flex-1');
  });

  it('shows reading locator when story paragraphs exist', () => {
    const wrapper = buildWrapper();
    expect(wrapper.text()).toContain('阅读定位');
    expect(wrapper.find('[data-testid="reading-locator"]').exists()).toBe(true);
    expect(wrapper.get('[data-testid="reading-locator-summary"]').text()).toContain('/2');
  });

  it('toggles reading locator details', async () => {
    const wrapper = buildWrapper();
    const toggleBtn = wrapper.get('[data-testid="toggle-reading-locator"]');

    expect(wrapper.text()).not.toContain('段落进度');

    await toggleBtn.trigger('click');

    expect(wrapper.text()).toContain('段落进度');
    expect(window.localStorage.getItem('nobody_reading_locator_expanded')).toBe('1');
    expect(toggleBtn.attributes('aria-expanded')).toBe('true');
  });

  it('restores locator details state from localStorage', async () => {
    window.localStorage.setItem('nobody_reading_locator_expanded', '0');
    const wrapper = buildWrapper();
    await wrapper.vm.$nextTick();

    expect(wrapper.text()).not.toContain('段落进度');
    expect(wrapper.get('[data-testid="toggle-reading-locator"]').attributes('aria-expanded')).toBe('false');
  });

  it('exposes scrollToBottom method', () => {
    const wrapper = buildWrapper();
    expect(typeof (wrapper.vm as { scrollToBottom?: unknown }).scrollToBottom).toBe('function');
  });

  it('resets scroll to top when chapter changes', async () => {
    const wrapper = buildWrapper();
    const host = wrapper.element as HTMLElement;
    host.scrollTop = 120;

    await wrapper.setProps({
      chapterTitle: '第二章',
      paragraphs: ['新段落一', '新段落二'],
    });
    await wrapper.vm.$nextTick();

    expect(host.scrollTop).toBe(0);
  });
});
