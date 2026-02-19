import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import FreeTextInputPanel from '../FreeTextInputPanel.vue';

describe('FreeTextInputPanel', () => {
  it('emits model update and submit', async () => {
    const wrapper = mount(FreeTextInputPanel, {
      props: {
        visible: true,
        modelValue: '',
        disabled: false,
        valid: true,
        validationMessage: '',
      },
    });

    const textarea = wrapper.find('textarea');
    await textarea.setValue('我去后山修炼');
    expect(wrapper.emitted('update:modelValue')?.[0]).toEqual(['我去后山修炼']);

    const submitButton = wrapper.findAll('button').find((btn) => btn.text() === '提交自由输入');
    expect(submitButton).toBeTruthy();
    await submitButton!.trigger('click');
    expect(wrapper.emitted('submit')).toBeTruthy();
  });

  it('hides panel when invisible', () => {
    const wrapper = mount(FreeTextInputPanel, {
      props: {
        visible: false,
        modelValue: '',
        disabled: false,
        valid: true,
        validationMessage: '',
      },
    });

    expect(wrapper.find('textarea').exists()).toBe(false);
  });
});
