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

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock('../../stores/gameStore', () => ({
  useGameStore: () => ({
    initializeGame: initializeGameMock,
    initializeRandomGame: vi.fn(),
  }),
}));

const flushPromises = async () => {
  await Promise.resolve();
  await Promise.resolve();
};

describe('ScriptSelector', () => {
  beforeEach(() => {
    pushMock.mockReset();
    openMock.mockReset();
    invokeMock.mockReset();
    initializeGameMock.mockReset();
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
    const target = wrapper
      .findAll('h3')
      .find((node) => node.text() === '现有小说');
    expect(target).toBeTruthy();

    await target!.trigger('click');
    await flushPromises();

    expect(invokeMock).toHaveBeenCalledWith('parse_novel_characters', {
      novelPath: 'C:\\novel.txt',
    });
    expect(wrapper.text()).toContain('选择主角');
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
    const target = wrapper
      .findAll('h3')
      .find((node) => node.text() === '现有小说');
    await target!.trigger('click');
    await flushPromises();

    const radioButtons = wrapper.findAll('input[type="radio"]');
    expect(radioButtons.length).toBe(2);
    await radioButtons[0]!.setValue();

    const startButton = wrapper
      .findAll('button')
      .find((node) => node.text() === '开始导入');
    expect(startButton).toBeTruthy();
    await startButton!.trigger('click');
    await flushPromises();

    expect(invokeMock).toHaveBeenCalledWith('load_existing_novel', {
      novelPath: 'C:\\novel.txt',
      selectedCharacter: 'Lin Mo',
    });
    expect(initializeGameMock).toHaveBeenCalled();
    expect(pushMock).toHaveBeenCalledWith('/game');
  });
});
