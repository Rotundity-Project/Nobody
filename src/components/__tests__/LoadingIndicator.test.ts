import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import LoadingIndicator from '../LoadingIndicator.vue';

describe('LoadingIndicator', () => {
  it('uses explicit size classes for lg spinner', () => {
    const wrapper = mount(LoadingIndicator, {
      props: {
        size: 'lg',
        message: '正在生成随机剧本',
      },
    });

    const spinner = wrapper.find('.animate-spin');
    expect(spinner.exists()).toBe(true);
    expect(spinner.classes()).toContain('h-8');
    expect(spinner.classes()).toContain('w-8');
  });
});
