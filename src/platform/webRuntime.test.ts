import { describe, expect, it } from 'vitest';
import { ActionType, type Script } from '../types/game';
import { invokeWebRuntime } from './webRuntime';

describe('webRuntime', () => {
  it('supports initialize -> action -> save/load core flow', async () => {
    const script = await invokeWebRuntime<Script>('generate_random_script');
    const gameState = await invokeWebRuntime('initialize_game', { script });
    expect(gameState).toBeTruthy();

    const plotState = await invokeWebRuntime('initialize_plot');
    expect(plotState).toBeTruthy();

    await invokeWebRuntime('execute_player_action', {
      action: {
        action_type: ActionType.FreeText,
        content: '尝试探索附近的灵脉',
        selected_option_id: null,
      },
    });

    const afterAction = await invokeWebRuntime('get_plot_state');
    expect((afterAction as { current_chapter: { content: string[] } }).current_chapter.content.length).toBeGreaterThan(1);

    await invokeWebRuntime('save_game', { slotId: 1 });
    const slots = await invokeWebRuntime('list_save_slots');
    expect((slots as Array<unknown>).length).toBeGreaterThan(0);

    const loaded = await invokeWebRuntime('load_game', { slotId: 1 });
    expect(loaded).toBeTruthy();
  });
});


