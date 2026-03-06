import { mount } from '@vue/test-utils';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import ScriptSelector from '../ScriptSelector.vue';

const pushMock = vi.fn();
const openMock = vi.fn();
const invokeMock = vi.fn();

const initializeGameMock = vi.fn();

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
    invokeMock.mockResolvedValue(null);
    window.localStorage.removeItem('nobody_web_script_onboarding_seen_v1');
  });

  it('shows web onboarding once and quick-selects random script type', async () => {
    const wrapper = mount(ScriptSelector);
    await wrapper.vm.$nextTick();

    expect(wrapper.find('[data-testid="web-onboarding-banner"]').exists()).toBe(true);
    await wrapper.get('[data-testid="web-onboarding-random-btn"]').trigger('click');

    expect(wrapper.find('[data-testid="web-onboarding-banner"]').exists()).toBe(false);
    expect(wrapper.get('[data-testid="script-type-random_generated"]').classes())
      .toContain('script-type-card-active');
    expect(window.localStorage.getItem('nobody_web_script_onboarding_seen_v1')).toBe('1');
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
    await cards[2]!.trigger('click');
    await wrapper.get('[data-testid="confirm-script-btn"]').trigger('click');
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

  it('updates theme class when ui theme event is dispatched', async () => {
    window.localStorage.setItem('nobody_ui_theme', 'theme-scroll');
    const wrapper = mount(ScriptSelector);
    expect(wrapper.classes()).toContain('theme-scroll');

    window.dispatchEvent(new CustomEvent('nobody:ui-theme-changed', { detail: 'theme-night' }));
    await wrapper.vm.$nextTick();
    expect(wrapper.classes()).toContain('theme-night');
    expect(wrapper.find('[data-testid="ui-theme-status"]').exists()).toBe(true);
    expect(wrapper.text()).toContain('界面主题已切换');
    expect(wrapper.text()).toContain('当前为深色风格（已自动同步）');
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
    await wrapper.get('[data-testid="confirm-script-btn"]').trigger('click');
    await flushPromises();

    await wrapper.get('[data-testid="confirm-script-btn"]').trigger('click');
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
    await vi.waitFor(() => {
      expect(initializeGameMock).toHaveBeenCalled();
    });
    expect(pushMock).toHaveBeenCalledWith('/game');
  });

  it('starts random script generation after profile confirm', async () => {
    invokeMock.mockImplementation((command: string) => {
      if (command === 'generate_random_script') {
        return Promise.resolve({
          id: 'random_1',
          name: 'Random',
          script_type: 'random_generated',
          world_setting: {
            cultivation_realms: [],
            spiritual_roots: [],
            techniques: [],
            locations: [],
            factions: [],
          },
          initial_state: {
            player_name: '无名弟子',
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
    await cards[1]!.trigger('click');
    await wrapper.get('[data-testid="confirm-script-btn"]').trigger('click');
    await wrapper.find('#profile-player-name').setValue('测试主角');
    const createBtn = wrapper
      .findAll('button')
      .find((btn) => btn.text() === '确认创建');
    expect(createBtn).toBeTruthy();
    await createBtn!.trigger('click');
    await flushPromises();

    expect(invokeMock).toHaveBeenCalledWith(
      'generate_random_script',
      undefined,
      120000,
      '随机剧本生成超时，请稍后重试',
    );
    expect(initializeGameMock).toHaveBeenCalled();
    expect(pushMock).toHaveBeenCalledWith('/game');
  });

  it('shows random generation progress text', async () => {
    let resolveInit: (() => void) | undefined;
    invokeMock.mockImplementation((command: string) => {
      if (command === 'generate_random_script') {
        return new Promise((resolve) => {
          resolveInit = () => resolve({
            id: 'random_1',
            name: 'Random',
            script_type: 'random_generated',
            world_setting: {
              cultivation_realms: [],
              spiritual_roots: [],
              techniques: [],
              locations: [],
              factions: [],
            },
            initial_state: {
              player_name: '无名弟子',
              player_spiritual_root: { element: 'Fire', grade: 'Double', affinity: 0.5 },
              starting_location: 'origin',
              starting_age: 16,
            },
          });
        });
      }
      return Promise.resolve(null);
    });
    initializeGameMock.mockResolvedValue(undefined);

    const wrapper = mount(ScriptSelector);
    const cards = getScriptTypeCards(wrapper);
    await cards[1]!.trigger('click');
    await wrapper.get('[data-testid="confirm-script-btn"]').trigger('click');
    const createBtn = wrapper
      .findAll('button')
      .find((btn) => btn.text() === '确认创建');
    expect(createBtn).toBeTruthy();
    await createBtn!.trigger('click');
    await flushPromises();

    expect(wrapper.text()).toContain('正在生成随机剧本');
    expect(wrapper.text()).toContain('生成进度 1/2');
    expect(wrapper.text()).toContain('阴阳轮转');

    resolveInit?.();
    await flushPromises();
  });
});
