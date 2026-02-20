import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import StoryScenePanel from '../StoryScenePanel.vue';

describe('StoryScenePanel', () => {
  it('renders story content and rhythm info when scene exists', () => {
    const wrapper = mount(StoryScenePanel, {
      props: {
        hasScene: true,
        chapterTitle: '第二章',
        showRecap: true,
        recapSummary: '上一章完成了突破。',
        paragraphs: ['第一段', '第二段'],
        optionSourceLabel: 'LLM-结构化',
        isGameInitialized: true,
        scrollElement: null,
      },
      global: {
        stubs: {
          VirtualStoryList: { template: '<div>VirtualStoryList</div>' },
          ChapterRecapCard: { template: '<div>ChapterRecapCard</div>' },
        },
      },
    });

    expect(wrapper.text()).toContain('第二章');
    expect(wrapper.text()).toContain('本章 2 段');
    expect(wrapper.text()).toContain('选项来源：LLM-结构化');
    expect(wrapper.get('[data-testid="rhythm-badge"]').text()).toContain('节奏：紧凑');
  });

  it('renders empty state when game is not initialized', () => {
    const wrapper = mount(StoryScenePanel, {
      props: {
        hasScene: false,
        chapterTitle: '',
        showRecap: false,
        recapSummary: '',
        paragraphs: [],
        optionSourceLabel: '',
        isGameInitialized: false,
        scrollElement: null,
      },
      global: {
        stubs: {
          VirtualStoryList: { template: '<div />' },
          ChapterRecapCard: { template: '<div />' },
        },
      },
    });

    expect(wrapper.text()).toContain('当前没有进行中的游戏');
  });
});
