import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import InfoTabsDialog from '../InfoTabsDialog.vue';

describe('InfoTabsDialog', () => {
  it('normalizes map location labels to Chinese-friendly text', async () => {
    const wrapper = mount(InfoTabsDialog, {
      props: {
        isOpen: true,
        playerName: '无名弟子',
        playerRealm: '炼气 (1-2)',
        playerCombatPower: '123',
        playerLocation: 'sect_valley',
        chapterProgress: '1 / 第一章',
        chapterInteraction: '1 / 2-4',
        segmentCount: 2,
        isWaitingForInput: true,
        worldLocations: [
          { id: 'sect_valley', name: 'sect_valley', spiritual_energy: 0.7 },
        ],
        reachableLocationIds: ['sect_valley'],
        mapOverview: [],
        recentCombatExplanations: [],
        currentLocationId: 'sect_valley',
        currentLocationLabel: 'sect_valley',
        isTraveling: false,
        isGameRunning: true,
        eventCount: 0,
        isDevMode: false,
        debugChapter: '1 / 第一章',
        debugOptionSource: 'n/a',
        debugOptionHint: '',
        debugRiskScore: null,
        debugDiagnostics: '',
        systemError: null,
      },
    });

    const mapTab = wrapper.findAll('button').find((btn) => btn.text().includes('地图行程'));
    expect(mapTab).toBeTruthy();
    await mapTab!.trigger('click');

    expect(wrapper.text()).toContain('宗门外谷');
    expect(wrapper.text()).not.toContain('id: sect_valley');
  });

  it('renders structured noname debug sections in dev mode', async () => {
    const wrapper = mount(InfoTabsDialog, {
      props: {
        isOpen: true,
        playerName: '无名弟子',
        playerRealm: '炼气 (1-2)',
        playerCombatPower: '123',
        playerLocation: 'sect_valley',
        chapterProgress: '1 / 第一章',
        chapterInteraction: '1 / 2-4',
        segmentCount: 2,
        isWaitingForInput: true,
        worldLocations: [
          { id: 'sect_valley', name: 'sect_valley', spiritual_energy: 0.7 },
        ],
        reachableLocationIds: ['sect_valley'],
        mapOverview: [],
        recentCombatExplanations: [],
        currentLocationId: 'sect_valley',
        currentLocationLabel: 'sect_valley',
        isTraveling: false,
        isGameRunning: true,
        eventCount: 0,
        isDevMode: true,
        debugChapter: '1 / 第一章',
        debugOptionSource: 'llm_structured',
        debugOptionHint: '来源说明',
        debugRiskScore: 2,
        debugDiagnostics: '诊断文本',
        debugNoNameTrace: 'trace text',
        noNameMode: 'assisted',
        debugNoNameProposalTitle: 'Director提案：山门危机',
        debugNoNameProposalStatus: 'applied',
        debugNoNameTargetSegment: 'current_turn_tail',
        debugNoNameIntendedEffect: '对当前回合尾段提供轻量导向',
        debugNoNameApplyOutcome: 'applied_summary_option_bias_and_plot_text',
        debugNoNameApplyReason: '已进入低风险输出层',
        debugNoNameScopes: ['diagnostics', 'plot_text_hint'],
        debugNoNameTransitions: ['proposal-1:ready', 'proposal-1:applied:plot_text_hint'],
        debugNoNamePlans: [
          { order: 1, target: 'plot_text_hint', decision: 'apply', priority: 300, note: '允许执行 plot_text_hint' },
        ],
        debugNoNameExecutions: [
          { target: 'plot_text_hint', outcome: 'applied', note: '已将提案提示插入正文' },
        ],
        systemError: null,
      },
    });

    const debugTab = wrapper.findAll('button').find((btn) => btn.text().includes('调试上下文'));
    expect(debugTab).toBeTruthy();
    await debugTab!.trigger('click');

    expect(wrapper.text()).toContain('最新提案');
    expect(wrapper.text()).toContain('状态迁移');
    expect(wrapper.text()).toContain('应用计划');
    expect(wrapper.text()).toContain('应用执行');
    expect(wrapper.text()).toContain('Director提案：山门危机');
    expect(wrapper.text()).toContain('current_turn_tail');
    expect(wrapper.text()).toContain('对当前回合尾段提供轻量导向');
    expect(wrapper.text()).toContain('diagnostics / plot_text_hint');
    expect(wrapper.text()).toContain('#1');
    expect(wrapper.text()).toContain('P300');
    expect(wrapper.text()).toContain('允许执行 plot_text_hint');
    expect(wrapper.text()).toContain('plot_text_hint');
  });
});
