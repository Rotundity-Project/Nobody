import { flushPromises, mount } from '@vue/test-utils';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import LLMConfigDialog from '../LLMConfigDialog.vue';

const invokeWithTimeoutMock = vi.fn();

vi.mock('../../utils/tauriInvoke', () => ({
  invokeWithTimeout: (...args: unknown[]) => invokeWithTimeoutMock(...args),
}));

vi.mock('../../utils/audioSystem', () => ({
  playClick: vi.fn(),
}));

vi.mock('../../utils/llmProvider', () => ({
  getLlmProviderLabel: () => 'DeepSeek',
  resolveLlmProviderKey: () => 'deepseek',
}));

describe('LLMConfigDialog', () => {
  beforeEach(() => {
    invokeWithTimeoutMock.mockReset();
    window.localStorage.removeItem('nobody_llm_api_key');
  });

  it('loads saved status immediately when mounted open', async () => {
    invokeWithTimeoutMock.mockResolvedValue({
      configured: true,
      source: 'file',
      endpoint: 'https://api.siliconflow.cn/v1/chat/completions',
      model: 'deepseek-ai/DeepSeek-V3.2',
      max_tokens: 1024,
      temperature: 0.7,
      api_key_saved: true,
      api_key_hint: '***ykgh',
    });

    const wrapper = mount(LLMConfigDialog, {
      props: {
        isOpen: true,
        inline: true,
      },
    });

    await flushPromises();

    expect(invokeWithTimeoutMock).toHaveBeenCalledWith(
      'get_llm_config_status',
      undefined,
      8000,
      '读取配置状态超时，请稍后重试',
    );
    expect(wrapper.text()).toContain('当前状态：已配置（来源: file，模型: deepseek-ai/DeepSeek-V3.2）');
    expect(wrapper.text()).toContain('已检测到本地已保存的 API Key（***ykgh），留空保存会继续沿用。');
    expect(wrapper.find('input[type="password"]').attributes('placeholder')).toBe('已保存，可留空沿用');
  });
});
