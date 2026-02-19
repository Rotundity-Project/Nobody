import { mount } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import LoadingStatePanel from '../LoadingStatePanel.vue';

vi.mock('../LoadingIndicator.vue', () => ({
  default: {
    name: 'LoadingIndicator',
    props: ['message', 'detail', 'size'],
    template: '<div class="loading-indicator">{{ message }}|{{ detail }}|{{ size }}</div>',
  },
}));

describe('LoadingStatePanel', () => {
  it('passes loading message and fixed detail to indicator', () => {
    const wrapper = mount(LoadingStatePanel, {
      props: {
        message: '正在续写剧情...',
      },
    });

    const indicator = wrapper.find('.loading-indicator');
    expect(indicator.exists()).toBe(true);
    expect(indicator.text()).toContain('正在续写剧情...');
    expect(indicator.text()).toContain('请稍候，剧情正在推进...');
    expect(indicator.text()).toContain('lg');
  });
});
