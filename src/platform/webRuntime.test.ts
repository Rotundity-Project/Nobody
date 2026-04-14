import { beforeEach, describe, expect, it } from 'vitest';
import { ActionType, type NoNameTrace, type PlotState, type Script } from '../types/game';
import { invokeWebRuntime } from './webRuntime';

describe('webRuntime', () => {
  beforeEach(async () => {
    if (typeof localStorage !== 'undefined') {
      localStorage.clear();
    }
    await invokeWebRuntime('clear_noname_recent_traces');
    await invokeWebRuntime('set_noname_mode', { mode: 'observeOnly' });
  });

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

  it('applies NoName plot hint to current turn head in assisted mode for free text actions', async () => {
    const script = await invokeWebRuntime<Script>('generate_random_script');
    await invokeWebRuntime('initialize_game', { script });
    await invokeWebRuntime('initialize_plot');
    await invokeWebRuntime('set_noname_mode', { mode: 'assisted' });

    await invokeWebRuntime('execute_player_action', {
      action: {
        action_type: ActionType.FreeText,
        content: '观察灵脉回响',
        selected_option_id: null,
      },
    });

    const plotState = await invokeWebRuntime<PlotState>('get_plot_state');
    const latestParagraph = plotState.current_chapter.content[plotState.current_chapter.content.length - 1] ?? '';
    expect(latestParagraph.startsWith('【NoName】重点关注：观察灵脉回响')).toBe(true);
    expect(plotState.current_chapter.summary).toContain('NoName提示：后续重点关注观察灵脉回响');
    expect(plotState.last_generation_diagnostics).toContain('target_segment=current_turn_head');

    const traces = await invokeWebRuntime<NoNameTrace[]>('get_noname_recent_traces');
    const latestTrace = traces[traces.length - 1];
    expect(latestTrace?.applyPlanLog?.[0]?.order).toBe(1);
    expect(latestTrace?.proposals[0]?.targetSegment).toBe('current_turn_head');
    expect(latestTrace?.proposals[0]?.applyScopes).toContain('plotTextHint');
    expect(latestTrace?.applyPlanLog?.[0]?.decision).toBe('apply');
    expect(latestTrace?.applyPlanLog?.[0]?.priority).toBe(300);
    expect(latestTrace?.applyResult?.reason).toContain('target=current_turn_head');
  });

  it('applies NoName summary hint to chapter summary head for even option actions in assisted mode', async () => {
    const script = await invokeWebRuntime<Script>('generate_random_script');
    await invokeWebRuntime('initialize_game', { script });
    await invokeWebRuntime('initialize_plot');
    await invokeWebRuntime('set_noname_mode', { mode: 'assisted' });

    await invokeWebRuntime('execute_player_action', {
      action: {
        action_type: ActionType.SelectedOption,
        content: '',
        selected_option_id: 0,
      },
    });

    const plotState = await invokeWebRuntime<PlotState>('get_plot_state');
    const latestParagraph = plotState.current_chapter.content[plotState.current_chapter.content.length - 1] ?? '';
    expect(latestParagraph.includes('【NoName】重点关注')).toBe(false);
    expect(plotState.current_chapter.summary.startsWith('NoName提示：后续重点关注在原地打坐，稳固气息')).toBe(true);
    expect(plotState.last_generation_diagnostics).toContain('target_segment=chapter_summary_head');

    const traces = await invokeWebRuntime<NoNameTrace[]>('get_noname_recent_traces');
    const latestTrace = traces[traces.length - 1];
    expect(latestTrace?.proposals[0]?.targetSegment).toBe('chapter_summary_head');
    expect(latestTrace?.proposals[0]?.applyScopes).not.toContain('plotTextHint');
    expect(latestTrace?.applyPlanLog?.some((item: { target: string }) => item.target === 'chapterSummaryHint')).toBe(true);
    expect(latestTrace?.applyResult?.reason).toContain('target=chapter_summary_head');
  });
});
