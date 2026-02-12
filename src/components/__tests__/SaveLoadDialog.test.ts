import { mount } from '@vue/test-utils';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { nextTick } from 'vue';
import SaveLoadDialog from '../SaveLoadDialog.vue';
import LoadingIndicator from '../LoadingIndicator.vue';

const listSaveSlotsMock = vi.fn();

vi.mock('../../stores/gameStore', () => ({
  useGameStore: () => ({
    listSaveSlots: listSaveSlotsMock,
    saveGame: vi.fn(),
    loadGame: vi.fn(),
  }),
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
