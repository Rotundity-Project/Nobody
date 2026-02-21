import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
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
  it('renders container with expected classes', () => {
    const wrapper = buildWrapper();
    expect(wrapper.classes()).toContain('relative');
    expect(wrapper.classes()).toContain('flex-1');
  });

  it('does not render reading locator block', () => {
    const wrapper = buildWrapper();
    expect(wrapper.text()).not.toContain('阅读定位');
    expect(wrapper.find('[data-testid="reading-locator"]').exists()).toBe(false);
  });

  it('shows scroll-to-bottom button when content is scrollable and not at bottom', async () => {
    const wrapper = buildWrapper();
    const host = wrapper.get('.runtime-story-scroll').element as HTMLElement;
    Object.defineProperty(host, 'scrollHeight', { configurable: true, value: 420 });
    Object.defineProperty(host, 'clientHeight', { configurable: true, value: 180 });
    host.scrollTop = 0;
    host.dispatchEvent(new Event('scroll'));
    await wrapper.vm.$nextTick();
    expect(wrapper.get('[data-testid="scroll-bottom-visible"]').text()).toBe('true');
  });

  it('hides scroll-to-bottom button when reading progress reaches bottom', async () => {
    const wrapper = buildWrapper();
    const host = wrapper.get('.runtime-story-scroll').element as HTMLElement;
    Object.defineProperty(host, 'scrollHeight', { configurable: true, value: 400 });
    Object.defineProperty(host, 'clientHeight', { configurable: true, value: 200 });
    host.scrollTop = 200;
    host.dispatchEvent(new Event('scroll'));
    await wrapper.vm.$nextTick();
    expect(wrapper.get('[data-testid="scroll-bottom-visible"]').text()).toBe('false');
  });

  it('hides scroll-to-bottom button when content is not scrollable', async () => {
    const wrapper = buildWrapper();
    const host = wrapper.get('.runtime-story-scroll').element as HTMLElement;
    Object.defineProperty(host, 'scrollHeight', { configurable: true, value: 180 });
    Object.defineProperty(host, 'clientHeight', { configurable: true, value: 180 });
    host.scrollTop = 0;
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
    const host = wrapper.get('.runtime-story-scroll').element as HTMLElement & {
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

  it('exposes scrollToBottom method', () => {
    const wrapper = buildWrapper();
    expect(typeof (wrapper.vm as { scrollToBottom?: unknown }).scrollToBottom).toBe('function');
  });

  it('resets scroll to top when chapter changes', async () => {
    const wrapper = buildWrapper();
    const host = wrapper.get('.runtime-story-scroll').element as HTMLElement;
    host.scrollTop = 120;

    await wrapper.setProps({
      chapterTitle: '第二章',
      paragraphs: ['新段落一', '新段落二'],
    });
    await wrapper.vm.$nextTick();

    expect(host.scrollTop).toBe(0);
  });

  it('shows page navigation when story content is too long', async () => {
    const wrapper = mount(StoryViewport, {
      props: {
        hasScene: true,
        chapterTitle: '第一章',
        showRecap: false,
        recapSummary: '',
        paragraphs: Array.from({ length: 14 }, (_, i) => `第${i + 1}段：` + '剧情'.repeat(80)),
        optionSourceLabel: '',
        isGameInitialized: true,
      },
      global: {
        stubs: {
          ScrollToBottomButton: ScrollStub,
          StoryScenePanel: true,
        },
      },
    });

    expect(wrapper.text()).toContain('第 1 /');
    const nextBtn = wrapper.findAll('button').find((btn) => btn.text() === '下一页');
    expect(nextBtn).toBeTruthy();
    await nextBtn!.trigger('click');
    expect(wrapper.text()).toContain('第 2 /');
  });

  it('supports keyboard pagination for long content', async () => {
    const wrapper = mount(StoryViewport, {
      props: {
        hasScene: true,
        chapterTitle: '第一章',
        showRecap: false,
        recapSummary: '',
        paragraphs: Array.from({ length: 14 }, (_, i) => `第${i + 1}段：` + '剧情'.repeat(80)),
        optionSourceLabel: '',
        isGameInitialized: true,
      },
      global: {
        stubs: {
          ScrollToBottomButton: ScrollStub,
          StoryScenePanel: true,
        },
      },
    });

    expect(wrapper.text()).toContain('第 1 /');
    await wrapper.get('.runtime-story-scroll').trigger('keydown', { key: 'ArrowRight' });
    expect(wrapper.text()).toContain('第 2 /');
    await wrapper.get('.runtime-story-scroll').trigger('keydown', { key: 'ArrowLeft' });
    expect(wrapper.text()).toContain('第 1 /');
  });
});
