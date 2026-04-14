import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import GameInfoCenterDialog from '../GameInfoCenterDialog.vue';
import type { NoNameApplyScope } from '../../types/game';

const baseStore = {
  playerCharacter: {
    name: '无名弟子',
    location: 'sect_valley',
  },
  currentScene: {
    location: 'sect_valley',
  },
  plotState: {
    segment_count: 2,
    current_chapter: {
      index: 1,
      title: '第一章',
    },
    last_generation_diagnostics: 'ok',
  },
  isWaitingForInput: true,
  reachableLocationIds: [],
  mapOverview: [],
  isGameInitialized: true,
  gameState: {
    event_history: [],
  },
  noNameTraces: [{
    traceId: 'trace-1',
    sessionId: 'session-1',
    turnId: 'turn-1',
    mode: 'assisted' as const,
    graphPath: ['CollectTurnInput', 'ApplyProposal'],
    capabilityCalls: [],
    proposals: [{
      proposalId: 'proposal-1',
      kind: 'plotCandidate',
      producerRole: 'director',
      title: 'Director提案：山门危机',
      summary: '建议优先观察山门危机',
      focus: '山门危机',
      targetSegment: 'current_turn_tail' as const,
      intendedEffect: '为下一轮低风险输出提供导向',
      rationale: '当前章节冲突正在汇聚',
      labels: ['director', 'assisted_ready'],
      applyScopes: ['diagnostics', 'chapterSummaryHint'] as NoNameApplyScope[],
      status: 'ready' as const,
      applyable: true,
    }],
    proposalTransitionLog: ['proposal-1:ready'],
    applyPlanLog: [{
      order: 1,
      target: 'chapter_summary_hint',
      decision: 'apply',
      priority: 200,
      note: '允许执行 chapter_summary_hint',
    }],
    applyExecutionLog: [{
      target: 'chapter_summary_hint',
      outcome: 'applied',
      note: '已写入章节摘要提示',
    }],
    guardrailResult: {
      outcome: 'accept',
      reason: null,
    },
    applyResult: {
      attempted: true,
      outcome: 'preflight_ready',
      reason: '已通过 assisted apply 预检',
    },
    fallbackUsed: false,
    elapsedMs: 12,
  }],
  error: null,
};

describe('GameInfoCenterDialog', () => {
  it('forwards close/clear-error/travel/mode events', async () => {
    const wrapper = mount(GameInfoCenterDialog, {
      props: {
        isOpen: true,
        gameStore: baseStore,
        playerRealmLabel: '凡人',
        playerCombatPowerLabel: '10',
        chapterProgressLabel: '1 / 第一章',
        chapterInteractionLabel: '1 / 2-3',
        worldLocationList: [],
        recentCombatReview: [],
        travelPending: false,
        isDevMode: true,
        optionSourceLabel: '',
        consistencyRiskScore: null,
      },
      global: {
        stubs: {
          InfoTabsDialog: {
            template: `
              <div>
                <button class="close-btn" @click="$emit('close')" />
                <button class="clear-btn" @click="$emit('clear-error')" />
                <button class="travel-btn" @click="$emit('travel', 'loc_1')" />
                <button class="mode-btn" @click="$emit('set-no-name-mode', 'assisted')" />
              </div>
            `,
          },
        },
      },
    });

    await wrapper.find('.close-btn').trigger('click');
    await wrapper.find('.clear-btn').trigger('click');
    await wrapper.find('.travel-btn').trigger('click');
    await wrapper.find('.mode-btn').trigger('click');

    expect(wrapper.emitted('close')).toBeTruthy();
    expect(wrapper.emitted('clear-error')).toBeTruthy();
    expect(wrapper.emitted('travel')?.[0]).toEqual(['loc_1']);
    expect(wrapper.emitted('set-no-name-mode')?.[0]).toEqual(['assisted']);
  });
});
