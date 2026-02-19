import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import ContinueActionPanel from '../ContinueActionPanel.vue';

describe('ContinueActionPanel', () => {
  it('renders message and emits continue', async () => {
    const wrapper = mount(ContinueActionPanel, {
      props: {
        message: '当前无需输入，点击继续即可推进剧情。',
        buttonText: '继续推进剧情',
      },
    });

    expect(wrapper.text()).toContain('当前无需输入');
    const button = wrapper.find('button');
    expect(button.text()).toContain('继续推进剧情');
    await button.trigger('click');
    expect(wrapper.emitted('continue')).toBeTruthy();
  });

  it('can render without message', () => {
    const wrapper = mount(ContinueActionPanel, {
      props: {
        buttonText: '继续写',
      },
    });
    expect(wrapper.text()).toContain('继续写');
  });
});
