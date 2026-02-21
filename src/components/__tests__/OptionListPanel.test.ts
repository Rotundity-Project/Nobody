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

  it('infers risk/probe tags from semantic text', () => {
    const wrapper = mount(OptionListPanel, {
      props: {
        visible: true,
        disabled: false,
        options: [
          { id: 1, description: '冒险突破当前境界', requirements: [], action: {} },
          { id: 2, description: '先观察四周再行动', requirements: [], action: {} },
        ],
      },
    });

    const text = wrapper.text();
    expect(text).toContain('风险');
    expect(text).toContain('探查');
  });

  it('prefers backend tag fields when present', () => {
    const taggedOptions = [
      { id: 1, description: '稳步前行', requirements: [], action: {}, risk_tier: 'high' },
      { id: 2, description: '谨慎前行', requirements: [], action: {}, tag: 'probe' },
    ] as unknown as PlayerOption[];

    const wrapper = mount(OptionListPanel, {
      props: {
        visible: true,
        disabled: false,
        options: taggedOptions,
      },
    });

    const badges = wrapper.findAll('span').map((node) => node.text());
    expect(badges).toContain('风险');
    expect(badges).toContain('探查');
  });
});
