import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import ScrollToBottomButton from '../ScrollToBottomButton.vue';

describe('ScrollToBottomButton', () => {
  it('renders only when visible', () => {
    const hiddenWrapper = mount(ScrollToBottomButton, {
      props: { visible: false },
    });
    expect(hiddenWrapper.find('button').exists()).toBe(false);

    const visibleWrapper = mount(ScrollToBottomButton, {
      props: { visible: true },
    });
    const button = visibleWrapper.find('button');
    expect(button.exists()).toBe(true);
    expect(button.attributes('type')).toBe('button');
    expect(visibleWrapper.text()).toContain('回到底部');
  });

  it('emits scroll event when clicked', async () => {
    const wrapper = mount(ScrollToBottomButton, {
      props: { visible: true },
    });
    await wrapper.find('button').trigger('click');
    expect(wrapper.emitted('scroll')).toBeTruthy();
  });
});


