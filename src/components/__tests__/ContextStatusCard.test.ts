import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import ContextStatusCard from '../ContextStatusCard.vue';

describe('ContextStatusCard', () => {
  it('formats fallback location label in Chinese', () => {
    const wrapper = mount(ContextStatusCard, {
      props: {
        visible: true,
        playerName: '无名弟子',
        playerRealm: '炼气 (1-2)',
        chapterProgress: '1 / 第一章',
        chapterInteraction: '1 / 2-4',
        locationLabel: 'sect_valley',
        interactionStateLabel: '等待选项',
      },
    });

    expect(wrapper.text()).toContain('宗门外谷');
    expect(wrapper.text()).not.toContain('sect_valley');
  });
});
