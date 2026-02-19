import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import QuickActionsBar from '../QuickActionsBar.vue';

describe('QuickActionsBar', () => {
  it('emits actions on button clicks', async () => {
    const wrapper = mount(QuickActionsBar, {
      props: {
        isGameInitialized: true,
      },
    });

    const labels = ['角色信息', '信息面板', '保存', '加载'];
    for (const label of labels) {
      const button = wrapper.findAll('button').find((btn) => btn.text() === label);
      expect(button).toBeTruthy();
      await button!.trigger('click');
    }

    expect(wrapper.emitted('open-character')).toBeTruthy();
    expect(wrapper.emitted('open-info')).toBeTruthy();
    expect(wrapper.emitted('open-save')).toBeTruthy();
    expect(wrapper.emitted('open-load')).toBeTruthy();
  });

  it('disables save button when game is not initialized', () => {
    const wrapper = mount(QuickActionsBar, {
      props: {
        isGameInitialized: false,
      },
    });

    const saveButton = wrapper.findAll('button').find((btn) => btn.text() === '保存');
    expect(saveButton?.attributes('disabled')).toBeDefined();
  });
});
