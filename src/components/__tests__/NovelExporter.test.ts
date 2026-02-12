import { mount } from '@vue/test-utils';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { nextTick } from 'vue';
import NovelExporter from '../NovelExporter.vue';

const invokeMock = vi.fn();
const saveMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  save: (...args: unknown[]) => saveMock(...args),
}));

vi.mock('../LoadingIndicator.vue', () => ({
  default: { name: 'LoadingIndicator', template: '<div />' },
}));

const flushPromises = async () => {
  await Promise.resolve();
  await Promise.resolve();
};

describe('NovelExporter', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    saveMock.mockReset();
  });

  it('generates novel and allows export', async () => {
    invokeMock.mockImplementation((command: string) => {
      if (command === 'generate_novel') {
        return Promise.resolve({
          title: '测试小说',
          chapters: [
            {
              index: 1,
              title: '第一章',
              content: '内容',
              source_event_ids: [1],
            },
          ],
          total_events: 1,
        });
      }
      if (command === 'export_novel') {
        return Promise.resolve(null);
      }
      return Promise.resolve(null);
    });

    saveMock.mockResolvedValue('C:\\novel.txt');

    const wrapper = mount(NovelExporter, {
      props: {
        isGameRunning: true,
        eventCount: 1,
      },
    });

    const generateButton = wrapper
      .findAll('button')
      .find((btn) => btn.text().includes('生成小说'));
    expect(generateButton).toBeTruthy();
    await generateButton!.trigger('click');
    await flushPromises();
    await nextTick();

    expect(invokeMock).toHaveBeenCalledWith('generate_novel', expect.any(Object));
    expect(wrapper.text()).toContain('测试小说');

    const exportButton = wrapper
      .findAll('button')
      .find((btn) => btn.text().includes('导出 TXT'));
    expect(exportButton).toBeTruthy();
    await exportButton!.trigger('click');
    await flushPromises();

    expect(saveMock).toHaveBeenCalled();
    expect(invokeMock).toHaveBeenCalledWith('export_novel', expect.any(Object));
  });
});
