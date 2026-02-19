import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import InputStatusNotice from '../InputStatusNotice.vue';

describe('InputStatusNotice', () => {
  it('shows error banner first when error exists', () => {
    const wrapper = mount(InputStatusNotice, {
      props: {
        error: '发生错误',
        showAutoAdvanceHint: true,
      },
    });

    expect(wrapper.text()).toContain('系统提示');
    expect(wrapper.text()).toContain('发生错误');
    expect(wrapper.text()).not.toContain('自动推进中');
  });

  it('shows auto-advance hint when no error', () => {
    const wrapper = mount(InputStatusNotice, {
      props: {
        error: null,
        showAutoAdvanceHint: true,
      },
    });

    expect(wrapper.text()).toContain('自动推进中');
    expect(wrapper.text()).toContain('当前状态无需玩家输入。');
  });

  it('renders empty when no hint needed', () => {
    const wrapper = mount(InputStatusNotice, {
      props: {
        error: null,
        showAutoAdvanceHint: false,
      },
    });

    expect(wrapper.text()).toBe('');
  });
});
