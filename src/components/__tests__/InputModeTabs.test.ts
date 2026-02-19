import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import InputModeTabs from '../InputModeTabs.vue';

describe('InputModeTabs', () => {
  it('emits switch-mode when clicking tabs', async () => {
    const wrapper = mount(InputModeTabs, {
      props: {
        visible: true,
        mode: 'options',
      },
    });

    const freeTextBtn = wrapper.findAll('button').find((btn) => btn.text() === '自由输入');
    expect(freeTextBtn).toBeTruthy();
    await freeTextBtn!.trigger('click');
    expect(wrapper.emitted('switch-mode')?.[0]).toEqual(['freeText']);

    const optionsBtn = wrapper.findAll('button').find((btn) => btn.text() === '选项');
    expect(optionsBtn).toBeTruthy();
    await optionsBtn!.trigger('click');
    expect(wrapper.emitted('switch-mode')?.[1]).toEqual(['options']);
  });

  it('does not render when hidden', () => {
    const wrapper = mount(InputModeTabs, {
      props: {
        visible: false,
        mode: 'options',
      },
    });
    expect(wrapper.findAll('button')).toHaveLength(0);
  });
});
