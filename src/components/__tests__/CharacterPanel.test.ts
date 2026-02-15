import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import CharacterPanel from '../CharacterPanel.vue';
import { Element, Grade, type Character } from '../../types/game';

const buildCharacter = (): Character => ({
  id: 'player',
  name: 'Lin Mo',
  stats: {
    spiritual_root: {
      element: Element.Fire,
      grade: Grade.Heavenly,
      affinity: 85,
    },
    cultivation_realm: {
      name: '练气',
      level: 1,
      sub_level: 2,
      power_multiplier: 1,
    },
    techniques: ['Fire Palm'],
    lifespan: {
      current_age: 80,
      max_age: 100,
      realm_bonus: 0,
    },
    combat_power: 1234,
  },
  inventory: ['Spirit Stone'],
  location: 'sect_valley',
});

describe('CharacterPanel', () => {
  it('renders character details', () => {
    const wrapper = mount(CharacterPanel, {
      props: {
        character: buildCharacter(),
      },
    });

    expect(wrapper.text()).toContain('Lin Mo');
    expect(wrapper.text()).toContain('宗门外谷');
    expect(wrapper.text()).toContain('单灵根');
    expect(wrapper.find('.bg-amber-600').exists()).toBe(true);
  });

  it('shows empty state when character is null', () => {
    const wrapper = mount(CharacterPanel, {
      props: {
        character: null,
      },
    });

    expect(wrapper.find('.text-center').exists()).toBe(true);
    expect(wrapper.text()).not.toContain('Lin Mo');
  });
});
