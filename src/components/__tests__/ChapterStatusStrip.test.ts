import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import ChapterStatusStrip from '../ChapterStatusStrip.vue';

describe('ChapterStatusStrip', () => {
  it('renders all status fields when visible', () => {
    const wrapper = mount(ChapterStatusStrip, {
      props: {
        visible: true,
        chapterProgress: '2 / 山雨欲来',
        chapterInteraction: '3 / 2-5',
        interactionState: '等待选项',
        optionSourceLabel: 'LLM-结构化',
        optionSourceHint: '受时延预算影响，已回退',
      },
    });

    expect(wrapper.text()).toContain('章节：2 / 山雨欲来');
    expect(wrapper.text()).toContain('交互：3 / 2-5');
    expect(wrapper.text()).toContain('状态：等待选项');
    expect(wrapper.text()).toContain('来源：LLM-结构化');
    expect(wrapper.text()).toContain('受时延预算影响，已回退');
  });

  it('hides source label and hint when empty', () => {
    const wrapper = mount(ChapterStatusStrip, {
      props: {
        visible: true,
        chapterProgress: '1 / 第一章',
        chapterInteraction: '1 / 2-4',
        interactionState: '自动推进',
        optionSourceLabel: '',
        optionSourceHint: '',
      },
    });

    expect(wrapper.text()).not.toContain('来源：');
    expect(wrapper.text()).not.toContain('时延预算');
  });
});
