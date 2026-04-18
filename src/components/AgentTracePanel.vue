<template>
  <section class="agent-trace-panel">
    <div
      v-if="!trace"
      class="agent-trace-empty"
    >
      暂无可展示的 NoName Trace。
    </div>
    <template v-else>
      <header class="agent-trace-header">
        <div>
          <p class="agent-trace-eyebrow">
            Trace #{{ selectedIndex + 1 }} / {{ totalCount }}
          </p>
          <h3 class="agent-trace-title">
            {{ trace.traceId }}
          </h3>
        </div>
        <div class="agent-trace-badges">
          <span class="agent-trace-badge">
            运行模式 {{ activeMode || trace.mode }}
          </span>
          <span
            class="agent-trace-badge"
            :class="trace.fallbackUsed ? 'agent-trace-badge-warn' : 'agent-trace-badge-ok'"
          >
            {{ trace.fallbackUsed ? '已回退' : '未回退' }}
          </span>
        </div>
      </header>

      <div class="agent-trace-grid">
        <section class="agent-trace-card">
          <p class="agent-trace-card-title">
            运行概览
          </p>
          <ul class="agent-trace-list">
            <li>turnId：{{ trace.turnId }}</li>
            <li>graphPath：{{ trace.graphPath.length > 0 ? trace.graphPath.join(' -> ') : '无' }}</li>
            <li>elapsedMs：{{ trace.elapsedMs }} ms</li>
            <li>guardrail：{{ guardrailLabel }}</li>
            <li>applyResult：{{ applyResultLabel }}</li>
          </ul>
        </section>

        <section class="agent-trace-card">
          <p class="agent-trace-card-title">
            最新提案
          </p>
          <div v-if="latestProposal">
            <p class="agent-trace-main-line">
              {{ latestProposal.title }}
            </p>
            <p class="agent-trace-muted">
              状态：{{ latestProposal.status || (latestProposal.applyable ? 'ready' : 'observed') }}
            </p>
            <p class="agent-trace-muted">
              类型：{{ latestProposal.kind }} / 角色：{{ latestProposal.producerRole }}
            </p>
            <p class="agent-trace-muted">
              目标段：{{ latestProposal.targetSegment }}
            </p>
            <p class="agent-trace-muted">
              作用域：{{ latestProposal.applyScopes?.length ? latestProposal.applyScopes.join(' / ') : '无' }}
            </p>
            <p class="agent-trace-body">
              预期效果：{{ latestProposal.intendedEffect }}
            </p>
            <p class="agent-trace-body">
              理由：{{ latestProposal.rationale }}
            </p>
            <p
              v-if="latestProposal.suggestedAction"
              class="agent-trace-muted"
            >
              建议动作：{{ latestProposal.suggestedAction }}
            </p>
          </div>
          <p
            v-else
            class="agent-trace-muted"
          >
            暂无提案
          </p>
        </section>
      </div>

      <section class="agent-trace-card">
        <p class="agent-trace-card-title">
          应用生命周期
        </p>
        <ul class="agent-trace-lifecycle">
          <li
            v-for="step in applyLifecycleSteps"
            :key="step.key"
            class="agent-trace-lifecycle-step"
            :class="`is-${step.tone}`"
          >
            <div class="agent-trace-lifecycle-head">
              <span class="agent-trace-main-line">{{ step.label }}</span>
              <span class="agent-trace-lifecycle-state">{{ step.state }}</span>
            </div>
            <p class="agent-trace-muted">
              {{ step.detail }}
            </p>
          </li>
        </ul>
      </section>

      <section class="agent-trace-card">
        <p class="agent-trace-card-title">
          状态迁移
        </p>
        <p
          v-if="!trace.proposalTransitionLog?.length"
          class="agent-trace-muted"
        >
          暂无状态迁移
        </p>
        <ul
          v-else
          class="agent-trace-rows"
        >
          <li
            v-for="(item, index) in trace.proposalTransitionLog"
            :key="`transition-${index}-${item}`"
            class="agent-trace-row"
          >
            {{ item }}
          </li>
        </ul>
      </section>

      <section class="agent-trace-card">
        <p class="agent-trace-card-title">
          协作观察
        </p>
        <p
          v-if="relatedObservations.length === 0"
          class="agent-trace-muted"
        >
          暂无 fan-out 角色观察
        </p>
        <ul
          v-else
          class="agent-trace-rows"
        >
          <li
            v-for="item in relatedObservations"
            :key="`${item.role}-${item.proposal.proposalId}`"
            class="agent-trace-row"
          >
            <p class="agent-trace-main-line">
              {{ item.role }} · {{ item.proposal.title }}
            </p>
            <p class="agent-trace-muted">
              焦点：{{ item.focus }} · 目标段：{{ item.proposal.targetSegment }}
            </p>
            <p class="agent-trace-muted">
              状态：{{ item.proposal.status || (item.proposal.applyable ? 'ready' : 'observed') }}
            </p>
            <p class="agent-trace-muted">
              理由：{{ item.rationale }}
            </p>
          </li>
        </ul>
      </section>

      <section class="agent-trace-card">
        <p class="agent-trace-card-title">
          协议事件
        </p>
        <p
          v-if="protocolEvents.length === 0"
          class="agent-trace-muted"
        >
          暂无协议事件
        </p>
        <ul
          v-else
          class="agent-trace-rows"
        >
          <li
            v-for="(event, index) in protocolEvents"
            :key="`${event.taskId}-${event.kind}-${index}`"
            class="agent-trace-row"
          >
            <p class="agent-trace-main-line">
              {{ event.channel }} · {{ event.kind }} · {{ event.status }}
            </p>
            <p class="agent-trace-muted">
              {{ event.from || 'runtime' }} → {{ event.to || 'runtime' }} · task={{ event.taskId }}
            </p>
            <p
              v-if="event.detail"
              class="agent-trace-muted"
            >
              detail：{{ event.detail }}
            </p>
          </li>
        </ul>
      </section>

      <section class="agent-trace-card">
        <p class="agent-trace-card-title">
          受控输出复核
        </p>
        <p
          v-if="controlledOutputReviews.length === 0"
          class="agent-trace-muted"
        >
          暂无受控输出复核记录
        </p>
        <ul
          v-else
          class="agent-trace-rows"
        >
          <li
            v-for="review in controlledOutputReviews"
            :key="review.requestId"
            class="agent-trace-row"
          >
            <p class="agent-trace-main-line">
              {{ review.requestedKind }} · {{ review.safeApplyScope || '无作用域' }} · {{ review.decision }}
            </p>
            <p class="agent-trace-muted">
              {{ review.requiresHumanReview ? '需要人工复核' : '可自动通过' }} · {{ review.reason }}
            </p>
            <p
              v-if="review.policyForbiddenScopes?.length"
              class="agent-trace-muted"
            >
              策略禁区：{{ review.policyForbiddenScopes.join(' / ') }}
            </p>
            <p
              v-if="review.requiresHumanReview"
              class="agent-trace-review-state"
            >
              {{ humanReviewDecisionLabel(review.requestId) }}
            </p>
            <div
              v-if="review.requiresHumanReview"
              class="agent-trace-review-actions"
            >
              <button
                type="button"
                class="agent-trace-review-btn"
                :class="{ 'is-active': humanReviewDecision(review.requestId) === 'approvedForHigherApply' }"
                @click="markHumanReview(review.requestId, 'approvedForHigherApply')"
              >
                标记可进入高层 apply 设计
              </button>
              <button
                type="button"
                class="agent-trace-review-btn"
                :class="{ 'is-active': humanReviewDecision(review.requestId) === 'rejectedForHigherApply' }"
                @click="markHumanReview(review.requestId, 'rejectedForHigherApply')"
              >
                暂不应用
              </button>
              <button
                v-if="humanReviewDecision(review.requestId) !== 'pending'"
                type="button"
                class="agent-trace-review-btn agent-trace-review-btn-ghost"
                @click="markHumanReview(review.requestId, 'pending')"
              >
                重置待复核
              </button>
            </div>
            <div
              v-if="humanReviewDecision(review.requestId) === 'approvedForHigherApply'"
              class="agent-trace-review-actions"
            >
              <button
                type="button"
                class="agent-trace-review-btn"
                @click="resolveSecondGuardrail(review.requestId, 'allow')"
              >
                二次护栏允许
              </button>
              <button
                type="button"
                class="agent-trace-review-btn"
                @click="resolveSecondGuardrail(review.requestId, 'reject')"
              >
                二次护栏拒绝
              </button>
              <button
                type="button"
                class="agent-trace-review-btn agent-trace-review-btn-ghost"
                @click="resolveSecondGuardrail(review.requestId, 'fallback')"
              >
                回退经典链路
              </button>
            </div>
            <div
              v-if="hasSecondGuardrailAllow(review.requestId)"
              class="agent-trace-manual-preview"
              :class="manualApplyPreview(review.requestId).toneClass"
            >
              <p class="agent-trace-main-line">
                人工写入预览：{{ manualApplyPreview(review.requestId).statusLabel }}
              </p>
              <p class="agent-trace-muted">
                {{ manualApplyPreview(review.requestId).statusText }}
              </p>
              <div
                v-if="manualApplyPreview(review.requestId).before && manualApplyPreview(review.requestId).after"
                class="agent-trace-manual-preview-grid"
              >
                <div>
                  <p class="agent-trace-preview-label">
                    写入前
                  </p>
                  <pre class="agent-trace-preview-text">{{ manualApplyPreview(review.requestId).before }}</pre>
                </div>
                <div>
                  <p class="agent-trace-preview-label">
                    写入后
                  </p>
                  <pre class="agent-trace-preview-text">{{ manualApplyPreview(review.requestId).after }}</pre>
                </div>
              </div>
            </div>
            <div
              v-if="hasSecondGuardrailAllow(review.requestId)"
              class="agent-trace-review-actions"
            >
              <button
                type="button"
                class="agent-trace-review-btn is-active"
                :disabled="!manualApplyPreview(review.requestId).canApply"
                @click="applyManualPlotTextHint(review.requestId)"
              >
                显式人工写入正文提示
              </button>
            </div>
          </li>
        </ul>
      </section>

      <div class="agent-trace-grid">
        <section class="agent-trace-card">
          <p class="agent-trace-card-title">
            应用计划
          </p>
          <p
            v-if="!trace.applyPlanLog?.length"
            class="agent-trace-muted"
          >
            暂无应用计划
          </p>
          <ul
            v-else
            class="agent-trace-rows"
          >
            <li
              v-for="plan in trace.applyPlanLog"
              :key="`plan-${plan.order}-${plan.target}-${plan.decision}`"
              class="agent-trace-row"
            >
              <p class="agent-trace-main-line">
                #{{ plan.order }} · {{ plan.target }} · {{ plan.decision }} · P{{ plan.priority }}
              </p>
              <p
                v-if="plan.note"
                class="agent-trace-muted"
              >
                {{ plan.note }}
              </p>
            </li>
          </ul>
        </section>

        <section class="agent-trace-card">
          <p class="agent-trace-card-title">
            应用执行
          </p>
          <p
            v-if="!trace.applyExecutionLog?.length"
            class="agent-trace-muted"
          >
            暂无应用执行
          </p>
          <ul
            v-else
            class="agent-trace-rows"
          >
            <li
              v-for="execution in trace.applyExecutionLog"
              :key="`execution-${execution.target}-${execution.outcome}`"
              class="agent-trace-row"
            >
              <p class="agent-trace-main-line">
                {{ execution.target }} · {{ execution.outcome }}
              </p>
              <p
                v-if="execution.note"
                class="agent-trace-muted"
              >
                {{ execution.note }}
              </p>
            </li>
          </ul>
        </section>
      </div>

      <section class="agent-trace-card">
        <p class="agent-trace-card-title">
          能力调用
        </p>
        <p
          v-if="!trace.capabilityCalls.length"
          class="agent-trace-muted"
        >
          暂无能力调用记录
        </p>
        <ul
          v-else
          class="agent-trace-rows"
        >
          <li
            v-for="(call, index) in trace.capabilityCalls"
            :key="`call-${index}-${call.capabilityId}`"
            class="agent-trace-row"
          >
            <span>{{ call.callKind }}</span>
            <span>{{ call.capabilityId }}</span>
            <span>{{ call.status }}</span>
          </li>
        </ul>
      </section>
    </template>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type {
  NoNameHumanReviewDecision,
  NoNameHumanReviewMarkPayload,
  NoNameManualApplySegmentSnapshot,
  NoNameManualPlotTextApplyPayload,
  NoNameProposal,
  NoNameSecondGuardrailDecision,
  NoNameSecondGuardrailResolvePayload,
  NoNameTrace,
} from '../types/game';
import { buildNoNameApplyLifecycle } from '../utils/noNameApplyLifecycle';

const props = withDefaults(defineProps<{
  trace: NoNameTrace | null;
  selectedIndex?: number;
  totalCount?: number;
  activeMode?: string;
  reviewDecisions?: Record<string, NoNameHumanReviewDecision>;
  manualApplySegment?: NoNameManualApplySegmentSnapshot | null;
}>(), {
  selectedIndex: 0,
  totalCount: 0,
  activeMode: '',
  reviewDecisions: () => ({}),
  manualApplySegment: null,
});

const emit = defineEmits<{
  (event: 'mark-controlled-output-review', payload: NoNameHumanReviewMarkPayload): void;
  (event: 'resolve-second-guardrail', payload: NoNameSecondGuardrailResolvePayload): void;
  (event: 'apply-manual-plot-text-hint', payload: NoNameManualPlotTextApplyPayload): void;
}>();

const latestProposal = computed(() => {
  if (!props.trace || props.trace.proposals.length === 0) {
    return null;
  }
  return props.trace.proposals[props.trace.proposals.length - 1];
});

const relatedObservations = computed(() => props.trace?.relatedObservations ?? []);
const protocolEvents = computed(() => props.trace?.protocolEvents ?? []);
const controlledOutputReviews = computed(() => props.trace?.controlledOutputReviews ?? []);
const applyLifecycleSteps = computed(() => (
  props.trace ? buildNoNameApplyLifecycle(props.trace, props.reviewDecisions) : []
));

const guardrailLabel = computed(() => {
  const result = props.trace?.guardrailResult;
  if (!result) {
    return '无';
  }
  return result.reason ? `${result.outcome} (${result.reason})` : result.outcome;
});

const applyResultLabel = computed(() => {
  const result = props.trace?.applyResult;
  if (!result) {
    return '无';
  }
  return result.reason ? `${result.outcome} (${result.reason})` : result.outcome;
});

function humanReviewDecision(requestId: string): NoNameHumanReviewDecision {
  return props.reviewDecisions[requestId] ?? 'pending';
}

function humanReviewDecisionLabel(requestId: string) {
  const decision = humanReviewDecision(requestId);
  if (decision === 'approvedForHigherApply') {
    return '人工结论：可进入下一阶段 apply 设计；当前不会自动写入剧情正文。';
  }
  if (decision === 'rejectedForHigherApply') {
    return '人工结论：暂不应用；保持当前安全边界。';
  }
  return '等待开发者确认：当前只记录，不会自动写入剧情正文。';
}

function markHumanReview(requestId: string, decision: NoNameHumanReviewDecision) {
  if (!props.trace) {
    return;
  }
  emit('mark-controlled-output-review', {
    traceId: props.trace.traceId,
    requestId,
    decision,
  });
}

function resolveSecondGuardrail(requestId: string, decision: NoNameSecondGuardrailDecision) {
  if (!props.trace) {
    return;
  }
  emit('resolve-second-guardrail', {
    traceId: props.trace.traceId,
    requestId,
    decision,
  });
}

function hasSecondGuardrailAllow(requestId: string) {
  const transition = `${requestId}:second_guardrail:allow`;
  return Boolean(props.trace?.proposalTransitionLog?.includes(transition))
    || Boolean(props.trace?.applyExecutionLog?.some((item) => (
      (item.target === 'plot_text_hint' || item.target === 'plotTextHint')
      && item.outcome === 'second_guardrail_allowed'
    )));
}

function hasManualPlotTextApplied(requestId: string) {
  return Boolean(props.trace?.proposalTransitionLog?.includes(`${requestId}:manual_apply:plot_text_hint`))
    || Boolean(props.trace?.applyExecutionLog?.some((item) => (
      (item.target === 'plot_text_hint' || item.target === 'plotTextHint')
      && item.outcome === 'manual_plot_text_applied'
    )));
}

function findProposalForReview(requestId: string): NoNameProposal | null {
  const proposals = props.trace?.proposals ?? [];
  return [...proposals].reverse().find((proposal) => requestId.includes(proposal.proposalId))
    ?? proposals[proposals.length - 1]
    ?? null;
}

function buildManualPlotText(proposal: NoNameProposal, segmentText: string) {
  const hint = `【NoName】重点关注：${proposal.focus}`;
  if (proposal.targetSegment === 'current_turn_head') {
    return `${hint}\n\n${segmentText.trim()}`;
  }
  return `${segmentText.trim()}\n\n${hint}`;
}

function manualApplyPreview(requestId: string) {
  if (hasManualPlotTextApplied(requestId)) {
    return {
      canApply: false,
      toneClass: 'is-safe',
      statusLabel: '已写入',
      statusText: '这条 review 已记录 manual_plot_text_applied，避免重复写入同一条正文提示。',
      before: '',
      after: '',
    };
  }

  const segment = props.manualApplySegment;
  if (!segment || !segment.text.trim()) {
    return {
      canApply: false,
      toneClass: 'is-warn',
      statusLabel: '缺少当前段落快照',
      statusText: '当前剧情段落为空，无法执行显式人工写入。',
      before: '',
      after: '',
    };
  }

  if (segment.text.includes('【NoName】') || segment.text.includes('NoName提示')) {
    return {
      canApply: false,
      toneClass: 'is-warn',
      statusLabel: '疑似已包含 NoName 标记',
      statusText: '当前段落已经包含 NoName 标记，为避免重复写入，按钮已禁用。',
      before: segment.text,
      after: '',
    };
  }

  const proposal = findProposalForReview(requestId);
  if (!proposal) {
    return {
      canApply: false,
      toneClass: 'is-warn',
      statusLabel: '找不到关联提案',
      statusText: '无法从 requestId 反查 proposal，暂不允许写入。',
      before: segment.text,
      after: '',
    };
  }

  return {
    canApply: true,
    toneClass: 'is-ready',
    statusLabel: `将写入第 ${segment.chapterIndex} 章第 ${segment.segmentIndex + 1} 段`,
    statusText: '请确认下方差异预览无误；后端还会再次校验章节、段落和正文快照。',
    before: segment.text,
    after: buildManualPlotText(proposal, segment.text),
  };
}

function applyManualPlotTextHint(requestId: string) {
  if (!props.trace) {
    return;
  }
  if (!manualApplyPreview(requestId).canApply) {
    return;
  }
  emit('apply-manual-plot-text-hint', {
    traceId: props.trace.traceId,
    requestId,
  });
}
</script>

<style scoped>
.agent-trace-panel {
  display: flex;
  flex-direction: column;
  gap: 14px;
  color: var(--ink-text-primary);
}

.agent-trace-empty,
.agent-trace-card {
  border: 1px solid var(--ink-border-soft);
  border-radius: 18px;
  background: color-mix(in srgb, var(--ink-card-bg-soft) 86%, transparent);
  box-shadow: 0 10px 26px rgba(39, 30, 18, 0.08);
  padding: 14px 16px;
}

.agent-trace-header {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  gap: 12px;
  align-items: flex-start;
}

.agent-trace-eyebrow {
  margin: 0;
  color: var(--ink-text-muted);
  font-size: 12px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
}

.agent-trace-title {
  margin: 6px 0 0;
  font-size: 22px;
  line-height: 1.1;
  color: var(--ink-title-color);
}

.agent-trace-badges {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.agent-trace-badge {
  border-radius: 999px;
  border: 1px solid var(--ink-border-accent);
  padding: 6px 10px;
  font-size: 12px;
  background: color-mix(in srgb, var(--ink-card-bg) 88%, transparent);
}

.agent-trace-badge-ok {
  color: #2f6b4b;
}

.agent-trace-badge-warn {
  color: #9b4d2e;
}

.agent-trace-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}

.agent-trace-card-title {
  margin: 0 0 10px;
  font-size: 13px;
  color: var(--ink-text-muted);
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.agent-trace-main-line {
  margin: 0;
  font-size: 15px;
  color: var(--ink-text-primary);
}

.agent-trace-body {
  margin: 8px 0 0;
  line-height: 1.6;
  color: var(--ink-text-primary);
}

.agent-trace-muted {
  margin: 8px 0 0;
  line-height: 1.55;
  color: var(--ink-text-muted);
}

.agent-trace-list,
.agent-trace-rows {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.agent-trace-lifecycle {
  list-style: none;
  padding: 0;
  margin: 0;
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
}

.agent-trace-lifecycle-step {
  border-radius: 14px;
  border: 1px solid color-mix(in srgb, var(--ink-border-soft) 72%, transparent);
  background: color-mix(in srgb, var(--ink-card-bg) 90%, transparent);
  padding: 10px 12px;
}

.agent-trace-lifecycle-step.is-done {
  border-color: color-mix(in srgb, #2f6b4b 46%, var(--ink-border-soft));
}

.agent-trace-lifecycle-step.is-pending {
  border-color: color-mix(in srgb, var(--ink-accent-main) 54%, var(--ink-border-soft));
}

.agent-trace-lifecycle-step.is-blocked,
.agent-trace-lifecycle-step.is-fallback {
  border-color: color-mix(in srgb, #9b4d2e 54%, var(--ink-border-soft));
}

.agent-trace-lifecycle-head {
  display: flex;
  justify-content: space-between;
  gap: 8px;
  align-items: center;
}

.agent-trace-lifecycle-state {
  flex: 0 0 auto;
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--ink-border-accent) 72%, transparent);
  padding: 3px 8px;
  color: var(--ink-text-muted);
  font-size: 12px;
}

.agent-trace-row {
  display: grid;
  gap: 4px;
  border-radius: 14px;
  border: 1px solid color-mix(in srgb, var(--ink-border-soft) 72%, transparent);
  background: color-mix(in srgb, var(--ink-card-bg) 90%, transparent);
  padding: 10px 12px;
}

.agent-trace-review-state {
  margin: 8px 0 0;
  border-radius: 12px;
  border: 1px solid color-mix(in srgb, var(--ink-border-accent) 72%, transparent);
  background: color-mix(in srgb, var(--ink-accent-main) 10%, var(--ink-card-bg));
  color: var(--ink-text-primary);
  padding: 8px 10px;
  line-height: 1.5;
}

.agent-trace-review-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 8px;
}

.agent-trace-manual-preview {
  margin-top: 10px;
  border-radius: 14px;
  border: 1px solid color-mix(in srgb, var(--ink-border-soft) 72%, transparent);
  background: color-mix(in srgb, var(--ink-card-bg) 90%, transparent);
  padding: 10px 12px;
}

.agent-trace-manual-preview.is-ready {
  border-color: color-mix(in srgb, #2f6b4b 44%, var(--ink-border-soft));
}

.agent-trace-manual-preview.is-warn {
  border-color: color-mix(in srgb, #9b4d2e 54%, var(--ink-border-soft));
}

.agent-trace-manual-preview.is-safe {
  border-color: color-mix(in srgb, var(--ink-accent-main) 48%, var(--ink-border-soft));
}

.agent-trace-manual-preview-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
  margin-top: 10px;
}

.agent-trace-preview-label {
  margin: 0 0 6px;
  color: var(--ink-text-muted);
  font-size: 12px;
}

.agent-trace-preview-text {
  margin: 0;
  max-height: 170px;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-word;
  border-radius: 12px;
  border: 1px solid color-mix(in srgb, var(--ink-border-soft) 70%, transparent);
  background: color-mix(in srgb, var(--ink-card-bg-soft) 88%, transparent);
  padding: 8px;
  color: var(--ink-text-primary);
  font-family: "LXGW WenKai", "Noto Serif SC", serif;
  font-size: 12px;
  line-height: 1.55;
}

.agent-trace-review-btn {
  border: 1px solid color-mix(in srgb, var(--ink-border-accent) 78%, transparent);
  border-radius: 999px;
  background: color-mix(in srgb, var(--ink-card-bg) 92%, transparent);
  color: var(--ink-text-primary);
  cursor: pointer;
  padding: 7px 10px;
  transition: background-color 180ms ease, border-color 180ms ease, transform 120ms ease;
}

.agent-trace-review-btn:hover {
  transform: translateY(-1px);
}

.agent-trace-review-btn:disabled {
  cursor: not-allowed;
  opacity: 0.5;
  transform: none;
}

.agent-trace-review-btn.is-active {
  border-color: var(--ink-accent-main);
  background: color-mix(in srgb, var(--ink-accent-main) 20%, var(--ink-card-bg));
}

.agent-trace-review-btn-ghost {
  color: var(--ink-text-muted);
}

@media (max-width: 900px) {
  .agent-trace-grid {
    grid-template-columns: 1fr;
  }

  .agent-trace-lifecycle {
    grid-template-columns: 1fr;
  }

  .agent-trace-manual-preview-grid {
    grid-template-columns: 1fr;
  }
}
</style>
