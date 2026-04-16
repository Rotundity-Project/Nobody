import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import AgentTracePanel from '../AgentTracePanel.vue';
import type { NoNameTrace } from '../../types/game';

const trace: NoNameTrace = {
  traceId: 'trace-2',
  sessionId: 'session-1',
  turnId: 'turn-8',
  mode: 'assisted',
  graphPath: ['CollectTurnInput', 'PlanTurn', 'ApplyProposal'],
  capabilityCalls: [
    { capabilityId: 'tool.generate_plot_candidate', callKind: 'tool', status: 'ok' },
  ],
  proposals: [{
    proposalId: 'proposal-2',
    kind: 'plotCandidate',
    producerRole: 'director',
    title: 'Director提案：山门危机',
    summary: '建议优先观察山门危机',
    focus: '山门危机',
    targetSegment: 'current_turn_tail',
    intendedEffect: '对当前回合尾段提供轻量导向',
    rationale: '当前章节冲突正在汇聚',
    suggestedAction: '继续低风险 assisted apply',
    labels: ['director'],
    applyScopes: ['diagnostics', 'plotTextHint'],
    status: 'applied',
    applyable: true,
  }],
  proposalTransitionLog: ['proposal-2:ready', 'proposal-2:applied'],
  applyPlanLog: [{
    order: 1,
    target: 'plot_text_hint',
    decision: 'apply',
    priority: 300,
    note: '允许执行 plot_text_hint',
  }],
  applyExecutionLog: [{
    target: 'plot_text_hint',
    outcome: 'applied',
    note: '已插入正文提示',
  }],
  relatedObservations: [{
    role: 'worldCurator',
    actionSummary: '玩家返回山门',
    focus: '山门法阵',
    rationale: '需要补齐世界设定锚点',
    proposal: {
      proposalId: 'proposal-world-1',
      kind: 'worldPatchProposal',
      producerRole: 'worldCurator',
      title: 'WorldCurator提案：山门法阵',
      summary: '补齐法阵约束',
      focus: '山门法阵',
      targetSegment: 'chapter_summary_tail',
      intendedEffect: '补足世界事实',
      rationale: '设定缺口明显',
      labels: ['worldCurator'],
      applyScopes: ['diagnostics', 'chapterSummaryHint'],
      status: 'observed',
      applyable: false,
    },
  }],
  protocolEvents: [{
    channel: 'agent',
    from: 'director',
    to: 'world_curator',
    kind: 'delegation',
    taskId: 'turn-8-world_curator-observe',
    status: 'running',
    detail: 'turn-8-director-observe',
  }],
  guardrailResult: {
    outcome: 'accept',
    reason: null,
  },
  applyResult: {
    attempted: true,
    outcome: 'preflight_ready',
    reason: '已通过 assisted 预检',
  },
  fallbackUsed: false,
  elapsedMs: 21,
};

describe('AgentTracePanel', () => {
  it('renders proposal, plan, execution and summary fields', () => {
    const wrapper = mount(AgentTracePanel, {
      props: {
        trace,
        selectedIndex: 1,
        totalCount: 3,
        activeMode: 'assisted',
      },
    });

    expect(wrapper.text()).toContain('trace-2');
    expect(wrapper.text()).toContain('Director提案：山门危机');
    expect(wrapper.text()).toContain('current_turn_tail');
    expect(wrapper.text()).toContain('plot_text_hint');
    expect(wrapper.text()).toContain('preflight_ready');
    expect(wrapper.text()).toContain('未回退');
    expect(wrapper.text()).toContain('ApplyProposal');
    expect(wrapper.text()).toContain('协作观察');
    expect(wrapper.text()).toContain('WorldCurator提案：山门法阵');
    expect(wrapper.text()).toContain('协议事件');
    expect(wrapper.text()).toContain('agent · delegation · running');
  });

  it('shows empty state when trace is missing', () => {
    const wrapper = mount(AgentTracePanel, {
      props: {
        trace: null,
      },
    });

    expect(wrapper.text()).toContain('暂无可展示的 NoName Trace');
  });
});
