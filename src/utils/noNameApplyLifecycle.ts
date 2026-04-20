import type {
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

function hasTransition(trace: NoNameTrace, suffix: string) {
  return Boolean(trace.proposalTransitionLog?.some((item) => item.endsWith(suffix)));
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
  const secondGuardrailAllowed = hasExecutionOutcome(trace, 'second_guardrail_allowed')
    || hasTransition(trace, ':second_guardrail:allow');
  const secondGuardrailRejected = hasExecutionOutcome(trace, 'second_guardrail_rejected')
    || hasTransition(trace, ':second_guardrail:reject');
  const secondGuardrailFallback = hasExecutionOutcome(trace, 'second_guardrail_fallback')
    || hasTransition(trace, ':second_guardrail:fallback');
  const awaitingSecondGuardrail = hasExecutionOutcome(trace, 'awaiting_second_guardrail')
    || hasTransition(trace, ':apply_intent:awaiting_second_guardrail');
  const manualApplied = hasExecutionOutcome(trace, 'manual_plot_text_applied')
    || hasTransition(trace, ':manual_apply:plot_text_hint');
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

  if (humanReviews.some((item) => item.safeApplyScope === 'plotTextHint') || secondGuardrailAllowed || manualApplied) {
    steps.push({
      key: 'manual-plot-text',
      label: '人工写入',
      state: manualApplied
        ? '已写入'
        : secondGuardrailAllowed
          ? '等待显式命令'
          : '未就绪',
      detail: manualApplied
        ? '已记录 manual_plot_text_applied'
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
