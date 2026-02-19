import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import CharacterInfoModal from '../CharacterInfoModal.vue';

describe('CharacterInfoModal', () => {
  it('does not render when closed', () => {
    const wrapper = mount(CharacterInfoModal, {
      props: {
        isOpen: false,
        character: null,
      },
      global: {
        stubs: {
          CharacterPanel: true,
        },
      },
    });

    expect(wrapper.find('.fixed').exists()).toBe(false);
  });

  it('emits close on overlay click and button click', async () => {
    const wrapper = mount(CharacterInfoModal, {
      props: {
        isOpen: true,
        character: null,
      },
      global: {
        stubs: {
          CharacterPanel: true,
        },
      },
    });

    await wrapper.find('.fixed').trigger('click');
    await wrapper.find('button').trigger('click');
    expect(wrapper.emitted('close')?.length).toBe(2);
  });
});
