import { describe, expect, it } from 'vitest';
import type { NoNameTrace } from '../types/game';
import {
  buildNoNameApplyLifecycleCheckpoints,
  buildNoNameApplyLifecycle,
  buildNoNameSafeOutputDrafts,
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

function buildReviewedDraftTrace(options: {
  humanReviewDecision?: 'pending' | 'approvedForHigherApply' | 'rejectedForHigherApply';
  secondGuardrail?: 'allow' | 'reject' | 'fallback' | null;
  manualApply?: boolean;
} = {}): NoNameTrace {
  const {
    humanReviewDecision = 'pending',
    secondGuardrail = null,
    manualApply = false,
  } = options;
  const requestId = 'controlled-output-proposal-draft-plot_augmentation_hint';
  const proposalId = 'proposal-draft';
  const proposalTransitionLog = [`${requestId}:apply_intent:awaiting_second_guardrail`];
  const applyExecutionLog = [{
    target: 'plotAugmentationHint',
    outcome: 'awaiting_second_guardrail',
  }];

  if (secondGuardrail) {
    proposalTransitionLog.push(`${requestId}:second_guardrail:${secondGuardrail}`);
    applyExecutionLog.push({
      target: 'plotAugmentationHint',
      outcome: secondGuardrail === 'allow'
        ? 'second_guardrail_allowed'
        : secondGuardrail === 'fallback'
          ? 'second_guardrail_fallback'
          : 'second_guardrail_rejected',
    });
  }
  if (manualApply) {
    proposalTransitionLog.push(`${requestId}:manual_apply:plot_augmentation_hint`);
    applyExecutionLog.push({
      target: 'plotAugmentationHint',
      outcome: 'manual_plot_augmentation_hint_applied',
    });
  }

  return {
    traceId: 'trace-safe-output-draft',
    sessionId: 'session-1',
    turnId: 'turn-1',
    mode: 'assisted',
    graphPath: ['CollectTurnInput', 'PlanTurn', 'ApplyProposal'],
    capabilityCalls: [],
    proposals: [{
      proposalId,
      kind: 'plotCandidate',
      producerRole: 'director',
      title: 'draft proposal',
      summary: 'stage a non-final safe output draft',
      focus: 'safe draft focus',
      targetSegment: 'current_turn_tail',
      intendedEffect: 'guide the next generation without mutating final state',
      rationale: 'test safe output draft',
      labels: ['director'],
      applyScopes: ['plotAugmentationHint'],
      status: 'applied',
      applyable: true,
    }],
    controlledOutputReviews: [{
      requestId,
      proposalId,
      requestedKind: 'nonFinalPlotAugmentation',
      decision: 'needsReview',
      reason: 'plot augmentation draft requires explicit review',
      normalizedKind: 'nonFinalPlotAugmentation',
      safeApplyScope: 'plotAugmentationHint',
      policyForbiddenScopes: ['finalPlotState', 'canonWorldFact'],
      requiresHumanReview: true,
      humanReviewDecision,
    }],
    proposalTransitionLog,
    applyPlanLog: [],
    applyExecutionLog,
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

  it('summarizes mixed second guardrail outcomes without contradictory manual state', () => {
    const trace: NoNameTrace = {
      ...buildTrace(
        'manual_plot_augmentation_hint_applied',
        'manual plot augmentation hint staged for focus=hidden cave clue',
      ),
      traceId: 'trace-mixed-second-guardrail',
      proposals: [{
        ...buildTrace(
          'manual_plot_augmentation_hint_applied',
          'manual plot augmentation hint staged for focus=hidden cave clue',
        ).proposals[0],
        proposalId: 'proposal-mixed',
        applyScopes: ['plotTextHint', 'optionBiasHint'],
      }],
      controlledOutputReviews: [{
        requestId: 'controlled-output-proposal-mixed-plot_text_hint',
        proposalId: 'proposal-mixed',
        requestedKind: 'sceneAugmentation',
        decision: 'needsReview',
        reason: 'plot text hint requires review',
        normalizedKind: 'sceneAugmentation',
        safeApplyScope: 'plotTextHint',
        policyForbiddenScopes: ['finalPlotState'],
        requiresHumanReview: true,
        humanReviewDecision: 'approvedForHigherApply',
      }, {
        requestId: 'controlled-output-proposal-mixed-option_bias_hint',
        proposalId: 'proposal-mixed',
        requestedKind: 'intermediateNarrativeHint',
        decision: 'needsReview',
        reason: 'option bias hint requires review',
        normalizedKind: 'intermediateNarrativeHint',
        safeApplyScope: 'optionBiasHint',
        policyForbiddenScopes: ['finalPlotState'],
        requiresHumanReview: true,
        humanReviewDecision: 'approvedForHigherApply',
      }],
      proposalTransitionLog: [
        'controlled-output-proposal-mixed-plot_text_hint:apply_intent:awaiting_second_guardrail',
        'controlled-output-proposal-mixed-option_bias_hint:apply_intent:awaiting_second_guardrail',
        'controlled-output-proposal-mixed-plot_text_hint:second_guardrail:allow',
        'controlled-output-proposal-mixed-option_bias_hint:second_guardrail:reject',
      ],
      applyExecutionLog: [{
        target: 'plot_text_hint',
        outcome: 'awaiting_second_guardrail',
      }, {
        target: 'option_bias_hint',
        outcome: 'awaiting_second_guardrail',
      }, {
        target: 'plot_text_hint',
        outcome: 'second_guardrail_allowed',
      }, {
        target: 'option_bias_hint',
        outcome: 'second_guardrail_rejected',
      }],
    };

    expect(buildNoNameApplyLifecycleCheckpoints(trace).map((checkpoint) => checkpoint.state)).toEqual([
      'approved',
      'mixed',
      'partially-ready',
    ]);
    expect(summarizeNoNameApplyLifecycleCheckpoints(trace)).toBe(
      '1.Human Review=approved -> 2.Second Guardrail=mixed -> 3.Manual Apply=partially-ready',
    );
    expect(buildNoNameApplyLifecycle(trace).find((step) => step.key === 'second-guardrail')?.state)
      .toBe('mixed');
    expect(buildNoNameApplyLifecycle(trace).find((step) => step.key === 'manual-plot-text')?.state)
      .toBe('partially-ready');
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

  it('builds safe output drafts without allowing final plot state writes', () => {
    const [draft] = buildNoNameSafeOutputDrafts(buildReviewedDraftTrace());

    expect(draft).toEqual(expect.objectContaining({
      draftId: 'safe-output-draft-controlled-output-proposal-draft-plot_augmentation_hint',
      sourceProposalId: 'proposal-draft',
      outputKind: 'nonFinalPlotAugmentation',
      safeApplyScope: 'plotAugmentationHint',
      lifecycleState: 'drafted',
    }));
    expect(draft?.evidence).toEqual(expect.objectContaining({
      reviewRequestId: 'controlled-output-proposal-draft-plot_augmentation_hint',
      humanReviewDecision: 'pending',
      secondGuardrailDecision: 'notEntered',
      manualApplyRecorded: false,
      finalPlotStateWriteAllowed: false,
    }));
    expect(draft?.evidence.reasons).toContain('draft probe never allows final plot state writes');
  });

  it('promotes safe output drafts only through explicit review and guardrail evidence', () => {
    expect(buildNoNameSafeOutputDrafts(buildReviewedDraftTrace({
      humanReviewDecision: 'approvedForHigherApply',
    }))[0]?.lifecycleState).toBe('reviewed');

    const [allowed] = buildNoNameSafeOutputDrafts(buildReviewedDraftTrace({
      humanReviewDecision: 'approvedForHigherApply',
      secondGuardrail: 'allow',
    }));
    expect(allowed?.lifecycleState).toBe('guardrailAllowed');
    expect(allowed?.evidence.secondGuardrailDecision).toBe('allow');
    expect(allowed?.evidence.finalPlotStateWriteAllowed).toBe(false);

    const [applied] = buildNoNameSafeOutputDrafts(buildReviewedDraftTrace({
      humanReviewDecision: 'approvedForHigherApply',
      secondGuardrail: 'allow',
      manualApply: true,
    }));
    expect(applied?.lifecycleState).toBe('manuallyApplied');
    expect(applied?.evidence.manualApplyRecorded).toBe(true);
    expect(applied?.evidence.finalPlotStateWriteAllowed).toBe(false);
  });

  it('keeps rejected and fallback safe output drafts out of manual apply', () => {
    const [rejectedByReview] = buildNoNameSafeOutputDrafts(buildReviewedDraftTrace({
      humanReviewDecision: 'rejectedForHigherApply',
    }));
    expect(rejectedByReview?.lifecycleState).toBe('blocked');
    expect(rejectedByReview?.evidence.manualApplyRecorded).toBe(false);

    const [rejectedByGuardrail] = buildNoNameSafeOutputDrafts(buildReviewedDraftTrace({
      humanReviewDecision: 'approvedForHigherApply',
      secondGuardrail: 'reject',
    }));
    expect(rejectedByGuardrail?.lifecycleState).toBe('blocked');
    expect(rejectedByGuardrail?.evidence.secondGuardrailDecision).toBe('reject');
    expect(rejectedByGuardrail?.evidence.finalPlotStateWriteAllowed).toBe(false);

    const [fallback] = buildNoNameSafeOutputDrafts(buildReviewedDraftTrace({
      humanReviewDecision: 'approvedForHigherApply',
      secondGuardrail: 'fallback',
    }));
    expect(fallback?.lifecycleState).toBe('fallback');
    expect(fallback?.evidence.manualApplyRecorded).toBe(false);
    expect(fallback?.evidence.finalPlotStateWriteAllowed).toBe(false);
  });
});
