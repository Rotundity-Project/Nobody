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
import type { NoNameTrace } from '../types/game';

const props = withDefaults(defineProps<{
  trace: NoNameTrace | null;
  selectedIndex?: number;
  totalCount?: number;
  activeMode?: string;
}>(), {
  selectedIndex: 0,
  totalCount: 0,
  activeMode: '',
});

const latestProposal = computed(() => {
  if (!props.trace || props.trace.proposals.length === 0) {
    return null;
  }
  return props.trace.proposals[props.trace.proposals.length - 1];
});

const relatedObservations = computed(() => props.trace?.relatedObservations ?? []);
const protocolEvents = computed(() => props.trace?.protocolEvents ?? []);
const controlledOutputReviews = computed(() => props.trace?.controlledOutputReviews ?? []);

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

.agent-trace-row {
  display: grid;
  gap: 4px;
  border-radius: 14px;
  border: 1px solid color-mix(in srgb, var(--ink-border-soft) 72%, transparent);
  background: color-mix(in srgb, var(--ink-card-bg) 90%, transparent);
  padding: 10px 12px;
}

@media (max-width: 900px) {
  .agent-trace-grid {
    grid-template-columns: 1fr;
  }
}
</style>
