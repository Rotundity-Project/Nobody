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

export interface NoNameApplyExecutionDisplay {
  targetLabel: string;
  outcomeLabel: string;
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

function hasExecutionOutcome(trace: NoNameTrace, outcome: string) {
  return Boolean(trace.applyExecutionLog?.some((item) => item.outcome === outcome));
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

function hasTransition(trace: NoNameTrace, suffix: string) {
  return Boolean(trace.proposalTransitionLog?.some((item) => item.endsWith(suffix)));
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
  const secondGuardrailAllowed = hasExecutionOutcome(trace, 'second_guardrail_allowed')
    || hasTransition(trace, ':second_guardrail:allow');
  const secondGuardrailRejected = hasExecutionOutcome(trace, 'second_guardrail_rejected')
    || hasTransition(trace, ':second_guardrail:reject');
  const secondGuardrailFallback = hasExecutionOutcome(trace, 'second_guardrail_fallback')
    || hasTransition(trace, ':second_guardrail:fallback');
  const awaitingSecondGuardrail = hasExecutionOutcome(trace, 'awaiting_second_guardrail')
    || hasTransition(trace, ':apply_intent:awaiting_second_guardrail');
  const manualApplied = hasExecutionOutcome(trace, 'manual_plot_text_applied')
    || hasExecutionOutcome(trace, 'manual_chapter_summary_hint_applied')
    || hasExecutionOutcome(trace, 'manual_option_bias_hint_applied')
    || hasExecutionOutcome(trace, 'manual_plot_augmentation_hint_applied')
    || hasTransition(trace, ':manual_apply:plot_text_hint')
    || hasTransition(trace, ':manual_apply:chapter_summary_hint')
    || hasTransition(trace, ':manual_apply:option_bias_hint')
    || hasTransition(trace, ':manual_apply:plot_augmentation_hint');
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
      state: secondGuardrailFallback
        ? 'fallback'
        : secondGuardrailRejected
          ? 'rejected'
          : secondGuardrailAllowed
            ? 'allow'
            : awaitingSecondGuardrail
              ? 'waiting'
              : '未进入',
      detail: secondGuardrailAllowed
        ? '已允许进入显式人工 apply；仍不会自动写正文'
        : secondGuardrailFallback
          ? '已要求回退经典链路'
          : secondGuardrailRejected
            ? '已拒绝高层 apply'
            : '等待人工批准后进入二次护栏',
      tone: secondGuardrailFallback
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
      state: manualApplied
        ? '已写入'
        : secondGuardrailAllowed
          ? '等待显式命令'
          : '未就绪',
      detail: manualApplied
        ? '已记录显式人工 apply 结果'
        : secondGuardrailAllowed
          ? '需要开发者确认差异预览后手动写入'
          : 'PlotTextHint 需要人工复核与二次护栏',
      tone: manualApplied ? 'done' : secondGuardrailAllowed ? 'pending' : 'info',
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
