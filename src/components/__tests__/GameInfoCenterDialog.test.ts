import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import GameInfoCenterDialog from '../GameInfoCenterDialog.vue';

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
  error: null,
};

describe('GameInfoCenterDialog', () => {
  it('forwards close/clear-error/travel events', async () => {
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
              </div>
            `,
          },
        },
      },
    });

    await wrapper.find('.close-btn').trigger('click');
    await wrapper.find('.clear-btn').trigger('click');
    await wrapper.find('.travel-btn').trigger('click');

    expect(wrapper.emitted('close')).toBeTruthy();
    expect(wrapper.emitted('clear-error')).toBeTruthy();
    expect(wrapper.emitted('travel')?.[0]).toEqual(['loc_1']);
  });
});
