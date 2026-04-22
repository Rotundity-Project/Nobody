import type {
  NoNameApplyExecutionRecord,
  NoNameHumanReviewDecision,
  NoNameTrace,
} from '../types/game';

export type NoNameApplyLifecycleTone = 'done' | 'pending' | 'blocked' | 'fallback' | 'info';

export interface NoNameApplyLifecycleStep {
  key: string;
  label: string;
  state: string;
  detail: string;
  tone: NoNameApplyLifecycleTone;
}

export type NoNameApplyLifecycleCheckpointKey =
  | 'human-review'
  | 'second-guardrail'
  | 'manual-apply';

export interface NoNameApplyLifecycleCheckpoint {
  key: NoNameApplyLifecycleCheckpointKey;
  label: string;
  order: number;
  state: string;
  detail: string;
  tone: NoNameApplyLifecycleTone;
}

export interface NoNameApplyExecutionDisplay {
  targetLabel: string;
  outcomeLabel: string;
}

interface NoNameSecondGuardrailStats {
  allowed: number;
  rejected: number;
  fallback: number;
  awaiting: number;
  manualApplied: number;
}

function normalizeTarget(value: string | undefined) {
  return (value ?? '').replace(/_/g, '').toLowerCase();
}

function hasExecution(trace: NoNameTrace, target: string, outcome: string) {
  const normalizedTarget = normalizeTarget(target);
  return Boolean(trace.applyExecutionLog?.some((item) => (
    normalizeTarget(item.target) === normalizedTarget
    && item.outcome === outcome
  )));
}

function latestExecutionForTarget(
  trace: NoNameTrace,
  target: string,
  outcomes: string[],
): NoNameApplyExecutionRecord | null {
  const normalizedTarget = normalizeTarget(target);
  return trace.applyExecutionLog
    ?.slice()
    .reverse()
    .find((item) => (
      normalizeTarget(item.target) === normalizedTarget
      && outcomes.includes(item.outcome)
    )) ?? null;
}

function countTransitions(trace: NoNameTrace, suffix: string) {
  return trace.proposalTransitionLog?.filter((item) => item.endsWith(suffix)).length ?? 0;
}

function countTransitionMatches(trace: NoNameTrace, predicate: (item: string) => boolean) {
  return trace.proposalTransitionLog?.filter(predicate).length ?? 0;
}

function countExecutionOutcomes(trace: NoNameTrace, outcome: string) {
  return trace.applyExecutionLog?.filter((item) => item.outcome === outcome).length ?? 0;
}

function countExecutionMatches(trace: NoNameTrace, predicate: (item: NoNameApplyExecutionRecord) => boolean) {
  return trace.applyExecutionLog?.filter(predicate).length ?? 0;
}

function countOutcomeEvidence(
  trace: NoNameTrace,
  transitionSuffix: string,
  executionOutcome: string,
) {
  const transitionCount = countTransitions(trace, transitionSuffix);
  return transitionCount > 0
    ? transitionCount
    : countExecutionOutcomes(trace, executionOutcome);
}

function countManualApplyEvidence(trace: NoNameTrace) {
  const transitionCount = countTransitionMatches(
    trace,
    (item) => item.includes(':manual_apply:'),
  );
  if (transitionCount > 0) {
    return transitionCount;
  }
  return countExecutionMatches(trace, (item) => [
    'manual_plot_text_applied',
    'manual_chapter_summary_hint_applied',
    'manual_option_bias_hint_applied',
    'manual_plot_augmentation_hint_applied',
  ].includes(item.outcome));
}

function buildSecondGuardrailStats(trace: NoNameTrace): NoNameSecondGuardrailStats {
  return {
    allowed: countOutcomeEvidence(trace, ':second_guardrail:allow', 'second_guardrail_allowed'),
    rejected: countOutcomeEvidence(trace, ':second_guardrail:reject', 'second_guardrail_rejected'),
    fallback: countOutcomeEvidence(trace, ':second_guardrail:fallback', 'second_guardrail_fallback'),
    awaiting: countOutcomeEvidence(trace, ':apply_intent:awaiting_second_guardrail', 'awaiting_second_guardrail'),
    manualApplied: countManualApplyEvidence(trace),
  };
}

function hasMixedSecondGuardrailOutcomes(stats: NoNameSecondGuardrailStats) {
  return [
    stats.allowed > 0,
    stats.rejected > 0,
    stats.fallback > 0,
  ].filter(Boolean).length > 1;
}

function formatSecondGuardrailCounts(stats: NoNameSecondGuardrailStats) {
  return `allow=${stats.allowed}, reject=${stats.rejected}, fallback=${stats.fallback}`;
}

export function formatNoNameApplyExecutionRecord(
  record: NoNameApplyExecutionRecord,
): NoNameApplyExecutionDisplay {
  if (normalizeTarget(record.target) === normalizeTarget('plotAugmentationHint')) {
    if (record.outcome === 'pending_plot_augmentation_consumed') {
      return {
        targetLabel: '剧情增强提示',
        outcomeLabel: '已消费',
      };
    }
    if (record.outcome === 'pending_plot_augmentation_retained') {
      return {
        targetLabel: '剧情增强提示',
        outcomeLabel: '已保留',
      };
    }
    if (record.outcome === 'manual_plot_augmentation_hint_applied') {
      return {
        targetLabel: '剧情增强提示',
        outcomeLabel: '已暂存待消费',
      };
    }
  }
  return {
    targetLabel: record.target,
    outcomeLabel: record.outcome,
  };
}

export function summarizeNoNameApplyExecutions(
  trace: NoNameTrace,
  options: {
    emptyLabel?: string;
    rawPrefix?: string;
    rawSuffix?: string;
    notePrefix?: string;
    noteSuffix?: string;
  } = {},
) {
  const {
    emptyLabel = '无',
    rawPrefix = '[raw=',
    rawSuffix = ']',
    notePrefix = '(',
    noteSuffix = ')',
  } = options;
  const executionLog = trace.applyExecutionLog ?? [];
  if (executionLog.length === 0) {
    return emptyLabel;
  }
  return executionLog
    .map((item) => {
      const display = formatNoNameApplyExecutionRecord(item);
      const raw = display.targetLabel !== item.target || display.outcomeLabel !== item.outcome
        ? `${rawPrefix}${item.target}:${item.outcome}${rawSuffix}`
        : '';
      const note = item.note ? `${notePrefix}${item.note}${noteSuffix}` : '';
      return `${display.targetLabel}:${display.outcomeLabel}${raw}${note}`;
    })
    .join(', ');
}

function resolvePreflightTone(outcome: string | undefined): NoNameApplyLifecycleTone {
  if (!outcome) {
    return 'pending';
  }
  if (outcome.includes('fallback')) {
    return 'fallback';
  }
  if (outcome.includes('reject') || outcome.includes('rejected') || outcome.includes('blocked')) {
    return 'blocked';
  }
  if (
    outcome.includes('ready')
    || outcome.includes('applied')
    || outcome.includes('allow')
    || outcome.includes('manual_plot_text_applied')
    || outcome.includes('manual_plot_augmentation_hint_applied')
  ) {
    return 'done';
  }
  return 'info';
}

function lifecycleHumanDecision(
  trace: NoNameTrace,
  requestId: string,
  reviewDecisions: Record<string, NoNameHumanReviewDecision>,
) {
  return reviewDecisions[requestId]
    ?? trace.controlledOutputReviews?.find((item) => item.requestId === requestId)?.humanReviewDecision
    ?? 'pending';
}

export function buildNoNameApplyLifecycleCheckpoints(
  trace: NoNameTrace,
  reviewDecisions: Record<string, NoNameHumanReviewDecision> = {},
): NoNameApplyLifecycleCheckpoint[] {
  const controlledReviews = trace.controlledOutputReviews ?? [];
  const humanReviews = controlledReviews.filter((item) => item.requiresHumanReview);
  const decisionCounts = humanReviews.reduce(
    (counts, review) => {
      const decision = lifecycleHumanDecision(trace, review.requestId, reviewDecisions);
      counts[decision] += 1;
      return counts;
    },
    {
      pending: 0,
      approvedForHigherApply: 0,
      rejectedForHigherApply: 0,
    } satisfies Record<NoNameHumanReviewDecision, number>,
  );
  const secondGuardrailStats = buildSecondGuardrailStats(trace);
  const secondGuardrailAllowed = secondGuardrailStats.allowed > 0;
  const secondGuardrailRejected = secondGuardrailStats.rejected > 0;
  const secondGuardrailFallback = secondGuardrailStats.fallback > 0;
  const mixedSecondGuardrail = hasMixedSecondGuardrailOutcomes(secondGuardrailStats);
  const awaitingSecondGuardrail = secondGuardrailStats.awaiting > 0;
  const manualApplied = secondGuardrailStats.manualApplied > 0;
  const needsManualApplyPath = humanReviews.length > 0
    || awaitingSecondGuardrail
    || secondGuardrailAllowed
    || secondGuardrailRejected
    || secondGuardrailFallback
    || manualApplied;

  const reviewCheckpoint: NoNameApplyLifecycleCheckpoint = humanReviews.length === 0
    ? {
      key: 'human-review',
      label: 'Human Review',
      order: 1,
      state: 'not-required',
      detail: 'No controlled output currently requires human review.',
      tone: 'info',
    }
    : decisionCounts.rejectedForHigherApply > 0
      ? {
        key: 'human-review',
        label: 'Human Review',
        order: 1,
        state: 'rejected',
        detail: `${decisionCounts.rejectedForHigherApply}/${humanReviews.length} review item(s) rejected for higher apply.`,
        tone: 'blocked',
      }
      : decisionCounts.pending > 0
        ? {
          key: 'human-review',
          label: 'Human Review',
          order: 1,
          state: 'pending',
          detail: `${decisionCounts.pending}/${humanReviews.length} review item(s) still waiting for explicit confirmation.`,
          tone: 'pending',
        }
        : {
          key: 'human-review',
          label: 'Human Review',
          order: 1,
          state: 'approved',
          detail: `${decisionCounts.approvedForHigherApply}/${humanReviews.length} review item(s) approved for the next guardrail.`,
          tone: 'done',
        };

  let secondGuardrailCheckpoint: NoNameApplyLifecycleCheckpoint;
  if (!needsManualApplyPath) {
    secondGuardrailCheckpoint = {
      key: 'second-guardrail',
      label: 'Second Guardrail',
      order: 2,
      state: 'not-required',
      detail: 'No higher-layer apply path is currently staged.',
      tone: 'info',
    };
  } else if (mixedSecondGuardrail) {
    secondGuardrailCheckpoint = {
      key: 'second-guardrail',
      label: 'Second Guardrail',
      order: 2,
      state: 'mixed',
      detail: `Multiple reviewed apply scopes have different second guardrail outcomes (${formatSecondGuardrailCounts(secondGuardrailStats)}).`,
      tone: 'blocked',
    };
  } else if (secondGuardrailFallback) {
    secondGuardrailCheckpoint = {
      key: 'second-guardrail',
      label: 'Second Guardrail',
      order: 2,
      state: 'fallback',
      detail: 'The second guardrail requested fallback to the classic path.',
      tone: 'fallback',
    };
  } else if (secondGuardrailRejected) {
    secondGuardrailCheckpoint = {
      key: 'second-guardrail',
      label: 'Second Guardrail',
      order: 2,
      state: 'rejected',
      detail: 'The second guardrail rejected the higher-layer apply.',
      tone: 'blocked',
    };
  } else if (secondGuardrailAllowed) {
    secondGuardrailCheckpoint = {
      key: 'second-guardrail',
      label: 'Second Guardrail',
      order: 2,
      state: 'allowed',
      detail: 'The second guardrail allows explicit manual apply only.',
      tone: 'done',
    };
  } else if (decisionCounts.rejectedForHigherApply > 0) {
    secondGuardrailCheckpoint = {
      key: 'second-guardrail',
      label: 'Second Guardrail',
      order: 2,
      state: 'blocked-by-review',
      detail: 'Human review rejection blocks the second guardrail path.',
      tone: 'blocked',
    };
  } else if (decisionCounts.pending > 0 || awaitingSecondGuardrail) {
    secondGuardrailCheckpoint = {
      key: 'second-guardrail',
      label: 'Second Guardrail',
      order: 2,
      state: 'waiting',
      detail: 'Waiting for approved review evidence before resolving the second guardrail.',
      tone: 'pending',
    };
  } else {
    secondGuardrailCheckpoint = {
      key: 'second-guardrail',
      label: 'Second Guardrail',
      order: 2,
      state: 'not-entered',
      detail: 'No second guardrail resolution has been recorded yet.',
      tone: 'pending',
    };
  }

  let manualApplyCheckpoint: NoNameApplyLifecycleCheckpoint;
  if (manualApplied && mixedSecondGuardrail) {
    manualApplyCheckpoint = {
      key: 'manual-apply',
      label: 'Manual Apply',
      order: 3,
      state: 'partially-applied',
      detail: `Some reviewed apply scopes were manually applied while others are blocked (${formatSecondGuardrailCounts(secondGuardrailStats)}).`,
      tone: 'blocked',
    };
  } else if (manualApplied) {
    manualApplyCheckpoint = {
      key: 'manual-apply',
      label: 'Manual Apply',
      order: 3,
      state: 'applied',
      detail: 'An explicit manual apply execution is recorded.',
      tone: 'done',
    };
  } else if (!needsManualApplyPath) {
    manualApplyCheckpoint = {
      key: 'manual-apply',
      label: 'Manual Apply',
      order: 3,
      state: 'not-required',
      detail: 'No manual apply scope is currently staged.',
      tone: 'info',
    };
  } else if (mixedSecondGuardrail && secondGuardrailAllowed) {
    manualApplyCheckpoint = {
      key: 'manual-apply',
      label: 'Manual Apply',
      order: 3,
      state: 'partially-ready',
      detail: `Only the allowed reviewed apply scope(s) can continue; rejected or fallback scope(s) stay blocked (${formatSecondGuardrailCounts(secondGuardrailStats)}).`,
      tone: 'blocked',
    };
  } else if (secondGuardrailAllowed) {
    manualApplyCheckpoint = {
      key: 'manual-apply',
      label: 'Manual Apply',
      order: 3,
      state: 'awaiting-command',
      detail: 'Ready for an explicit developer-triggered apply command.',
      tone: 'pending',
    };
  } else if (secondGuardrailRejected || secondGuardrailFallback || decisionCounts.rejectedForHigherApply > 0) {
    manualApplyCheckpoint = {
      key: 'manual-apply',
      label: 'Manual Apply',
      order: 3,
      state: 'blocked',
      detail: 'Manual apply is blocked by review or second guardrail result.',
      tone: secondGuardrailFallback ? 'fallback' : 'blocked',
    };
  } else {
    manualApplyCheckpoint = {
      key: 'manual-apply',
      label: 'Manual Apply',
      order: 3,
      state: 'not-ready',
      detail: 'Manual apply is waiting for review and second guardrail evidence.',
      tone: 'pending',
    };
  }

  return [
    reviewCheckpoint,
    secondGuardrailCheckpoint,
    manualApplyCheckpoint,
  ];
}

export function summarizeNoNameApplyLifecycleCheckpoints(
  trace: NoNameTrace,
  reviewDecisions: Record<string, NoNameHumanReviewDecision> = {},
) {
  return buildNoNameApplyLifecycleCheckpoints(trace, reviewDecisions)
    .map((checkpoint) => `${checkpoint.order}.${checkpoint.label}=${checkpoint.state}`)
    .join(' -> ');
}

export function summarizeNoNamePendingPlotAugmentation(trace: NoNameTrace): string {
  const consumptionExecution = latestExecutionForTarget(
    trace,
    'plotAugmentationHint',
    ['pending_plot_augmentation_consumed', 'pending_plot_augmentation_retained'],
  );
  if (consumptionExecution?.outcome === 'pending_plot_augmentation_consumed') {
    return consumptionExecution.note
      ? `已消费（${consumptionExecution.note}）`
      : '已消费';
  }
  if (consumptionExecution?.outcome === 'pending_plot_augmentation_retained') {
    return consumptionExecution.note
      ? `已保留（${consumptionExecution.note}）`
      : '已保留';
  }

  const stagedExecution = latestExecutionForTarget(
    trace,
    'plotAugmentationHint',
    ['manual_plot_augmentation_hint_applied'],
  );
  if (stagedExecution) {
    return stagedExecution.note
      ? `待消费（${stagedExecution.note}）`
      : '待消费';
  }

  const latestProposal = trace.proposals[trace.proposals.length - 1] ?? null;
  const hasPlotAugmentationScope = latestProposal?.applyScopes?.some((scope) => (
    normalizeTarget(scope) === normalizeTarget('plotAugmentationHint')
  ));
  if (hasPlotAugmentationScope) {
    return '待观察（proposal 支持 PlotAugmentationHint，但尚未看到 pending 消费记录）';
  }

  return '无';
}

export function buildNoNameApplyLifecycle(
  trace: NoNameTrace,
  reviewDecisions: Record<string, NoNameHumanReviewDecision> = {},
): NoNameApplyLifecycleStep[] {
  const latestProposal = trace.proposals[trace.proposals.length - 1] ?? null;
  const controlledReviews = trace.controlledOutputReviews ?? [];
  const humanReviews = controlledReviews.filter((item) => item.requiresHumanReview);
  const pendingHumanReviewCount = humanReviews.filter((item) => (
    lifecycleHumanDecision(trace, item.requestId, reviewDecisions) === 'pending'
  )).length;
  const lowRiskTargets = ['diagnostics', 'chapterSummaryHint', 'optionBiasHint'];
  const appliedLowRiskTargets = lowRiskTargets.filter((target) => hasExecution(trace, target, 'applied'));
  const pendingAugmentationExecution = latestExecutionForTarget(
    trace,
    'plotAugmentationHint',
    ['pending_plot_augmentation_consumed', 'pending_plot_augmentation_retained'],
  );
  const latestProposalScopes = latestProposal?.applyScopes ?? [];
  const hasPlotAugmentationScope = latestProposalScopes
    .some((scope) => normalizeTarget(scope) === normalizeTarget('plotAugmentationHint'));
  const secondGuardrailStats = buildSecondGuardrailStats(trace);
  const secondGuardrailAllowed = secondGuardrailStats.allowed > 0;
  const secondGuardrailRejected = secondGuardrailStats.rejected > 0;
  const secondGuardrailFallback = secondGuardrailStats.fallback > 0;
  const mixedSecondGuardrail = hasMixedSecondGuardrailOutcomes(secondGuardrailStats);
  const awaitingSecondGuardrail = secondGuardrailStats.awaiting > 0;
  const manualApplied = secondGuardrailStats.manualApplied > 0;
  const preflightOutcome = trace.applyResult?.outcome;

  const steps: NoNameApplyLifecycleStep[] = [
    {
      key: 'proposal',
      label: '提案阶段',
      state: latestProposal
        ? (latestProposal.status ?? (latestProposal.applyable ? 'ready' : 'observed'))
        : '无提案',
      detail: latestProposal
        ? `${latestProposal.producerRole} · ${latestProposal.targetSegment} · ${latestProposal.applyScopes?.join('/') || 'no-scope'}`
        : '尚未生成 NoName proposal',
      tone: latestProposal?.applyable || latestProposal?.status === 'applied' || latestProposal?.status === 'ready'
        ? 'done'
        : 'info',
    },
    {
      key: 'preflight',
      label: 'Apply 预检',
      state: preflightOutcome ?? '未尝试',
      detail: trace.applyResult?.reason ?? '尚未记录 applyResult',
      tone: resolvePreflightTone(preflightOutcome),
    },
    {
      key: 'low-risk',
      label: '低风险输出',
      state: appliedLowRiskTargets.length > 0 ? '已应用' : '未应用',
      detail: appliedLowRiskTargets.length > 0
        ? `已应用：${appliedLowRiskTargets.join(' / ')}`
        : '尚未看到 diagnostics / chapterSummaryHint / optionBiasHint 的 applied 记录',
      tone: appliedLowRiskTargets.length > 0 ? 'done' : 'pending',
    },
    {
      key: 'review',
      label: '人工复核',
      state: humanReviews.length === 0
        ? '无需人工'
        : pendingHumanReviewCount === 0
          ? '已确认'
          : `待确认 ${pendingHumanReviewCount}`,
      detail: controlledReviews.length > 0
        ? `受控输出 ${controlledReviews.length} 条，其中人工复核 ${humanReviews.length} 条`
        : '无 controlled output review',
      tone: pendingHumanReviewCount > 0 ? 'pending' : 'done',
    },
  ];

  if (pendingAugmentationExecution || hasPlotAugmentationScope) {
    const consumed = pendingAugmentationExecution?.outcome === 'pending_plot_augmentation_consumed';
    const retained = pendingAugmentationExecution?.outcome === 'pending_plot_augmentation_retained';
    steps.push({
      key: 'plot-augmentation-consumption',
      label: '剧情增强消费',
      state: consumed
        ? '已消费'
        : retained
          ? '已保留'
          : '待观察',
      detail: pendingAugmentationExecution?.note
        ?? 'PlotAugmentationHint 已进入待消费层，等待下一次 plot_engine 生成结果确认',
      tone: consumed ? 'done' : retained ? 'pending' : 'info',
    });
  }

  if (humanReviews.length > 0 || awaitingSecondGuardrail || secondGuardrailAllowed || secondGuardrailRejected || secondGuardrailFallback) {
    steps.push({
      key: 'second-guardrail',
      label: '二次护栏',
      state: mixedSecondGuardrail
        ? 'mixed'
        : secondGuardrailFallback
          ? 'fallback'
          : secondGuardrailRejected
          ? 'rejected'
          : secondGuardrailAllowed
            ? 'allow'
            : awaitingSecondGuardrail
              ? 'waiting'
              : '未进入',
      detail: mixedSecondGuardrail
        ? `Multiple second guardrail outcomes: ${formatSecondGuardrailCounts(secondGuardrailStats)}`
        : secondGuardrailAllowed
        ? '已允许进入显式人工 apply；仍不会自动写正文'
        : secondGuardrailFallback
          ? '已要求回退经典链路'
          : secondGuardrailRejected
            ? '已拒绝高层 apply'
            : '等待人工批准后进入二次护栏',
      tone: mixedSecondGuardrail
        ? 'blocked'
        : secondGuardrailFallback
          ? 'fallback'
          : secondGuardrailRejected
          ? 'blocked'
          : secondGuardrailAllowed
            ? 'done'
            : 'pending',
    });
  }

  if (
    humanReviews.some((item) => (
      ['plotTextHint', 'chapterSummaryHint', 'optionBiasHint', 'plotAugmentationHint']
        .includes(item.safeApplyScope ?? '')
    ))
    || secondGuardrailAllowed
    || manualApplied
  ) {
    steps.push({
      key: 'manual-plot-text',
      label: '人工写入',
      state: manualApplied && mixedSecondGuardrail
        ? 'partially-applied'
        : manualApplied
        ? '已写入'
        : mixedSecondGuardrail && secondGuardrailAllowed
          ? 'partially-ready'
          : secondGuardrailAllowed
            ? '等待显式命令'
            : '未就绪',
      detail: manualApplied && mixedSecondGuardrail
        ? `Some manual apply scopes completed while others are blocked: ${formatSecondGuardrailCounts(secondGuardrailStats)}`
        : manualApplied
        ? '已记录显式人工 apply 结果'
        : mixedSecondGuardrail && secondGuardrailAllowed
          ? `Only allowed scopes can continue: ${formatSecondGuardrailCounts(secondGuardrailStats)}`
          : secondGuardrailAllowed
          ? '需要开发者确认差异预览后手动写入'
          : 'PlotTextHint 需要人工复核与二次护栏',
      tone: manualApplied
        ? (mixedSecondGuardrail ? 'blocked' : 'done')
        : mixedSecondGuardrail
          ? 'blocked'
          : secondGuardrailAllowed
            ? 'pending'
            : 'info',
    });
  }

  if (trace.fallbackUsed) {
    steps.push({
      key: 'fallback',
      label: '回退',
      state: '已回退',
      detail: 'fallbackUsed=true，后续继续依赖经典链路',
      tone: 'fallback',
    });
  }

  return steps;
}

export function summarizeNoNameApplyLifecycle(
  trace: NoNameTrace,
  reviewDecisions: Record<string, NoNameHumanReviewDecision> = {},
) {
  return buildNoNameApplyLifecycle(trace, reviewDecisions)
    .map((step) => `${step.label}:${step.state}`)
    .join(' -> ');
}
