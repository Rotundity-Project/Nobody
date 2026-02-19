import { mount, shallowMount } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import CharacterPanel from '../CharacterPanel.vue';
import GameView from '../GameView.vue';
import MainMenu from '../MainMenu.vue';

vi.mock('vue-router', () => ({
  useRouter: () => ({
    push: vi.fn(),
  }),
}));

vi.mock('../../stores/gameStore', () => ({
  useGameStore: () => ({
    isGameInitialized: false,
    isWaitingForInput: false,
    availableOptions: [],
    currentScene: null,
    playerCharacter: null,
    plotState: null,
    gameState: null,
    reachableLocationIds: [],
    mapOverview: [],
    error: null,
    clearError: vi.fn(),
  }),
}));

describe('responsive layout classes', () => {
  it('CharacterPanel keeps constrained panel layout', () => {
    const wrapper = shallowMount(CharacterPanel, {
      props: {
        character: null,
      },
    });
    const classes = wrapper.classes();
    expect(classes).toContain('panel-surface');
    expect(classes).toContain('max-h-[70vh]');
    expect(classes).toContain('overflow-y-auto');
  });

  it('GameView uses responsive header layout', () => {
    const wrapper = shallowMount(GameView, {
      global: {
        stubs: {
          CharacterPanel: true,
          SaveLoadDialog: true,
          LLMConfigDialog: true,
          NovelExporter: true,
          LoadingStatePanel: true,
        },
      },
    });
    const classes = wrapper.classes();
    expect(classes).toContain('flex');
    expect(classes).toContain('flex-col');
    const hasResponsiveHeader = wrapper.findAll('div').some((node) => {
      const nodeClasses = node.classes();
      return nodeClasses.includes('sm:flex-row')
        && nodeClasses.includes('sm:items-center')
        && nodeClasses.includes('sm:justify-between');
    });
    expect(hasResponsiveHeader).toBe(true);
  });

  it('MainMenu buttons have responsive width classes', () => {
    const wrapper = mount(MainMenu, {
      global: {
        stubs: {
          LLMConfigDialog: true,
        },
      },
    });
    const buttons = wrapper.findAll('button');
    expect(buttons.length).toBeGreaterThan(0);
    const buttonClasses = buttons[0]?.classes() ?? [];
    expect(buttonClasses).toContain('w-full');
    expect(buttonClasses).toContain('sm:w-64');
  });
});


