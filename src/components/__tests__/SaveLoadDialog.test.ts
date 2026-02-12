import { mount } from '@vue/test-utils';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { nextTick } from 'vue';
import SaveLoadDialog from '../SaveLoadDialog.vue';
import LoadingIndicator from '../LoadingIndicator.vue';

const listSaveSlotsMock = vi.fn();
const saveGameMock = vi.fn();
const loadGameMock = vi.fn();
const playClickMock = vi.fn();

vi.mock('../../stores/gameStore', () => ({
  useGameStore: () => ({
    listSaveSlots: listSaveSlotsMock,
    saveGame: saveGameMock,
    loadGame: loadGameMock,
  }),
}));

vi.mock('../../utils/audioSystem', () => ({
  playClick: () => playClickMock(),
}));

const flushPromises = async () => {
  await Promise.resolve();
  await Promise.resolve();
};

const createDeferred = <T>() => {
  let resolve: (value: T) => void = () => undefined;
  let reject: (reason?: unknown) => void = () => undefined;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
};

describe('SaveLoadDialog loading states', () => {
  beforeEach(() => {
    listSaveSlotsMock.mockReset();
    saveGameMock.mockReset();
    loadGameMock.mockReset();
    playClickMock.mockReset();
  });

  it('shows and clears loading indicator when fetching slots', async () => {
    const deferred = createDeferred<
      {
        slot_id: number;
        version: string;
        timestamp: number;
        player_name: string;
        player_age: number;
        realm: string;
        location: string;
        game_time: string;
      }[]
    >();
    listSaveSlotsMock.mockReturnValue(deferred.promise);

    const wrapper = mount(SaveLoadDialog, {
      props: {
        isOpen: false,
        mode: 'save',
      },
    });

    await wrapper.setProps({ isOpen: true });
    await nextTick();
    expect(wrapper.findComponent(LoadingIndicator).exists()).toBe(true);
    expect(wrapper.text()).toContain('正在读取存档列表');

    deferred.resolve([]);
    await flushPromises();
    await nextTick();

    expect(wrapper.findComponent(LoadingIndicator).exists()).toBe(false);
  });
});

describe('SaveLoadDialog actions', () => {
  beforeEach(() => {
    listSaveSlotsMock.mockReset();
    saveGameMock.mockReset();
    loadGameMock.mockReset();
    playClickMock.mockReset();
  });

  it('saves to selected slot in save mode', async () => {
    listSaveSlotsMock.mockResolvedValue([]);
    saveGameMock.mockResolvedValue(undefined);

    const wrapper = mount(SaveLoadDialog, {
      props: {
        isOpen: false,
        mode: 'save',
      },
    });

    await wrapper.setProps({ isOpen: true });
    await flushPromises();
    const slots = wrapper.findAll('.border-2');
    expect(slots.length).toBeGreaterThan(0);
    await slots[0]!.trigger('click');

    const confirmButton = wrapper
      .findAll('button')
      .find((btn) => btn.classes().includes('flex-1'));
    expect(confirmButton).toBeTruthy();
    await confirmButton!.trigger('click');
    await flushPromises();

    expect(saveGameMock).toHaveBeenCalledWith(1);
    expect(wrapper.emitted('saved')).toBeTruthy();
  });

  it('loads from selected slot in load mode', async () => {
    listSaveSlotsMock.mockResolvedValue([
      {
        slot_id: 1,
        version: '1',
        timestamp: 1,
        player_name: 'Lin',
        player_age: 16,
        realm: '练气',
        location: 'sect',
        game_time: '1-1-1',
      },
    ]);
    loadGameMock.mockResolvedValue(undefined);

    const wrapper = mount(SaveLoadDialog, {
      props: {
        isOpen: false,
        mode: 'load',
      },
    });

    await wrapper.setProps({ isOpen: true });
    await flushPromises();
    const slots = wrapper.findAll('.border-2');
    expect(slots.length).toBeGreaterThan(0);
    await slots[0]!.trigger('click');

    const confirmButton = wrapper
      .findAll('button')
      .find((btn) => btn.classes().includes('flex-1'));
    expect(confirmButton).toBeTruthy();
    await confirmButton!.trigger('click');
    await flushPromises();

    expect(loadGameMock).toHaveBeenCalledWith(1);
    expect(wrapper.emitted('loaded')).toBeTruthy();
  });
});
