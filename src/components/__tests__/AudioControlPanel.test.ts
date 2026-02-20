import { mount } from '@vue/test-utils';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import AudioControlPanel from '../AudioControlPanel.vue';

const applyAudioSettingsMock = vi.fn();
const setBgmEnabledMock = vi.fn();
const setMasterVolumeMock = vi.fn();
const setSfxEnabledMock = vi.fn();
const playClickMock = vi.fn();

vi.mock('../../utils/audioSystem', () => ({
  applyAudioSettings: (...args: unknown[]) => applyAudioSettingsMock(...args),
  getAudioSettings: () => ({
    master: 0.55,
    bgmEnabled: true,
    sfxEnabled: true,
  }),
  playClick: () => playClickMock(),
  setBgmEnabled: (enabled: boolean) => setBgmEnabledMock(enabled),
  setMasterVolume: (value: number) => setMasterVolumeMock(value),
  setSfxEnabled: (enabled: boolean) => setSfxEnabledMock(enabled),
}));

describe('AudioControlPanel', () => {
  beforeEach(() => {
    applyAudioSettingsMock.mockClear();
    setBgmEnabledMock.mockClear();
    setMasterVolumeMock.mockClear();
    setSfxEnabledMock.mockClear();
    playClickMock.mockClear();
  });

  it('applies preset volume and toggles mute/restore', async () => {
    const wrapper = mount(AudioControlPanel);

    await wrapper.get('[data-testid="volume-preset-high"]').trigger('click');
    expect(setMasterVolumeMock).toHaveBeenCalledWith(0.8);

    await wrapper.get('[data-testid="volume-toggle-mute"]').trigger('click');
    expect(setMasterVolumeMock).toHaveBeenCalledWith(0);

    await wrapper.get('[data-testid="volume-toggle-mute"]').trigger('click');
    expect(setMasterVolumeMock).toHaveBeenCalledWith(0.8);
  });

  it('toggles bgm and sfx switches', async () => {
    const wrapper = mount(AudioControlPanel);

    await wrapper.get('[data-testid="toggle-bgm-btn"]').trigger('click');
    await wrapper.get('[data-testid="toggle-sfx-btn"]').trigger('click');

    expect(setBgmEnabledMock).toHaveBeenCalledWith(false);
    expect(setSfxEnabledMock).toHaveBeenCalledWith(false);
    expect(playClickMock).toHaveBeenCalledTimes(2);
  });
});
