import { mount } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import NotificationCenter from '../NotificationCenter.vue';

describe('NotificationCenter', () => {
  it('auto dismisses info toast', async () => {
    vi.useFakeTimers();
    try {
      const wrapper = mount(NotificationCenter, {
        props: {
          notifications: [
            {
              id: 'theme-sync',
              kind: 'info',
              title: '界面主题已切换',
              message: '当前为深色风格（已自动同步）',
              priority: 'toast',
            },
          ],
        },
      });

      expect(wrapper.find('.notify-toast-info').exists()).toBe(true);
      vi.advanceTimersByTime(2700);
      await wrapper.vm.$nextTick();

      expect(wrapper.emitted('dismiss')?.[0]).toEqual(['theme-sync']);
    } finally {
      vi.useRealTimers();
    }
  });
});
