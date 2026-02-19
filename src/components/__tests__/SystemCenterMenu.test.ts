import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import SystemCenterMenu from '../SystemCenterMenu.vue';

describe('SystemCenterMenu', () => {
  it('emits action events from menu items', async () => {
    const wrapper = mount(SystemCenterMenu, {
      props: {
        isOpen: true,
        showAudioPanel: false,
      },
      global: {
        stubs: {
          AudioControlPanel: { template: '<div />' },
        },
      },
    });

    const labels = ['快捷键', 'LLM 设置', '剧情设置', '一致性设置', '音量设置'];
    for (const label of labels) {
      const button = wrapper.findAll('button').find((item) => item.text() === label);
      expect(button).toBeTruthy();
      await button!.trigger('click');
    }

    expect(wrapper.emitted('open-shortcuts')).toBeTruthy();
    expect(wrapper.emitted('open-llm')).toBeTruthy();
    expect(wrapper.emitted('open-story-settings')).toBeTruthy();
    expect(wrapper.emitted('open-consistency')).toBeTruthy();
    expect(wrapper.emitted('toggle-audio')).toBeTruthy();
  });

  it('emits close-menu when clicking outside while open', async () => {
    const wrapper = mount(SystemCenterMenu, {
      props: {
        isOpen: true,
        showAudioPanel: false,
      },
      attachTo: document.body,
      global: {
        stubs: {
          AudioControlPanel: { template: '<div />' },
        },
      },
    });

    document.body.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
    await wrapper.vm.$nextTick();

    expect(wrapper.emitted('close-menu')).toBeTruthy();
    wrapper.unmount();
  });
});
