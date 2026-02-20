import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import ChapterRecapCard from '../ChapterRecapCard.vue';

describe('ChapterRecapCard', () => {
  it('renders summary text when visible', () => {
    const wrapper = mount(ChapterRecapCard, {
      props: {
        visible: true,
        summary: '上一章中主角完成了第一次突破。',
      },
    });

    expect(wrapper.text()).toContain('上一章摘要');
    expect(wrapper.text()).toContain('主角完成了第一次突破');
  });

  it('does not render when hidden', () => {
    const wrapper = mount(ChapterRecapCard, {
      props: {
        visible: false,
        summary: 'hidden summary',
      },
    });

    expect(wrapper.text()).toBe('');
  });
});
