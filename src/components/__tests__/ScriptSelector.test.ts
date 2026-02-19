import { mount } from '@vue/test-utils';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import ScriptSelector from '../ScriptSelector.vue';

const pushMock = vi.fn();
const openMock = vi.fn();
const invokeMock = vi.fn();

const initializeGameMock = vi.fn();
const initializeRandomGameMock = vi.fn();

vi.mock('vue-router', () => ({
  useRouter: () => ({
    push: pushMock,
  }),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: (...args: unknown[]) => openMock(...args),
}));

vi.mock('../../utils/tauriInvoke', () => ({
  invokeWithTimeout: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock('../../stores/gameStore', () => ({
  useGameStore: () => ({
    initializeGame: initializeGameMock,
    initializeRandomGame: initializeRandomGameMock,
  }),
}));

const flushPromises = async () => {
  await Promise.resolve();
  await Promise.resolve();
};

const getScriptTypeCards = (wrapper: ReturnType<typeof mount>) => ([
  wrapper.get('[data-testid="script-type-custom"]'),
  wrapper.get('[data-testid="script-type-random_generated"]'),
  wrapper.get('[data-testid="script-type-existing_novel"]'),
]);

describe('ScriptSelector', () => {
  beforeEach(() => {
    pushMock.mockReset();
    openMock.mockReset();
    invokeMock.mockReset();
    initializeGameMock.mockReset();
    initializeRandomGameMock.mockReset();
  });

  it('parses novel and shows character selection', async () => {
    openMock.mockResolvedValue('C:\\novel.txt');
    invokeMock.mockImplementation((command: string) => {
      if (command === 'parse_novel_characters') {
        return Promise.resolve(['Lin Mo', 'Su Wan']);
      }
      return Promise.resolve(null);
    });

    const wrapper = mount(ScriptSelector);
    const cards = getScriptTypeCards(wrapper);
    expect(cards.length).toBeGreaterThanOrEqual(3);

    await cards[2]!.trigger('click');
    await flushPromises();

    expect(invokeMock).toHaveBeenCalledWith(
      'parse_novel_characters',
      {
        novelPath: 'C:\\novel.txt',
      },
      60000,
      '解析小说超时，请检查文件或重试',
    );
    expect(wrapper.text()).toContain('Lin Mo');
    expect(wrapper.text()).toContain('Su Wan');
  });

  it('imports novel with selected character', async () => {
    openMock.mockResolvedValue('C:\\novel.txt');
    invokeMock.mockImplementation((command: string) => {
      if (command === 'parse_novel_characters') {
        return Promise.resolve(['Lin Mo', 'Su Wan']);
      }
      if (command === 'load_existing_novel') {
        return Promise.resolve({
          id: 'novel_1',
          name: 'Novel',
          script_type: 'existing_novel',
          world_setting: {
            cultivation_realms: [],
            spiritual_roots: [],
            techniques: [],
            locations: [],
            factions: [],
          },
          initial_state: {
            player_name: 'Lin Mo',
            player_spiritual_root: { element: 'Fire', grade: 'Double', affinity: 0.5 },
            starting_location: 'origin',
            starting_age: 16,
          },
        });
      }
      return Promise.resolve(null);
    });
    initializeGameMock.mockResolvedValue(undefined);

    const wrapper = mount(ScriptSelector);
    const cards = getScriptTypeCards(wrapper);
    await cards[2]!.trigger('click');
    await flushPromises();

    const radioButtons = wrapper.findAll('input[type="radio"]');
    expect(radioButtons.length).toBe(2);
    await radioButtons[0]!.setValue();

    const startButton = wrapper
      .findAll('button')
      .find((btn) => btn.text() === '开始导入');
    expect(startButton).toBeTruthy();
    await startButton!.trigger('click');
    await flushPromises();

    expect(invokeMock).toHaveBeenNthCalledWith(
      2,
      'load_existing_novel',
      {
        novelPath: 'C:\\novel.txt',
        selectedCharacter: 'Lin Mo',
      },
      90000,
      '导入小说超时，请重试',
    );
    expect(initializeGameMock).toHaveBeenCalled();
    expect(pushMock).toHaveBeenCalledWith('/game');
  });

  it('starts random script generation', async () => {
    initializeRandomGameMock.mockResolvedValue(undefined);

    const wrapper = mount(ScriptSelector);
    const cards = getScriptTypeCards(wrapper);
    expect(cards.length).toBeGreaterThanOrEqual(2);

    await cards[1]!.trigger('click');
    await flushPromises();

    expect(initializeRandomGameMock).toHaveBeenCalled();
    expect(pushMock).toHaveBeenCalledWith('/game');
  });

  it('shows random generation progress text without redundant dots', async () => {
    let resolveInit: (() => void) | undefined;
    initializeRandomGameMock.mockImplementation(
      () =>
        new Promise<void>((resolve) => {
          resolveInit = resolve;
        }),
    );

    const wrapper = mount(ScriptSelector);
    const cards = getScriptTypeCards(wrapper);
    await cards[1]!.trigger('click');
    await flushPromises();

    expect(wrapper.text()).toContain('正在生成随机剧本');
    expect(wrapper.text()).toContain('生成进度 1/2');
    expect(wrapper.text()).toContain('请稍候，正在处理请求');
    expect(wrapper.text()).not.toContain('请稍候，正在处理请求...');

    resolveInit?.();
    await flushPromises();
  });
});
