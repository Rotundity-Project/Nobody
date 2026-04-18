import { mount } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import NoNameDebugConsole from '../NoNameDebugConsole.vue';
import type { NoNameTrace } from '../../types/game';

const traces: NoNameTrace[] = [
  {
    traceId: 'trace-1',
    sessionId: 'session-1',
    turnId: 'turn-1',
    mode: 'observeOnly',
    graphPath: ['CollectTurnInput', 'PlanTurn'],
    capabilityCalls: [],
    proposals: [{
      proposalId: 'proposal-1',
      kind: 'plotCandidate',
      producerRole: 'director',
      title: 'Director提案：初始观察',
      summary: '先收集世界线索',
      focus: '初始观察',
      targetSegment: 'current_turn_tail',
      intendedEffect: '保持观察',
      rationale: '当前信息有限',
      labels: ['director'],
      applyScopes: ['diagnostics'],
      status: 'observed',
      applyable: false,
    }],
    proposalTransitionLog: [],
    applyPlanLog: [],
    applyExecutionLog: [],
    guardrailResult: null,
    applyResult: null,
    fallbackUsed: false,
    elapsedMs: 11,
  },
  {
    traceId: 'trace-2',
    sessionId: 'session-1',
    turnId: 'turn-2',
    mode: 'assisted',
    graphPath: ['CollectTurnInput', 'PlanTurn', 'ApplyProposal'],
    capabilityCalls: [],
    proposals: [{
      proposalId: 'proposal-2',
      kind: 'worldPatchProposal',
      producerRole: 'worldCurator',
      title: 'WorldCurator提案：山门法阵',
      summary: '补全法阵约束',
      focus: '山门法阵',
      targetSegment: 'chapter_summary_tail',
      intendedEffect: '补足世界事实',
      rationale: '山门设定需要锚点',
      labels: ['world_curator'],
      applyScopes: ['diagnostics', 'chapterSummaryHint'],
      status: 'ready',
      applyable: true,
    }],
    proposalTransitionLog: ['proposal-2:ready'],
    applyPlanLog: [{
      order: 1,
      target: 'chapter_summary_hint',
      decision: 'apply',
      priority: 200,
      note: '补足章节锚点',
    }],
    applyExecutionLog: [{
      target: 'chapter_summary_hint',
      outcome: 'applied',
      note: '已完成写入',
    }],
    guardrailResult: { outcome: 'accept', reason: null },
    applyResult: { attempted: true, outcome: 'preflight_ready', reason: '允许应用' },
    protocolEvents: [{
      channel: 'agent',
      from: 'runtime',
      to: 'worldCurator',
      kind: 'taskRequest',
      taskId: 'task-1',
      status: 'queued',
      detail: 'fan-out',
    }],
    controlledOutputReviews: [{
      requestId: 'controlled-output-proposal-2-plot_text_hint',
      requestedKind: 'sceneAugmentation',
      decision: 'needsReview',
      reason: 'plot text hint requires human review before higher-layer apply',
      normalizedKind: 'sceneAugmentation',
      safeApplyScope: 'plotTextHint',
      policyForbiddenScopes: ['finalPlotState', 'canonWorldFact'],
      requiresHumanReview: true,
    }],
    fallbackUsed: false,
    elapsedMs: 28,
  },
];

describe('NoNameDebugConsole', () => {
  it('renders latest trace by default and can switch traces', async () => {
    const wrapper = mount(NoNameDebugConsole, {
      props: {
        isOpen: true,
        traces,
        noNameMode: 'assisted',
        isDevMode: true,
      },
    });

    expect(wrapper.text()).toContain('WorldCurator提案：山门法阵');
    expect(wrapper.text()).toContain('当前模式：assisted');
    expect(wrapper.text()).toContain('Protocol Events: 1');
    expect(wrapper.text()).toContain('Controlled Reviews: 1 (1 needs human review)');
    expect(wrapper.text()).toContain('Human Review Decisions: 0 approved / 0 rejected / 1 pending');
    expect(wrapper.text()).toContain('Apply Lifecycle:');
    expect(wrapper.text()).toContain('Proposals: 1/1 applyable');

    const traceButtons = wrapper.findAll('.noname-debug-console-trace-btn');
    expect(traceButtons).toHaveLength(2);

    await traceButtons[0].trigger('click');
    expect(wrapper.text()).toContain('Director提案：初始观察');
  });

  it('tracks local human review decisions for NeedsReview outputs', async () => {
    const wrapper = mount(NoNameDebugConsole, {
      props: {
        isOpen: true,
        traces,
        noNameMode: 'assisted',
        isDevMode: true,
      },
    });

    expect(wrapper.text()).toContain('等待开发者确认');

    const reviewButtons = wrapper.findAll('.agent-trace-review-btn');
    await reviewButtons
      .find((button) => button.text() === '标记可进入高层 apply 设计')
      ?.trigger('click');

    expect(wrapper.text()).toContain('人工结论：可进入下一阶段 apply 设计');
    expect(wrapper.text()).toContain('Human Review Decisions: 1 approved / 0 rejected / 0 pending');

    await wrapper
      .findAll('.agent-trace-review-btn')
      .find((button) => button.text() === '二次护栏允许')
      ?.trigger('click');
    expect(wrapper.emitted('resolve-second-guardrail')?.[0]).toEqual([{
      traceId: 'trace-2',
      requestId: 'controlled-output-proposal-2-plot_text_hint',
      decision: 'allow',
    }]);
  });

  it('emits close, clear and mode change events', async () => {
    const wrapper = mount(NoNameDebugConsole, {
      props: {
        isOpen: true,
        traces,
        noNameMode: 'observeOnly',
        isDevMode: true,
      },
    });

    await wrapper.findAll('.noname-debug-console-btn')[1].trigger('click');
    await wrapper.findAll('.noname-debug-console-mode-btn')[2].trigger('click');
    await wrapper.find('.noname-debug-console-btn-primary').trigger('click');

    expect(wrapper.emitted('clear-traces')).toBeTruthy();
    expect(wrapper.emitted('set-no-name-mode')?.[0]).toEqual(['assisted']);
    expect(wrapper.emitted('close')).toBeTruthy();
  });

  it('copies the selected trace report', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, 'clipboard', {
      configurable: true,
      value: { writeText },
    });

    const wrapper = mount(NoNameDebugConsole, {
      props: {
        isOpen: true,
        traces,
        noNameMode: 'assisted',
        isDevMode: true,
      },
    });

    await wrapper.findAll('.noname-debug-console-btn')[0].trigger('click');

    expect(writeText).toHaveBeenCalledWith(expect.stringContaining('Trace: trace-2'));
    expect(writeText).toHaveBeenCalledWith(expect.stringContaining('Protocol Events: 1'));
    expect(writeText).toHaveBeenCalledWith(expect.stringContaining('Controlled Reviews: 1'));
    expect(writeText).toHaveBeenCalledWith(expect.stringContaining('Apply Lifecycle:'));
    expect(wrapper.text()).toContain('摘要已复制');
  });
});
