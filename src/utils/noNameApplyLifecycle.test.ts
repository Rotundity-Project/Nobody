import { describe, expect, it } from 'vitest';
import type { NoNameTrace } from '../types/game';
import {
  buildNoNameApplyLifecycleCheckpoints,
  buildNoNameApplyLifecycle,
  formatNoNameApplyExecutionRecord,
  summarizeNoNameApplyLifecycleCheckpoints,
  summarizeNoNameApplyExecutions,
  summarizeNoNameApplyLifecycle,
  summarizeNoNamePendingPlotAugmentation,
} from './noNameApplyLifecycle';

function buildTrace(outcome: string, note: string): NoNameTrace {
  return {
    traceId: 'trace-pending-augmentation',
    sessionId: 'session-1',
    turnId: 'turn-1',
    mode: 'assisted',
    graphPath: ['CollectTurnInput', 'PlanTurn', 'ApplyProposal'],
    capabilityCalls: [],
    proposals: [{
      proposalId: 'proposal-1',
      kind: 'plotCandidate',
      producerRole: 'director',
      title: 'plot augmentation proposal',
      summary: 'stage non-final plot augmentation',
      focus: 'hidden cave clue',
      targetSegment: 'current_turn_tail',
      intendedEffect: 'guide the next generation without mutating final state',
      rationale: 'test',
      labels: ['director'],
      applyScopes: ['plotAugmentationHint'],
      status: 'applied',
      applyable: true,
    }],
    proposalTransitionLog: [`proposal-1:pending_plot_augmentation:${outcome}`],
    applyPlanLog: [],
    applyExecutionLog: [{
      target: 'plot_augmentation_hint',
      outcome,
      note,
    }],
    guardrailResult: { outcome: 'accept' },
    applyResult: { attempted: true, outcome: 'applied_scoped_outputs' },
    fallbackUsed: false,
    elapsedMs: 3,
  };
}

describe('buildNoNameApplyLifecycle', () => {
  it('summarizes the human review, second guardrail, and manual apply order', () => {
    const trace: NoNameTrace = {
      ...buildTrace(
        'manual_plot_augmentation_hint_applied',
        'manual plot augmentation hint staged for focus=hidden cave clue',
      ),
      traceId: 'trace-manual-apply-order',
      proposals: [{
        ...buildTrace(
          'manual_plot_augmentation_hint_applied',
          'manual plot augmentation hint staged for focus=hidden cave clue',
        ).proposals[0],
        proposalId: 'proposal-manual-apply-order',
        applyScopes: ['plotTextHint'],
      }],
      controlledOutputReviews: [{
        requestId: 'review-plot-text',
        proposalId: 'proposal-manual-apply-order',
        requestedKind: 'sceneAugmentation',
        decision: 'needsReview',
        reason: 'plot text hint requires review',
        normalizedKind: 'sceneAugmentation',
        safeApplyScope: 'plotTextHint',
        policyForbiddenScopes: ['finalPlotState'],
        requiresHumanReview: true,
        humanReviewDecision: 'approvedForHigherApply',
      }],
      proposalTransitionLog: [
        'proposal-manual-apply-order:apply_intent:awaiting_second_guardrail',
        'proposal-manual-apply-order:second_guardrail:allow',
        'proposal-manual-apply-order:manual_apply:plot_text_hint',
      ],
      applyExecutionLog: [{
        target: 'plot_text_hint',
        outcome: 'awaiting_second_guardrail',
      }, {
        target: 'plot_text_hint',
        outcome: 'second_guardrail_allowed',
      }, {
        target: 'plot_text_hint',
        outcome: 'manual_plot_text_applied',
      }],
    };

    const checkpoints = buildNoNameApplyLifecycleCheckpoints(trace);

    expect(checkpoints.map((checkpoint) => checkpoint.key)).toEqual([
      'human-review',
      'second-guardrail',
      'manual-apply',
    ]);
    expect(checkpoints.map((checkpoint) => checkpoint.state)).toEqual([
      'approved',
      'allowed',
      'applied',
    ]);
    expect(summarizeNoNameApplyLifecycleCheckpoints(trace)).toBe(
      '1.Human Review=approved -> 2.Second Guardrail=allowed -> 3.Manual Apply=applied',
    );
  });

  it('keeps manual apply not ready while human review is pending', () => {
    const trace: NoNameTrace = {
      ...buildTrace(
        'manual_plot_augmentation_hint_applied',
        'manual plot augmentation hint staged for focus=hidden cave clue',
      ),
      controlledOutputReviews: [{
        requestId: 'review-pending',
        proposalId: 'proposal-1',
        requestedKind: 'sceneAugmentation',
        decision: 'needsReview',
        reason: 'plot text hint requires review',
        normalizedKind: 'sceneAugmentation',
        safeApplyScope: 'plotTextHint',
        policyForbiddenScopes: ['finalPlotState'],
        requiresHumanReview: true,
        humanReviewDecision: 'pending',
      }],
      proposalTransitionLog: ['proposal-1:apply_intent:awaiting_second_guardrail'],
      applyExecutionLog: [{
        target: 'plot_text_hint',
        outcome: 'awaiting_second_guardrail',
      }],
    };

    expect(buildNoNameApplyLifecycleCheckpoints(trace).map((checkpoint) => checkpoint.state)).toEqual([
      'pending',
      'waiting',
      'not-ready',
    ]);
  });

  it('does not treat ordinary pending hints as reviewed apply checkpoints', () => {
    const trace = buildTrace(
      'pending_plot_augmentation_retained',
      'pending plot augmentation retained because quick_mode; count=1',
    );

    expect(buildNoNameApplyLifecycleCheckpoints(trace).map((checkpoint) => checkpoint.state)).toEqual([
      'not-required',
      'not-required',
      'not-required',
    ]);
  });

  it('surfaces consumed pending plot augmentation hints', () => {
    const trace = buildTrace(
      'pending_plot_augmentation_consumed',
      'pending plot augmentation consumed after plot_engine generation; count=1',
    );

    const step = buildNoNameApplyLifecycle(trace)
      .find((item) => item.key === 'plot-augmentation-consumption');

    expect(step?.label).toBe('剧情增强消费');
    expect(step?.state).toBe('已消费');
    expect(step?.tone).toBe('done');
    expect(step?.detail).toContain('count=1');
    expect(summarizeNoNameApplyLifecycle(trace)).toContain('剧情增强消费:已消费');
    expect(summarizeNoNamePendingPlotAugmentation(trace)).toContain('已消费');
    expect(summarizeNoNamePendingPlotAugmentation(trace)).toContain('count=1');
  });

  it('surfaces retained pending plot augmentation hints', () => {
    const trace = buildTrace(
      'pending_plot_augmentation_retained',
      'pending plot augmentation retained because quick_mode; count=1',
    );

    const step = buildNoNameApplyLifecycle(trace)
      .find((item) => item.key === 'plot-augmentation-consumption');

    expect(step?.label).toBe('剧情增强消费');
    expect(step?.state).toBe('已保留');
    expect(step?.tone).toBe('pending');
    expect(step?.detail).toContain('quick_mode');
    expect(summarizeNoNamePendingPlotAugmentation(trace)).toContain('已保留');
  });

  it('summarizes staged pending plot augmentation hints', () => {
    const trace = buildTrace(
      'manual_plot_augmentation_hint_applied',
      'manual plot augmentation hint staged for focus=hidden cave clue',
    );

    expect(summarizeNoNamePendingPlotAugmentation(trace)).toContain('待消费');
    expect(summarizeNoNamePendingPlotAugmentation(trace)).toContain('hidden cave clue');
  });

  it('formats pending plot augmentation execution records for display', () => {
    expect(formatNoNameApplyExecutionRecord({
      target: 'plot_augmentation_hint',
      outcome: 'pending_plot_augmentation_consumed',
    })).toEqual({
      targetLabel: '剧情增强提示',
      outcomeLabel: '已消费',
    });
    expect(formatNoNameApplyExecutionRecord({
      target: 'plotAugmentationHint',
      outcome: 'pending_plot_augmentation_retained',
    })).toEqual({
      targetLabel: '剧情增强提示',
      outcomeLabel: '已保留',
    });
  });

  it('summarizes apply execution records with readable and raw labels', () => {
    const trace = buildTrace(
      'pending_plot_augmentation_consumed',
      'pending plot augmentation consumed after plot_engine generation; count=1',
    );

    expect(summarizeNoNameApplyExecutions(trace)).toContain(
      '剧情增强提示:已消费[raw=plot_augmentation_hint:pending_plot_augmentation_consumed]',
    );
    expect(summarizeNoNameApplyExecutions(trace, {
      emptyLabel: 'none',
      rawPrefix: ' [raw=',
      notePrefix: ' (',
    })).toContain(
      '剧情增强提示:已消费 [raw=plot_augmentation_hint:pending_plot_augmentation_consumed]',
    );
  });
});
