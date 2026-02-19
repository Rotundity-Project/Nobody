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
    expect(visibleWrapper.find('button').exists()).toBe(true);
  });

  it('emits scroll event when clicked', async () => {
    const wrapper = mount(ScrollToBottomButton, {
      props: { visible: true },
    });
    await wrapper.find('button').trigger('click');
    expect(wrapper.emitted('scroll')).toBeTruthy();
  });
});
