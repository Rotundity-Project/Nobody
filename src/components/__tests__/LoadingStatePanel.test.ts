import { mount } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import LoadingStatePanel from '../LoadingStatePanel.vue';

vi.mock('../LoadingIndicator.vue', () => ({
  default: {
    name: 'LoadingIndicator',
    props: ['message', 'detail', 'size', 'progress', 'progressText'],
    template:
      '<div class="loading-indicator">{{ message }}|{{ detail }}|{{ size }}|{{ progress }}|{{ progressText }}</div>',
  },
}));

describe('LoadingStatePanel', () => {
  it('passes loading message and progress data to indicator', () => {
    const wrapper = mount(LoadingStatePanel, {
      props: {
        message: '正在续写剧情...',
        progress: 58,
        progressText: '模型生成中，请稍候...',
        elapsedMs: 1260,
      },
    });

    const indicator = wrapper.find('.loading-indicator');
    expect(indicator.exists()).toBe(true);
    expect(indicator.text()).toContain('正在续写剧情...');
    expect(indicator.text()).toContain('请稍候，剧情正在推进...');
    expect(indicator.text()).toContain('lg');
    expect(indicator.text()).toContain('58');
    expect(indicator.text()).toContain('模型生成中，请稍候...');
    expect(wrapper.text()).toContain('已耗时 1.3 s');
  });

  it('shows stop button and emits when auto advance can be interrupted', async () => {
    const wrapper = mount(LoadingStatePanel, {
      props: {
        message: '正在自动推进剧情...',
        canStopAutoAdvance: true,
      },
    });

    const stopButton = wrapper.findAll('button').find((btn) => btn.text() === '中断自动推进');
    expect(stopButton).toBeTruthy();
    await stopButton!.trigger('click');
    expect(wrapper.emitted('stop-auto-advance')).toBeTruthy();
  });
});
