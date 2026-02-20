import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import GameTopBar from '../GameTopBar.vue';

describe('GameTopBar', () => {
  it('renders title and emits back', async () => {
    const wrapper = mount(GameTopBar, {
      props: {
        isSystemMenuOpen: false,
        showAudioPanel: false,
        isGameInitialized: true,
      },
      global: {
        stubs: {
          SystemCenterMenu: true,
          QuickActionsBar: true,
        },
      },
    });

    expect(wrapper.text()).toContain('Nobody');
    const backButton = wrapper.find('button[title="返回"]');
    await backButton.trigger('click');
    expect(wrapper.emitted('back')).toBeTruthy();
  });

  it('forwards menu and quick action events', async () => {
    const wrapper = mount(GameTopBar, {
      props: {
        isSystemMenuOpen: true,
        showAudioPanel: true,
        isGameInitialized: true,
      },
      global: {
        stubs: {
          SystemCenterMenu: {
            template: `
              <div>
                <button class="menu-btn" @click="$emit('toggle-menu')">menu</button>
                <button class="llm-btn" @click="$emit('open-llm')">llm</button>
              </div>
            `,
          },
          QuickActionsBar: {
            template: `
              <div>
                <button class="save-btn" @click="$emit('open-save')">save</button>
              </div>
            `,
          },
        },
      },
    });

    await wrapper.find('.menu-btn').trigger('click');
    await wrapper.find('.llm-btn').trigger('click');
    await wrapper.find('.save-btn').trigger('click');

    expect(wrapper.emitted('toggle-menu')).toBeTruthy();
    expect(wrapper.emitted('open-llm')).toBeTruthy();
    expect(wrapper.emitted('open-save')).toBeTruthy();
  });
});
