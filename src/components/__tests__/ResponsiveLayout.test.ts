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
    expect(classes).toContain('ink-character-panel');
    expect(classes).toContain('max-h-[70vh]');
    expect(classes).toContain('overflow-y-auto');
  });

  it('GameView renders runtime shell component', () => {
    const wrapper = mount(GameView, {
      global: {
        stubs: {
          GameRuntimeView: true,
        },
      },
    });
    expect(wrapper.findComponent({ name: 'GameRuntimeView' }).exists()).toBe(true);
  });

  it('MainMenu keeps menu button structure', () => {
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
    expect(buttonClasses).toContain('menu-btn');
    expect(buttonClasses).toContain('menu-btn-primary');
  });
});


