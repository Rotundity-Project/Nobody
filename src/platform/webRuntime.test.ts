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

  it('skips NoName traces and diagnostics in disabled mode', async () => {
    const script = await invokeWebRuntime<Script>('generate_random_script');
    await invokeWebRuntime('initialize_game', { script });
    await invokeWebRuntime('initialize_plot');
    await invokeWebRuntime('set_noname_mode', { mode: 'disabled' });

    await invokeWebRuntime('execute_player_action', {
      action: {
        action_type: ActionType.FreeText,
        content: '观察山门灵气',
        selected_option_id: null,
      },
    });

    const plotState = await invokeWebRuntime<PlotState>('get_plot_state');
    const traces = await invokeWebRuntime<NoNameTrace[]>('get_noname_recent_traces');
    const latestParagraph = plotState.current_chapter.content[plotState.current_chapter.content.length - 1] ?? '';
    expect(traces).toHaveLength(0);
    expect(latestParagraph.includes('【NoName】重点关注')).toBe(false);
    expect(plotState.last_generation_diagnostics ?? '').not.toContain('NoName.');
  });

  it('records observe-only trace without applying NoName outputs', async () => {
    const script = await invokeWebRuntime<Script>('generate_random_script');
    await invokeWebRuntime('initialize_game', { script });
    await invokeWebRuntime('initialize_plot');

    await invokeWebRuntime('execute_player_action', {
      action: {
        action_type: ActionType.FreeText,
        content: '观察山门灵气',
        selected_option_id: null,
      },
    });

    const plotState = await invokeWebRuntime<PlotState>('get_plot_state');
    const traces = await invokeWebRuntime<NoNameTrace[]>('get_noname_recent_traces');
    const latestTrace = traces[traces.length - 1];
    const latestParagraph = plotState.current_chapter.content[plotState.current_chapter.content.length - 1] ?? '';
    expect(latestTrace?.mode).toBe('observeOnly');
    expect(latestTrace?.proposals[0]?.status).toBe('observed');
    expect(latestTrace?.proposals[0]?.applyable).toBe(false);
    expect(latestTrace?.applyResult?.outcome).toBe('skipped_observe_only');
    expect(latestTrace?.applyExecutionLog).toHaveLength(0);
    expect(latestParagraph.includes('【NoName】重点关注')).toBe(false);
    expect(plotState.current_chapter.summary).not.toContain('NoName提示');
    expect(plotState.last_generation_diagnostics).toContain('NoName.observeOnly');
  });

  it('queues NoName plot hint for review while applying low-risk hints in assisted mode', async () => {
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
    expect(latestParagraph.includes('【NoName】重点关注：观察灵脉回响')).toBe(false);
    expect(plotState.current_chapter.summary).toContain('NoName提示：后续重点关注观察灵脉回响');
    expect(plotState.last_generation_diagnostics).toContain('target_segment=current_turn_head');

    const traces = await invokeWebRuntime<NoNameTrace[]>('get_noname_recent_traces');
    const latestTrace = traces[traces.length - 1];
    expect(latestTrace?.applyPlanLog?.[0]?.order).toBe(1);
    expect(latestTrace?.proposals[0]?.targetSegment).toBe('current_turn_head');
    expect(latestTrace?.proposals[0]?.applyScopes).toContain('plotTextHint');
    expect(latestTrace?.applyPlanLog?.[0]?.decision).toBe('needs_review');
    expect(latestTrace?.applyPlanLog?.[0]?.priority).toBe(300);
    expect(latestTrace?.controlledOutputReviews?.some((item) => (
      item.safeApplyScope === 'plotTextHint' && item.decision === 'needsReview'
    ))).toBe(true);
    const plotTextReview = latestTrace?.controlledOutputReviews?.find((item) => item.safeApplyScope === 'plotTextHint');
    expect(plotTextReview?.proposalId).toBe(latestTrace?.proposals[0]?.proposalId);
    expect(plotTextReview?.policyForbiddenScopes).toEqual(expect.arrayContaining([
      'finalPlotState',
      'canonWorldFact',
      'characterStats',
      'inventoryOrResource',
      'mapTopology',
      'chapterLifecycle',
      'playerChoice',
      'combatOutcome',
    ]));
    expect(latestTrace?.applyResult?.outcome).toBe('applied_summary_and_option_bias');
    expect(latestTrace?.applyResult?.reason).toContain('plotTextHint 等待人工复核');
    expect(latestTrace?.applyResult?.reason).toContain('target=current_turn_head');
  });

  it('keeps Web NoName mock aligned with backend review and context fields', async () => {
    const script = await invokeWebRuntime<Script>('generate_random_script');
    await invokeWebRuntime('initialize_game', { script });
    await invokeWebRuntime('initialize_plot');
    await invokeWebRuntime('set_noname_mode', { mode: 'assisted' });

    await invokeWebRuntime('execute_player_action', {
      action: {
        action_type: ActionType.FreeText,
        content: '校准护山阵纹',
        selected_option_id: null,
      },
    });

    const traces = await invokeWebRuntime<NoNameTrace[]>('get_noname_recent_traces');
    const latestTrace = traces[traces.length - 1];
    const reviews = latestTrace.controlledOutputReviews ?? [];
    const relatedObservations = latestTrace.relatedObservations ?? [];
    const plotTextReview = reviews.find((item) => item.safeApplyScope === 'plotTextHint');
    const plotAugmentationReview = reviews.find((item) => item.safeApplyScope === 'plotAugmentationHint');
    const diagnosticsReview = reviews.find((item) => item.safeApplyScope === 'diagnostics');
    const worldContext = relatedObservations.find((item) => item.role === 'worldCurator');

    expect(reviews).toHaveLength(5);
    expect(plotTextReview).toEqual(expect.objectContaining({
      requestedKind: 'sceneAugmentation',
      decision: 'needsReview',
      normalizedKind: 'sceneAugmentation',
      requiresHumanReview: true,
      humanReviewDecision: null,
    }));
    expect(plotTextReview?.proposalId).toBe(latestTrace.proposals[0]?.proposalId);
    expect(plotTextReview?.policyForbiddenScopes).toEqual(expect.arrayContaining([
      'finalPlotState',
      'canonWorldFact',
      'chapterLifecycle',
      'playerChoice',
    ]));
    expect(plotAugmentationReview).toEqual(expect.objectContaining({
      requestedKind: 'nonFinalPlotAugmentation',
      decision: 'needsReview',
      requiresHumanReview: true,
    }));
    expect(diagnosticsReview).toEqual(expect.objectContaining({
      requestedKind: 'narrativeNote',
      decision: 'allow',
      requiresHumanReview: false,
    }));

    expect(worldContext).toBeTruthy();
    expect(worldContext?.actionSummary).toBe('校准护山阵纹');
    expect(worldContext?.roleGoal).toContain('Maintain world facts');
    expect(worldContext?.forbiddenScopes).toEqual(expect.arrayContaining([
      'Must not decide NPC private intent.',
      'Must not write final plot state.',
    ]));
    expect(worldContext?.noteTypeHits).toContain('goal:web-mock-note-context');
    expect(worldContext?.sourceStats).toEqual(expect.arrayContaining([
      { source: 'semantic', count: 1 },
      { source: 'noteContext', count: 1 },
    ]));
    expect(worldContext?.contextSliceStats).toEqual(expect.arrayContaining([
      { section: 'narrativeNotes', sourceCount: 1, visibleCount: 1 },
    ]));
    expect(worldContext?.proposal.producerRole).toBe('worldCurator');
    expect(worldContext?.proposal.applyable).toBe(false);
  });

  it('records human review apply intent without mutating plot text in web mode', async () => {
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

    const beforePlotState = await invokeWebRuntime<PlotState>('get_plot_state');
    const beforeContent = [...beforePlotState.current_chapter.content];
    const segmentIndex = beforeContent.length - 1;
    const expectedSegmentText = beforeContent[segmentIndex];
    const traces = await invokeWebRuntime<NoNameTrace[]>('get_noname_recent_traces');
    const latestTrace = traces[traces.length - 1];
    const review = latestTrace.controlledOutputReviews?.find((item) => (
      item.safeApplyScope === 'plotTextHint' && item.requiresHumanReview
    ));
    expect(review?.decision).toBe('needsReview');
    expect(latestTrace.controlledOutputReviews?.some((item) => (
      item.safeApplyScope === 'plotAugmentationHint'
      && item.requestedKind === 'nonFinalPlotAugmentation'
      && item.requiresHumanReview
    ))).toBe(true);

    const updatedTrace = await invokeWebRuntime<NoNameTrace>('mark_noname_controlled_output_review', {
      traceId: latestTrace.traceId,
      requestId: review?.requestId,
      decision: 'approvedForHigherApply',
    });

    const afterPlotState = await invokeWebRuntime<PlotState>('get_plot_state');
    expect(afterPlotState.current_chapter.content).toEqual(beforeContent);
    expect(updatedTrace.controlledOutputReviews?.find((item) => (
      item.safeApplyScope === 'plotTextHint'
    ))?.humanReviewDecision).toBe('approvedForHigherApply');
    expect(updatedTrace.applyPlanLog?.some((item) => (
      item.target === 'plotTextHint' && item.decision === 'review_intent_ready'
    ))).toBe(true);
    expect(updatedTrace.applyExecutionLog?.some((item) => (
      item.target === 'plotTextHint' && item.outcome === 'awaiting_second_guardrail'
    ))).toBe(true);

    const resolvedTrace = await invokeWebRuntime<NoNameTrace>('resolve_noname_second_guardrail', {
      traceId: latestTrace.traceId,
      requestId: review?.requestId,
      decision: 'allow',
    });
    const finalPlotState = await invokeWebRuntime<PlotState>('get_plot_state');
    expect(finalPlotState.current_chapter.content).toEqual(beforeContent);
    expect(resolvedTrace.applyPlanLog?.some((item) => (
      item.target === 'plotTextHint' && item.decision === 'second_guardrail_allow'
    ))).toBe(true);
    expect(resolvedTrace.applyExecutionLog?.some((item) => (
      item.target === 'plotTextHint' && item.outcome === 'second_guardrail_allowed'
    ))).toBe(true);
    expect(resolvedTrace.applyResult?.outcome).toBe('second_guardrail_allow');

    const applied = await invokeWebRuntime<{ trace: NoNameTrace; plotState: PlotState }>('apply_noname_manual_plot_text_hint', {
      traceId: latestTrace.traceId,
      requestId: review?.requestId,
      chapterIndex: beforePlotState.current_chapter.index,
      segmentIndex,
      expectedSegmentText,
    });
    expect(applied.plotState.current_chapter.content[segmentIndex]).toContain('【NoName】重点关注：观察灵脉回响');
    expect(applied.trace.applyExecutionLog?.some((item) => (
      item.target === 'plotTextHint' && item.outcome === 'manual_plot_text_applied'
    ))).toBe(true);
  });

  it('consumes reviewed plot augmentation hints on the next assisted generation', async () => {
    const script = await invokeWebRuntime<Script>('generate_random_script');
    await invokeWebRuntime('initialize_game', { script });
    await invokeWebRuntime('initialize_plot');
    await invokeWebRuntime('set_noname_mode', { mode: 'assisted' });

    await invokeWebRuntime('execute_player_action', {
      action: {
        action_type: ActionType.FreeText,
        content: '追踪洞府残响',
        selected_option_id: null,
      },
    });

    const traces = await invokeWebRuntime<NoNameTrace[]>('get_noname_recent_traces');
    const firstTrace = traces[traces.length - 1];
    const review = firstTrace.controlledOutputReviews?.find((item) => (
      item.safeApplyScope === 'plotAugmentationHint' && item.requiresHumanReview
    ));
    expect(review?.decision).toBe('needsReview');

    await invokeWebRuntime<NoNameTrace>('mark_noname_controlled_output_review', {
      traceId: firstTrace.traceId,
      requestId: review?.requestId,
      decision: 'approvedForHigherApply',
    });
    await invokeWebRuntime<NoNameTrace>('resolve_noname_second_guardrail', {
      traceId: firstTrace.traceId,
      requestId: review?.requestId,
      decision: 'allow',
    });

    const beforeApply = await invokeWebRuntime<PlotState>('get_plot_state');
    const applied = await invokeWebRuntime<{ trace: NoNameTrace; plotState: PlotState }>('apply_noname_reviewed_output', {
      traceId: firstTrace.traceId,
      requestId: review?.requestId,
      scope: 'plotAugmentationHint',
      chapterIndex: beforeApply.current_chapter.index,
      expectedPlotAugmentationHints: beforeApply.pending_plot_augmentation_hints ?? [],
    });
    expect(applied.plotState.pending_plot_augmentation_hints).toHaveLength(1);

    await invokeWebRuntime('execute_player_action', {
      action: {
        action_type: ActionType.FreeText,
        content: '沿着残响前进',
        selected_option_id: null,
      },
      quickMode: false,
    });

    const afterConsume = await invokeWebRuntime<PlotState>('get_plot_state');
    const afterTraces = await invokeWebRuntime<NoNameTrace[]>('get_noname_recent_traces');
    const latestTrace = afterTraces[afterTraces.length - 1];

    expect(afterConsume.pending_plot_augmentation_hints).toHaveLength(0);
    expect(afterConsume.last_generation_diagnostics).toContain('NoName pending plot augmentation consumed=1');
    expect(latestTrace.applyExecutionLog?.some((item) => (
      item.target === 'plotAugmentationHint'
      && item.outcome === 'pending_plot_augmentation_consumed'
      && item.note?.includes('count=1')
    ))).toBe(true);
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
