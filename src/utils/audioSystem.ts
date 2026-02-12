export interface AudioSettings {
  master: number;
  bgmEnabled: boolean;
  sfxEnabled: boolean;
}

const STORAGE_KEY = 'nobody_audio_settings';

let audioContext: AudioContext | null = null;
let masterGain: GainNode | null = null;
let bgmGain: GainNode | null = null;
let sfxGain: GainNode | null = null;
let bgmOscillators: OscillatorNode[] = [];

const defaultSettings: AudioSettings = {
  master: 0.55,
  bgmEnabled: true,
  sfxEnabled: true,
};

const ensureContext = () => {
  if (typeof window === 'undefined') return null;
  const AudioContextCtor =
    (window as { AudioContext?: typeof AudioContext; webkitAudioContext?: typeof AudioContext })
      .AudioContext ||
    (window as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
  if (!AudioContextCtor) {
    return null;
  }
  if (!audioContext) {
    audioContext = new AudioContextCtor();
    masterGain = audioContext.createGain();
    bgmGain = audioContext.createGain();
    sfxGain = audioContext.createGain();

    bgmGain.gain.value = 0.15;
    sfxGain.gain.value = 0.4;

    bgmGain.connect(masterGain);
    sfxGain.connect(masterGain);
    masterGain.connect(audioContext.destination);
  }
  return audioContext;
};

const resumeContext = () => {
  if (!audioContext) return;
  if (audioContext.state === 'suspended') {
    void audioContext.resume();
  }
};

const persistSettings = (settings: AudioSettings) => {
  if (typeof window === 'undefined') return;
  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
};

export const getAudioSettings = (): AudioSettings => {
  if (typeof window === 'undefined') {
    return { ...defaultSettings };
  }
  const raw = window.localStorage.getItem(STORAGE_KEY);
  if (!raw) return { ...defaultSettings };
  try {
    const parsed = JSON.parse(raw) as Partial<AudioSettings>;
    return {
      master: typeof parsed.master === 'number' ? parsed.master : defaultSettings.master,
      bgmEnabled:
        typeof parsed.bgmEnabled === 'boolean'
          ? parsed.bgmEnabled
          : defaultSettings.bgmEnabled,
      sfxEnabled:
        typeof parsed.sfxEnabled === 'boolean'
          ? parsed.sfxEnabled
          : defaultSettings.sfxEnabled,
    };
  } catch {
    return { ...defaultSettings };
  }
};

export const applyAudioSettings = (settings: AudioSettings) => {
  setMasterVolume(settings.master);
  setSfxEnabled(settings.sfxEnabled);
  setBgmEnabled(settings.bgmEnabled);
};

export const setMasterVolume = (value: number) => {
  ensureContext();
  if (masterGain) {
    masterGain.gain.value = Math.min(1, Math.max(0, value));
  }
  persistSettings({
    ...getAudioSettings(),
    master: value,
  });
};

export const setBgmEnabled = (enabled: boolean) => {
  ensureContext();
  if (enabled) {
    startBgm();
  } else {
    stopBgm();
  }
  persistSettings({
    ...getAudioSettings(),
    bgmEnabled: enabled,
  });
};

export const setSfxEnabled = (enabled: boolean) => {
  ensureContext();
  if (sfxGain) {
    sfxGain.gain.value = enabled ? 0.4 : 0.0;
  }
  persistSettings({
    ...getAudioSettings(),
    sfxEnabled: enabled,
  });
};

export const playClick = () => {
  const settings = getAudioSettings();
  if (!settings.sfxEnabled) return;

  const ctx = ensureContext();
  if (!ctx || !sfxGain) return;

  resumeContext();

  const osc = ctx.createOscillator();
  const gain = ctx.createGain();

  osc.type = 'triangle';
  osc.frequency.value = 620;

  const now = ctx.currentTime;
  gain.gain.setValueAtTime(0.0001, now);
  gain.gain.exponentialRampToValueAtTime(0.18, now + 0.02);
  gain.gain.exponentialRampToValueAtTime(0.0001, now + 0.12);

  osc.connect(gain);
  gain.connect(sfxGain);

  osc.start(now);
  osc.stop(now + 0.14);
};

const startBgm = () => {
  const ctx = ensureContext();
  if (!ctx || !bgmGain) return;
  if (bgmOscillators.length > 0) return;

  resumeContext();

  const frequencies = [130.81, 196.0, 261.63];
  bgmOscillators = frequencies.map((freq) => {
    const osc = ctx.createOscillator();
    osc.type = 'sine';
    osc.frequency.value = freq;
    const gain = ctx.createGain();
    gain.gain.value = 0.12;
    osc.connect(gain);
    gain.connect(bgmGain);
    osc.start();
    return osc;
  });
};

const stopBgm = () => {
  bgmOscillators.forEach((osc) => {
    try {
      osc.stop();
      osc.disconnect();
    } catch {
      // ignore stopping errors
    }
  });
  bgmOscillators = [];
};
