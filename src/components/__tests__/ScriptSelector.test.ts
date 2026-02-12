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

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
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

const getScriptTypeCards = (wrapper: ReturnType<typeof mount>) =>
  wrapper.findAll('.space-y-4 > div');

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

    // existing_novel card
    await cards[2]!.trigger('click');
    await flushPromises();

    expect(invokeMock).toHaveBeenCalledWith('parse_novel_characters', {
      novelPath: 'C:\\novel.txt',
    });
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

    const startButton = wrapper.find('.mt-4 button');
    expect(startButton.exists()).toBe(true);
    await startButton.trigger('click');
    await flushPromises();

    expect(invokeMock).toHaveBeenCalledWith('load_existing_novel', {
      novelPath: 'C:\\novel.txt',
      selectedCharacter: 'Lin Mo',
    });
    expect(initializeGameMock).toHaveBeenCalled();
    expect(pushMock).toHaveBeenCalledWith('/game');
  });

  it('starts random script generation', async () => {
    initializeRandomGameMock.mockResolvedValue(undefined);

    const wrapper = mount(ScriptSelector);
    const cards = getScriptTypeCards(wrapper);
    expect(cards.length).toBeGreaterThanOrEqual(2);

    // random_generated card
    await cards[1]!.trigger('click');
    await flushPromises();

    expect(initializeRandomGameMock).toHaveBeenCalled();
    expect(pushMock).toHaveBeenCalledWith('/game');
  });
});
