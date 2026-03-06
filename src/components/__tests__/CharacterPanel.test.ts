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
      affinity: 0.85,
    },
    cultivation_realm: {
      name: 'Qi Condensation',
      level: 1,
      sub_level: 2,
      power_multiplier: 1,
    },
    techniques: ['Fire Palm', '青霜剑诀'],
    lifespan: {
      current_age: 80,
      max_age: 100,
      realm_bonus: 0,
    },
    combat_power: 1234,
  },
  inventory: ['Spirit Stone'],
  location: 'sect_valley',
  growth_log: ['境界突破 +1', '功法熟练度提升'],
  personality_tags: ['谨慎', '果断'],
  combat_status: {
    injury_level: 2,
    reputation: 4,
    enmity: 1,
    qi_deviation: 0,
  },
  social_profile: {
    sect_affinity: 7,
    mentor_bond: 6,
    vendetta: 2,
    favor: 5,
    camp_stance: 'neutral',
  },
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
    expect(wrapper.text()).toContain('战后状态');
    expect(wrapper.text()).toContain('关系画像');
    expect(wrapper.find('.grade-badge-heavenly').exists()).toBe(true);
  });

  it('shows empty state when character is null', () => {
    const wrapper = mount(CharacterPanel, {
      props: {
        character: null,
      },
    });

    expect(wrapper.find('.text-center').exists()).toBe(true);
    expect(wrapper.text()).toContain('暂无角色数据');
    expect(wrapper.text()).not.toContain('Lin Mo');
  });
});
