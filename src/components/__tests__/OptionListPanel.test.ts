import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import OptionListPanel from '../OptionListPanel.vue';
import type { PlayerOption } from '../../types/game';

const options: PlayerOption[] = [
  { id: 1, description: '尝试突破', requirements: ['修为足够'], action: {} },
  { id: 2, description: '稳固境界', requirements: [], action: {} },
];

describe('OptionListPanel', () => {
  it('renders options and emits select event', async () => {
    const wrapper = mount(OptionListPanel, {
      props: {
        visible: true,
        options,
        disabled: false,
      },
    });

    expect(wrapper.text()).toContain('尝试突破');
    expect(wrapper.text()).toContain('条件：修为足够');

    const firstButton = wrapper.findAll('button')[0];
    await firstButton?.trigger('click');
    expect(wrapper.emitted('select')?.[0]).toEqual([options[0]]);
  });

  it('does not render when hidden', () => {
    const wrapper = mount(OptionListPanel, {
      props: {
        visible: false,
        options,
        disabled: false,
      },
    });
    expect(wrapper.findAll('button')).toHaveLength(0);
  });
});