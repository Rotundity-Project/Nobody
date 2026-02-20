import { mount } from '@vue/test-utils';
import { beforeEach, describe, expect, it } from 'vitest';
import StoryViewport from '../StoryViewport.vue';

const ScrollStub = {
  name: 'ScrollToBottomButton',
  props: ['visible'],
  template: '<div data-testid="scroll-bottom-visible">{{ visible }}</div>',
};

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
        ScrollToBottomButton: ScrollStub,
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
    const locatorSummary = wrapper.get('[data-testid="reading-locator-summary"]');
    expect(locatorSummary.text()).toContain('/2');
    expect(locatorSummary.attributes('aria-live')).toBe('polite');
    expect(locatorSummary.attributes('aria-atomic')).toBe('true');
    expect(wrapper.get('[data-testid="scroll-bottom-visible"]').text()).toBe('true');
  });

  it('hides scroll-to-bottom button when reading progress reaches bottom', async () => {
    const wrapper = buildWrapper();
    const host = wrapper.element as HTMLElement;
    Object.defineProperty(host, 'scrollHeight', { configurable: true, value: 400 });
    Object.defineProperty(host, 'clientHeight', { configurable: true, value: 200 });
    host.scrollTop = 200;
    host.dispatchEvent(new Event('scroll'));
    await wrapper.vm.$nextTick();

    expect(wrapper.get('[data-testid="scroll-bottom-visible"]').text()).toBe('false');
  });

  it('uses auto scroll behavior when reduced motion is preferred', () => {
    const originalMatchMedia = window.matchMedia;
    Object.defineProperty(window, 'matchMedia', {
      configurable: true,
      value: () => ({ matches: true }),
    });

    const wrapper = buildWrapper();
    const host = wrapper.element as HTMLElement & {
      scrollTo?: (options?: ScrollToOptions | number, y?: number) => void;
      scrollHeight: number;
    };
    Object.defineProperty(host, 'scrollHeight', { configurable: true, value: 480 });
    const calls: Array<{ top: number; behavior: ScrollBehavior }> = [];
    host.scrollTo = (options) => {
      if (!options || typeof options === 'number') {
        return;
      }
      calls.push({
        top: typeof options.top === 'number' ? options.top : 0,
        behavior: options.behavior ?? 'auto',
      });
    };

    (wrapper.vm as { scrollToBottom: () => void }).scrollToBottom();

    expect(calls.length).toBe(1);
    expect(calls[0]?.behavior).toBe('auto');

    Object.defineProperty(window, 'matchMedia', {
      configurable: true,
      value: originalMatchMedia,
    });
  });

  it('toggles reading locator details', async () => {
    const wrapper = buildWrapper();
    const toggleBtn = wrapper.get('[data-testid="toggle-reading-locator"]');
    expect(toggleBtn.attributes('aria-controls')).toBe('reading-locator-details');
    expect(toggleBtn.attributes('aria-label')).toBe('展开阅读定位详情');

    expect(wrapper.text()).not.toContain('段落进度');

    await toggleBtn.trigger('click');

    expect(wrapper.text()).toContain('段落进度');
    expect(wrapper.find('#reading-locator-details').exists()).toBe(true);
    expect(window.localStorage.getItem('nobody_reading_locator_expanded')).toBe('1');
    expect(toggleBtn.attributes('aria-expanded')).toBe('true');
    expect(toggleBtn.attributes('aria-label')).toBe('收起阅读定位详情');
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
