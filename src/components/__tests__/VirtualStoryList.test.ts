import { mount } from '@vue/test-utils';
import { nextTick } from 'vue';
import { describe, expect, it, vi } from 'vitest';
import VirtualStoryList from '../VirtualStoryList.vue';

class ResizeObserverMock {
  callback: () => void;
  constructor(callback: () => void) {
    this.callback = callback;
  }
  observe() {
    this.callback();
  }
  disconnect() {}
}

describe('VirtualStoryList', () => {
  it('renders a subset of paragraphs and responds to scroll', async () => {
    vi.stubGlobal('ResizeObserver', ResizeObserverMock as unknown as typeof ResizeObserver);
    vi.stubGlobal('requestAnimationFrame', (cb: FrameRequestCallback) => {
      cb(0);
      return 1;
    });
    vi.stubGlobal('cancelAnimationFrame', () => {});

    const scrollEl = document.createElement('div');
    Object.defineProperty(scrollEl, 'clientHeight', { value: 200, configurable: true });
    Object.defineProperty(scrollEl, 'clientWidth', { value: 1000, configurable: true });
    Object.defineProperty(scrollEl, 'scrollTop', { value: 0, writable: true, configurable: true });

    const paragraphs = Array.from({ length: 160 }, (_, i) => `段落内容 ${i} ${'x'.repeat(120)}`);

    const wrapper = mount(VirtualStoryList, {
      props: {
        paragraphs,
        scrollElement: scrollEl,
      },
      attachTo: document.body,
    });

    await nextTick();
    const initialCount = wrapper.findAll('p').length;
    expect(initialCount).toBeGreaterThan(0);
    expect(initialCount).toBeLessThan(paragraphs.length);

    scrollEl.scrollTop = 2000;
    scrollEl.dispatchEvent(new Event('scroll'));
    await nextTick();

    const afterScrollCount = wrapper.findAll('p').length;
    expect(afterScrollCount).toBeGreaterThan(0);
    expect(afterScrollCount).toBeLessThan(paragraphs.length);
    wrapper.unmount();
  });
});
