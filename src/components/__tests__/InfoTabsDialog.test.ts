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
});
