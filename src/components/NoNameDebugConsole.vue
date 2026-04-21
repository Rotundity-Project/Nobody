<template>
  <div
    v-if="isOpen"
    class="noname-debug-console-overlay"
    @click.self="$emit('close')"
  >
    <section class="noname-debug-console-shell">
      <header class="noname-debug-console-header">
        <div>
          <p class="noname-debug-console-eyebrow">
            NoName Agent
          </p>
          <h2 class="noname-debug-console-title">
            独立调试台
          </h2>
          <p class="noname-debug-console-meta">
            当前模式：{{ noNameMode }} · 历史 Trace：{{ traces.length }}
          </p>
        </div>
        <div class="noname-debug-console-actions">
          <button
            type="button"
            class="noname-debug-console-btn"
            :disabled="!selectedTraceReport"
            @click="copySelectedTraceReport"
          >
            复制摘要
          </button>
          <button
            v-if="isDevMode"
            type="button"
            class="noname-debug-console-btn"
            @click="$emit('clear-traces')"
          >
            清空 Trace
          </button>
          <button
            type="button"
            class="noname-debug-console-btn noname-debug-console-btn-primary"
            @click="$emit('close')"
          >
            关闭
          </button>
        </div>
      </header>

      <div class="noname-debug-console-mode-strip">
        <button
          v-for="item in modeOptions"
          :key="item.value"
          type="button"
          class="noname-debug-console-mode-btn"
          :class="{ 'is-active': noNameMode === item.value }"
          @click="$emit('set-no-name-mode', item.value)"
        >
          {{ item.label }}
        </button>
      </div>

      <div class="noname-debug-console-layout">
        <aside class="noname-debug-console-sidebar">
          <p class="noname-debug-console-section-title">
            Trace 列表
          </p>
          <div
            v-if="traces.length === 0"
            class="noname-debug-console-empty"
          >
            暂无 NoName Trace。
          </div>
          <div
            v-else
            class="noname-debug-console-trace-list"
          >
            <button
              v-for="(trace, index) in traces"
              :key="trace.traceId"
              type="button"
              class="noname-debug-console-trace-btn"
              :class="{ 'is-active': index === selectedIndex }"
              @click="selectedIndex = index"
            >
              <span class="noname-debug-console-trace-title">{{ trace.traceId }}</span>
              <span class="noname-debug-console-trace-meta">
                {{ trace.mode }} · {{ trace.turnId }} · {{ trace.elapsedMs }} ms
              </span>
            </button>
          </div>

          <section
            v-if="selectedTraceReport"
            class="noname-debug-console-report"
          >
            <div class="noname-debug-console-report-head">
              <p class="noname-debug-console-section-title">
                诊断摘要
              </p>
              <span
                v-if="copyStatus"
                class="noname-debug-console-copy-status"
              >
                {{ copyStatus }}
              </span>
            </div>
            <pre class="noname-debug-console-report-body">{{ selectedTraceReport }}</pre>
          </section>
        </aside>

        <main class="noname-debug-console-main">
          <AgentTracePanel
            :trace="selectedTrace"
            :selected-index="selectedIndex"
            :total-count="traces.length"
            :active-mode="noNameMode"
            :review-decisions="selectedReviewDecisions"
            :manual-apply-segment="manualApplySegment"
            :manual-apply-summary="manualApplySummary"
            :manual-apply-diagnostics="manualApplyDiagnostics"
            :manual-apply-plot-augmentations="manualApplyPlotAugmentations"
            @mark-controlled-output-review="markControlledOutputReview"
            @resolve-second-guardrail="resolveSecondGuardrail"
            @apply-manual-plot-text-hint="applyManualPlotTextHint"
            @apply-manual-chapter-summary-hint="applyManualChapterSummaryHint"
            @apply-manual-option-bias-hint="applyManualOptionBiasHint"
            @apply-manual-plot-augmentation-hint="applyManualPlotAugmentationHint"
          />
        </main>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import AgentTracePanel from './AgentTracePanel.vue';
import type {
  NoNameHumanReviewDecision,
  NoNameHumanReviewMarkPayload,
  NoNameManualApplyDiagnosticsSnapshot,
  NoNameManualApplyPlotAugmentationSnapshot,
  NoNameManualApplySegmentSnapshot,
  NoNameManualApplySummarySnapshot,
  NoNameManualChapterSummaryHintApplyPayload,
  NoNameManualOptionBiasHintApplyPayload,
  NoNameManualPlotAugmentationHintApplyPayload,
  NoNameManualPlotTextApplyPayload,
  NoNameMode,
  NoNameSecondGuardrailResolvePayload,
  NoNameTrace,
} from '../types/game';
import {
  summarizeNoNameApplyExecutions,
  summarizeNoNameApplyLifecycle,
  summarizeNoNamePendingPlotAugmentation,
} from '../utils/noNameApplyLifecycle';

const props = withDefaults(defineProps<{
  isOpen: boolean;
  traces: NoNameTrace[];
  noNameMode: NoNameMode;
  isDevMode?: boolean;
  manualApplySegment?: NoNameManualApplySegmentSnapshot | null;
  manualApplySummary?: NoNameManualApplySummarySnapshot | null;
  manualApplyDiagnostics?: NoNameManualApplyDiagnosticsSnapshot | null;
  manualApplyPlotAugmentations?: NoNameManualApplyPlotAugmentationSnapshot | null;
}>(), {
  isDevMode: false,
  manualApplySegment: null,
  manualApplySummary: null,
  manualApplyDiagnostics: null,
  manualApplyPlotAugmentations: null,
});

const emit = defineEmits<{
  (event: 'close'): void;
  (event: 'clear-traces'): void;
  (event: 'set-no-name-mode', mode: NoNameMode): void;
  (event: 'mark-controlled-output-review', payload: NoNameHumanReviewMarkPayload): void;
  (event: 'resolve-second-guardrail', payload: NoNameSecondGuardrailResolvePayload): void;
  (event: 'apply-manual-plot-text-hint', payload: NoNameManualPlotTextApplyPayload): void;
  (event: 'apply-manual-chapter-summary-hint', payload: NoNameManualChapterSummaryHintApplyPayload): void;
  (event: 'apply-manual-option-bias-hint', payload: NoNameManualOptionBiasHintApplyPayload): void;
  (event: 'apply-manual-plot-augmentation-hint', payload: NoNameManualPlotAugmentationHintApplyPayload): void;
}>();

const modeOptions: Array<{ value: NoNameMode; label: string }> = [
  { value: 'disabled', label: '关闭' },
  { value: 'observeOnly', label: '观察' },
  { value: 'assisted', label: '辅助' },
];

const manualSelectedIndex = ref(-1);
const selectedIndex = computed({
  get() {
    if (props.traces.length === 0) {
      return 0;
    }
    if (manualSelectedIndex.value >= 0 && manualSelectedIndex.value < props.traces.length) {
      return manualSelectedIndex.value;
    }
    return props.traces.length - 1;
  },
  set(value: number) {
    manualSelectedIndex.value = value;
  },
});

watch(
  () => props.traces.length,
  (length) => {
    if (length === 0) {
      manualSelectedIndex.value = 0;
      return;
    }
    if (manualSelectedIndex.value >= length) {
      manualSelectedIndex.value = length - 1;
    }
  },
);

const selectedTrace = computed(() => {
  if (props.traces.length === 0) {
    return null;
  }
  const index = selectedIndex.value;
  return props.traces[index] ?? props.traces[props.traces.length - 1];
});

const copyStatus = ref('');
const humanReviewDecisions = ref<Record<string, NoNameHumanReviewDecision>>({});

const selectedReviewDecisions = computed<Record<string, NoNameHumanReviewDecision>>(() => {
  const trace = selectedTrace.value;
  if (!trace) {
    return {};
  }
  return Object.fromEntries(
    (trace.controlledOutputReviews ?? []).map((review) => [
      review.requestId,
      humanReviewDecisions.value[humanReviewDecisionKey(trace.traceId, review.requestId)]
        ?? review.humanReviewDecision
        ?? 'pending',
    ]),
  );
});

const selectedTraceReport = computed(() => buildTraceReport(
  selectedTrace.value,
  selectedReviewDecisions.value,
));

async function copySelectedTraceReport() {
  const report = selectedTraceReport.value;
  if (!report) {
    return;
  }
  try {
    await navigator.clipboard?.writeText(report);
    copyStatus.value = '摘要已复制';
  } catch {
    copyStatus.value = '复制失败，请手动选择摘要';
  }
}

function markControlledOutputReview(payload: NoNameHumanReviewMarkPayload) {
  humanReviewDecisions.value = {
    ...humanReviewDecisions.value,
    [humanReviewDecisionKey(payload.traceId, payload.requestId)]: payload.decision,
  };
  emit('mark-controlled-output-review', payload);
}

function resolveSecondGuardrail(payload: NoNameSecondGuardrailResolvePayload) {
  emit('resolve-second-guardrail', payload);
}

function applyManualPlotTextHint(payload: NoNameManualPlotTextApplyPayload) {
  emit('apply-manual-plot-text-hint', payload);
}

function applyManualChapterSummaryHint(payload: NoNameManualChapterSummaryHintApplyPayload) {
  emit('apply-manual-chapter-summary-hint', payload);
}

function applyManualOptionBiasHint(payload: NoNameManualOptionBiasHintApplyPayload) {
  emit('apply-manual-option-bias-hint', payload);
}

function applyManualPlotAugmentationHint(payload: NoNameManualPlotAugmentationHintApplyPayload) {
  emit('apply-manual-plot-augmentation-hint', payload);
}

function humanReviewDecisionKey(traceId: string, requestId: string) {
  return `${traceId}::${requestId}`;
}

function buildTraceReport(
  trace: NoNameTrace | null,
  reviewDecisions: Record<string, NoNameHumanReviewDecision> = {},
) {
  if (!trace) {
    return '';
  }
  const latestProposal = trace.proposals.length > 0
    ? trace.proposals[trace.proposals.length - 1]
    : null;
  const readyCount = trace.proposals.filter((proposal) => proposal.applyable || proposal.status === 'ready').length;
  const graphPath = trace.graphPath.length > 0 ? trace.graphPath.join(' -> ') : '无';
  const protocolEvents = trace.protocolEvents ?? [];
  const controlledReviews = trace.controlledOutputReviews ?? [];
  const humanReviewCount = controlledReviews.filter((review) => review.requiresHumanReview).length;
  const reviewDecisionCounts = controlledReviews.reduce(
    (counts, review) => {
      const decision = reviewDecisions[review.requestId] ?? 'pending';
      counts[decision] += 1;
      return counts;
    },
    {
      pending: 0,
      approvedForHigherApply: 0,
      rejectedForHigherApply: 0,
    } satisfies Record<NoNameHumanReviewDecision, number>,
  );
  const relatedObservations = trace.relatedObservations ?? [];
  const roleContextSummary = relatedObservations.length > 0
    ? relatedObservations
      .map((item) => {
        const roleGoal = item.roleGoal ?? 'no-role-goal';
        const sceneFocus = item.sceneFocus ?? 'no-scene-focus';
        const forbiddenScopes = item.forbiddenScopes?.length
          ? item.forbiddenScopes.join('/')
          : 'none';
        const noteTypeHits = item.noteTypeHits?.length
          ? item.noteTypeHits.join('/')
          : 'none';
        const sourceStats = item.sourceStats?.length
          ? item.sourceStats.map((source) => `${source.source}:${source.count}`).join('/')
          : 'none';
        const tokenBudgetUsed = item.contextTokenBudgetUsed ?? 0;
        const contextSliceStats = item.contextSliceStats?.length
          ? item.contextSliceStats.map((stat) => `${stat.section}:${stat.sourceCount}->${stat.visibleCount}`).join('/')
          : 'none';
        return `${item.role}:${roleGoal}:${sceneFocus}[forbidden=${forbiddenScopes}][notes=${noteTypeHits}][sources=${sourceStats}][tokens=${tokenBudgetUsed}][slice=${contextSliceStats}]`;
      })
      .join(', ')
    : 'none';
  const guardrail = trace.guardrailResult
    ? `${trace.guardrailResult.outcome}${trace.guardrailResult.reason ? ` (${trace.guardrailResult.reason})` : ''}`
    : '无';
  const applyResult = trace.applyResult
    ? `${trace.applyResult.outcome}${trace.applyResult.reason ? ` (${trace.applyResult.reason})` : ''}`
    : '无';
  const lifecycle = summarizeNoNameApplyLifecycle(trace, reviewDecisions);
  const pendingPlotAugmentation = summarizeNoNamePendingPlotAugmentation(trace);
  const applyExecutions = summarizeNoNameApplyExecutions(trace, {
    emptyLabel: 'none',
    rawPrefix: ' [raw=',
    notePrefix: ' (',
  });
  return [
    `Trace: ${trace.traceId}`,
    `Mode: ${trace.mode}`,
    `Turn: ${trace.turnId}`,
    `Graph: ${graphPath}`,
    `Proposals: ${readyCount}/${trace.proposals.length} applyable`,
    `Latest Proposal: ${latestProposal ? `${latestProposal.producerRole}:${latestProposal.kind}:${latestProposal.focus}` : '无'}`,
    `Related Observations: ${relatedObservations.length}`,
    `Role Contexts: ${roleContextSummary}`,
    `Protocol Events: ${protocolEvents.length}`,
    `Controlled Reviews: ${controlledReviews.length} (${humanReviewCount} needs human review)`,
    `Human Review Decisions: ${reviewDecisionCounts.approvedForHigherApply} approved / ${reviewDecisionCounts.rejectedForHigherApply} rejected / ${reviewDecisionCounts.pending} pending`,
    `Guardrail: ${guardrail}`,
    `Apply Result: ${applyResult}`,
    `Apply Lifecycle: ${lifecycle}`,
    `Apply Executions: ${applyExecutions}`,
    `Plot Augmentation: ${pendingPlotAugmentation}`,
    `Fallback: ${trace.fallbackUsed ? 'yes' : 'no'}`,
    `Elapsed: ${trace.elapsedMs} ms`,
  ].join('\n');
}
</script>

<style scoped>
.noname-debug-console-overlay {
  position: fixed;
  inset: 0;
  z-index: 70;
  background: rgba(17, 12, 8, 0.34);
  backdrop-filter: blur(10px);
  padding: 22px;
}

.noname-debug-console-shell {
  height: 100%;
  border-radius: 28px;
  border: 1px solid var(--ink-border-soft);
  background:
    radial-gradient(circle at top right, rgba(215, 173, 92, 0.18), transparent 28%),
    linear-gradient(180deg, rgba(255, 252, 247, 0.96), rgba(249, 244, 235, 0.97));
  box-shadow: 0 30px 80px rgba(22, 14, 8, 0.24);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.noname-debug-console-header {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: flex-start;
  padding: 22px 24px 16px;
  border-bottom: 1px solid color-mix(in srgb, var(--ink-border-soft) 76%, transparent);
}

.noname-debug-console-eyebrow {
  margin: 0;
  font-size: 12px;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  color: var(--ink-text-muted);
}

.noname-debug-console-title {
  margin: 8px 0 0;
  font-size: 30px;
  color: var(--ink-title-color);
}

.noname-debug-console-meta {
  margin: 8px 0 0;
  color: var(--ink-text-muted);
}

.noname-debug-console-actions,
.noname-debug-console-mode-strip {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.noname-debug-console-actions {
  justify-content: flex-end;
}

.noname-debug-console-btn,
.noname-debug-console-mode-btn,
.noname-debug-console-trace-btn {
  border: 1px solid var(--ink-border-accent);
  background: color-mix(in srgb, var(--ink-card-bg) 90%, transparent);
  color: var(--ink-text-primary);
  transition: background-color 180ms ease, border-color 180ms ease, transform 120ms ease;
}

.noname-debug-console-btn {
  border-radius: 12px;
  padding: 10px 14px;
}

.noname-debug-console-btn-primary,
.noname-debug-console-mode-btn.is-active,
.noname-debug-console-trace-btn.is-active {
  background: color-mix(in srgb, var(--ink-accent-main) 18%, var(--ink-card-bg));
  border-color: var(--ink-accent-main);
}

.noname-debug-console-mode-strip {
  padding: 0 24px 18px;
}

.noname-debug-console-mode-btn {
  border-radius: 999px;
  padding: 8px 12px;
}

.noname-debug-console-layout {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 280px minmax(0, 1fr);
  gap: 0;
}

.noname-debug-console-sidebar {
  border-right: 1px solid color-mix(in srgb, var(--ink-border-soft) 76%, transparent);
  padding: 18px 18px 22px;
  overflow-y: auto;
}

.noname-debug-console-main {
  padding: 18px 22px 22px;
  overflow-y: auto;
}

.noname-debug-console-section-title {
  margin: 0 0 12px;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.16em;
  color: var(--ink-text-muted);
}

.noname-debug-console-empty {
  border: 1px dashed var(--ink-border-soft);
  border-radius: 16px;
  padding: 16px;
  color: var(--ink-text-muted);
}

.noname-debug-console-trace-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.noname-debug-console-trace-btn {
  text-align: left;
  border-radius: 16px;
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.noname-debug-console-trace-title {
  font-size: 14px;
}

.noname-debug-console-trace-meta {
  font-size: 12px;
  color: var(--ink-text-muted);
}

.noname-debug-console-btn:disabled {
  cursor: not-allowed;
  opacity: 0.54;
}

.noname-debug-console-report {
  margin-top: 16px;
  border: 1px solid color-mix(in srgb, var(--ink-border-soft) 76%, transparent);
  border-radius: 16px;
  background: color-mix(in srgb, var(--ink-card-bg-soft) 88%, transparent);
  padding: 12px;
}

.noname-debug-console-report-head {
  display: flex;
  justify-content: space-between;
  gap: 10px;
  align-items: center;
}

.noname-debug-console-copy-status {
  color: var(--ink-text-muted);
  font-size: 12px;
}

.noname-debug-console-report-body {
  margin: 10px 0 0;
  max-height: 220px;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-word;
  color: var(--ink-text-primary);
  font-size: 12px;
  line-height: 1.55;
  font-family: "LXGW WenKai", "Noto Serif SC", serif;
}

@media (max-width: 1024px) {
  .noname-debug-console-overlay {
    padding: 10px;
  }

  .noname-debug-console-layout {
    grid-template-columns: 1fr;
  }

  .noname-debug-console-sidebar {
    border-right: 0;
    border-bottom: 1px solid color-mix(in srgb, var(--ink-border-soft) 76%, transparent);
  }
}
</style>
