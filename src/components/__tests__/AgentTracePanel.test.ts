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
    roleGoal: 'Maintain world facts, scene constraints, and canon anchors.',
    sceneFocus: '山门法阵',
    forbiddenScopes: ['Must not decide NPC private intent.', 'Must not choose the main plot beat alone.'],
    noteTypeHits: ['goal: Hold Gate', 'foreshadowing: Broken Ward'],
    sourceStats: [{ source: 'hard_facts', count: 2 }, { source: 'chapter_summaries', count: 1 }],
    contextTokenBudgetUsed: 42,
    contextSliceStats: [{ section: 'worldFacts', sourceCount: 4, visibleCount: 3 }],
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
    expect(wrapper.text()).toContain('Trace 快读');
    expect(wrapper.text()).toContain('提案');
    expect(wrapper.text()).toContain('复核');
    expect(wrapper.text()).toContain('人工');
    expect(wrapper.text()).toContain('执行');
    expect(wrapper.text()).toContain('护栏');
    expect(wrapper.text()).toContain('Director提案：山门危机');
    expect(wrapper.text()).toContain('current_turn_tail');
    expect(wrapper.text()).toContain('plot_text_hint');
    expect(wrapper.text()).toContain('preflight_ready');
    expect(wrapper.text()).toContain('未回退');
    expect(wrapper.text()).toContain('ApplyProposal');
    expect(wrapper.text()).toContain('应用生命周期');
    expect(wrapper.text()).toContain('Lifecycle Checkpoints: 1.Human Review=pending');
    expect(wrapper.text()).toContain('提案阶段');
    expect(wrapper.text()).toContain('低风险输出');
    expect(wrapper.text()).toContain('协作观察');
    expect(wrapper.text()).toContain('WorldCurator提案：山门法阵');
    expect(wrapper.text()).toContain('角色上下文');
    expect(wrapper.text()).toContain('Maintain world facts');
    expect(wrapper.text()).toContain('角色禁区');
    expect(wrapper.text()).toContain('Must not decide NPC private intent.');
    expect(wrapper.text()).toContain('笔记命中');
    expect(wrapper.text()).toContain('goal: Hold Gate');
    expect(wrapper.text()).toContain('上下文来源');
    expect(wrapper.text()).toContain('hard_facts:2');
    expect(wrapper.text()).toContain('token=42');
    expect(wrapper.text()).toContain('裁剪差异');
    expect(wrapper.text()).toContain('worldFacts:4->3');
    expect(wrapper.text()).toContain('协议事件');
    expect(wrapper.text()).toContain('agent · delegation · running');
    expect(wrapper.text()).toContain('受控输出复核');
    expect(wrapper.text()).toContain('sceneAugmentation · plotTextHint · needsReview');
    expect(wrapper.text()).toContain('需要人工复核');
    expect(wrapper.text()).toContain('策略禁区：finalPlotState / canonWorldFact');
    expect(wrapper.text()).toContain('等待开发者确认');
  });

  it('shows pending plot augmentation summary in the overview', () => {
    const wrapper = mount(AgentTracePanel, {
      props: {
        trace: {
          ...trace,
          proposals: [{
            ...trace.proposals[0],
            applyScopes: ['diagnostics', 'plotAugmentationHint'],
          }],
          applyExecutionLog: [
            ...(trace.applyExecutionLog ?? []),
            {
              target: 'plot_augmentation_hint',
              outcome: 'pending_plot_augmentation_consumed',
              note: 'pending plot augmentation consumed after plot_engine generation; count=1',
            },
          ],
        },
      },
    });

    expect(wrapper.text()).toContain('剧情增强提示');
    expect(wrapper.text()).toContain('已消费');
    expect(wrapper.text()).toContain('count=1');
    expect(wrapper.text()).toContain('原始记录：plot_augmentation_hint / pending_plot_augmentation_consumed');
  });

  it('emits a human review decision without applying plot text', async () => {
    const wrapper = mount(AgentTracePanel, {
      props: {
        trace,
        reviewDecisions: {
          'controlled-output-proposal-2-plot_text_hint': 'pending',
        },
      },
    });

    const buttons = wrapper.findAll('.agent-trace-review-btn');
    expect(buttons.map((button) => button.text())).toContain('标记可进入高层 apply 设计');

    await buttons.find((button) => button.text() === '标记可进入高层 apply 设计')?.trigger('click');

    expect(wrapper.emitted('mark-controlled-output-review')?.[0]).toEqual([{
      traceId: 'trace-2',
      requestId: 'controlled-output-proposal-2-plot_text_hint',
      decision: 'approvedForHigherApply',
    }]);

    await wrapper.setProps({
      reviewDecisions: {
        'controlled-output-proposal-2-plot_text_hint': 'approvedForHigherApply',
      },
    });
    expect(wrapper.text()).toContain('人工结论：可进入下一阶段 apply 设计');
    expect(wrapper.text()).toContain('重置待复核');
    expect(wrapper.text()).toContain('二次护栏允许');

    await wrapper
      .findAll('.agent-trace-review-btn')
      .find((button) => button.text() === '二次护栏允许')
      ?.trigger('click');
    expect(wrapper.emitted('resolve-second-guardrail')?.[0]).toEqual([{
      traceId: 'trace-2',
      requestId: 'controlled-output-proposal-2-plot_text_hint',
      decision: 'allow',
    }]);

    await wrapper.setProps({
      trace: {
        ...trace,
        proposalTransitionLog: [
          ...(trace.proposalTransitionLog ?? []),
          'controlled-output-proposal-2-plot_text_hint:second_guardrail:allow',
        ],
        applyExecutionLog: [
          ...(trace.applyExecutionLog ?? []),
          {
            target: 'plot_text_hint',
            outcome: 'second_guardrail_allowed',
            note: '允许进入显式人工 apply',
          },
        ],
      },
      manualApplySegment: {
        chapterIndex: 1,
        segmentIndex: 0,
        text: '你看见山门风声渐紧。',
      },
    });
    expect(wrapper.text()).toContain('显式人工写入正文提示');
    expect(wrapper.text()).toContain('人工写入');
    expect(wrapper.text()).toContain('等待显式命令');
    expect(wrapper.text()).toContain('人工写入预览');
    expect(wrapper.text()).toContain('写入前');
    expect(wrapper.text()).toContain('写入后');
    expect(wrapper.text()).toContain('【NoName】重点关注：山门危机');
    await wrapper
      .findAll('.agent-trace-review-btn')
      .find((button) => button.text().includes('显式人工写入正文提示'))
      ?.trigger('click');
    expect(wrapper.emitted('apply-manual-plot-text-hint')?.[0]).toEqual([{
      traceId: 'trace-2',
      requestId: 'controlled-output-proposal-2-plot_text_hint',
    }]);
  });

  it('disables manual plot text apply when the current segment already has a NoName marker', async () => {
    const wrapper = mount(AgentTracePanel, {
      props: {
        trace: {
          ...trace,
          proposalTransitionLog: [
            ...(trace.proposalTransitionLog ?? []),
            'controlled-output-proposal-2-plot_text_hint:second_guardrail:allow',
          ],
        },
        manualApplySegment: {
          chapterIndex: 1,
          segmentIndex: 0,
          text: '【NoName】重点关注：山门危机\n\n你看见山门风声渐紧。',
        },
      },
    });

    expect(wrapper.text()).toContain('疑似已包含 NoName 标记');
    const button = wrapper
      .findAll('.agent-trace-review-btn')
      .find((item) => item.text().includes('显式人工写入正文提示'));
    expect(button?.attributes('disabled')).toBeDefined();

    await button?.trigger('click');
    expect(wrapper.emitted('apply-manual-plot-text-hint')).toBeUndefined();
  });

  it('emits manual chapter summary and option bias apply events after second guardrail allow', async () => {
    const chapterSummaryTrace: NoNameTrace = {
      ...trace,
      proposals: [{
        ...trace.proposals[0],
        proposalId: 'proposal-summary',
        focus: '山门危机',
        targetSegment: 'chapter_summary_tail',
        applyScopes: ['chapterSummaryHint'],
      }],
      controlledOutputReviews: [{
        requestId: 'controlled-output-proposal-summary-chapter_summary_hint',
        requestedKind: 'recapNote',
        decision: 'needsReview',
        reason: 'summary hint requires review',
        normalizedKind: 'recapNote',
        safeApplyScope: 'chapterSummaryHint',
        policyForbiddenScopes: ['finalPlotState'],
        requiresHumanReview: true,
      }],
      proposalTransitionLog: [
        'controlled-output-proposal-summary-chapter_summary_hint:second_guardrail:allow',
      ],
      applyExecutionLog: [{
        target: 'chapterSummaryHint',
        outcome: 'second_guardrail_allowed',
        note: 'allowed',
      }],
    };
    const wrapper = mount(AgentTracePanel, {
      props: {
        trace: chapterSummaryTrace,
        manualApplySummary: {
          chapterIndex: 1,
          summary: '已有摘要',
        },
      },
    });

    expect(wrapper.text()).toContain('人工写入预览：将写入第 1 章摘要');
    const summaryButtons = wrapper.findAll('.agent-trace-review-btn');
    await summaryButtons[summaryButtons.length - 1]?.trigger('click');
    expect(wrapper.emitted('apply-manual-chapter-summary-hint')?.[0]).toEqual([{
      traceId: 'trace-2',
      requestId: 'controlled-output-proposal-summary-chapter_summary_hint',
    }]);

    await wrapper.setProps({
      trace: {
        ...chapterSummaryTrace,
        proposals: [{
          ...chapterSummaryTrace.proposals[0],
          proposalId: 'proposal-option',
          focus: '暗洞',
          targetSegment: 'current_turn_tail',
          applyScopes: ['optionBiasHint'],
        }],
        controlledOutputReviews: [{
          requestId: 'controlled-output-proposal-option-option_bias_hint',
          requestedKind: 'intermediateNarrativeHint',
          decision: 'needsReview',
          reason: 'option bias requires review',
          normalizedKind: 'intermediateNarrativeHint',
          safeApplyScope: 'optionBiasHint',
          policyForbiddenScopes: ['finalPlotState'],
          requiresHumanReview: true,
        }],
        proposalTransitionLog: [
          'controlled-output-proposal-option-option_bias_hint:second_guardrail:allow',
        ],
        applyExecutionLog: [{
          target: 'optionBiasHint',
          outcome: 'second_guardrail_allowed',
          note: 'allowed',
        }],
      },
      manualApplyDiagnostics: {
        chapterIndex: 1,
        diagnostics: 'base diagnostics',
      },
    });
    expect(wrapper.text()).toContain('人工写入预览：将写入第 1 章诊断提示');
    const optionButtons = wrapper.findAll('.agent-trace-review-btn');
    await optionButtons[optionButtons.length - 1]?.trigger('click');
    expect(wrapper.emitted('apply-manual-option-bias-hint')?.[0]).toEqual([{
      traceId: 'trace-2',
      requestId: 'controlled-output-proposal-option-option_bias_hint',
    }]);
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
