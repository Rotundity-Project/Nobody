<template>
  <div class="script-shell min-h-screen px-4 py-8 sm:px-6 sm:py-10" :class="activeThemeClass">
    <div class="script-paper mx-auto w-full max-w-[1200px] rounded-2xl p-5 sm:p-8">
      <header class="script-header">
        <div>
          <h1 class="script-main-title">NOBODY</h1>
          <p class="script-cn-subtitle">小人物</p>
          <p class="script-sub-title">CHOOSE YOUR PATH · 选择剧本类型</p>
        </div>
        <button
          type="button"
          class="script-seal"
          title="轻触印章，聆听天机"
          @click="handleSealClick"
        >
          <span>择道</span>
        </button>
      </header>
      <p v-if="sealWhisper" class="script-seal-whisper">{{ sealWhisper }}</p>

      <p class="script-flow-hint">{{ flowHint }}</p>

      <section class="mt-4" data-testid="script-type-container">
        <div class="script-grid">
          <button
            v-for="scriptType in scriptTypes"
            :key="scriptType.type"
            type="button"
            class="script-type-card"
            :class="{
              'script-type-card-active': selectedType === scriptType.type,
              'script-type-card-disabled': !scriptType.available,
            }"
            :disabled="!scriptType.available || isLoading"
            :data-testid="`script-type-${scriptType.type}`"
            @click="scriptType.available && selectScriptType(scriptType.type)"
          >
            <div class="script-type-head">
              <span class="script-type-icon" aria-hidden="true">{{ scriptType.icon }}</span>
              <h3 class="script-type-title">{{ scriptType.title }}</h3>
            </div>
            <p class="script-type-desc">{{ scriptType.description }}</p>
            <p class="script-type-action">
              <span class="script-positive">{{ scriptType.actionVerb }}</span>{{ scriptType.actionTail }}
            </p>
            <p v-if="!scriptType.available" class="script-type-soon">即将推出</p>
          </button>
        </div>
        <p class="script-tip">选择一种方式开始你的修仙之旅。</p>
      </section>

      <section v-if="showCharacterSelect" class="script-card mt-4">
        <h3 class="script-label">选择主角</h3>
        <p class="script-help mb-2">从小说中选择一个角色作为玩家。</p>
        <div class="script-radio-list">
          <label
            v-for="character in novelCharacters"
            :key="character"
            class="script-radio-item"
          >
            <input
              type="radio"
              name="novel-character"
              :value="character"
              v-model="selectedCharacter"
            />
            <span>{{ character }}</span>
          </label>
        </div>
      </section>

      <div v-if="isLoading && isRandomGenerating" class="mt-4 script-card">
        <div class="taiji-loader-wrap">
          <div class="taiji-loader-stage" aria-hidden="true">
            <div class="taiji-loader">
              <span class="taiji-eye taiji-eye-yang"></span>
              <span class="taiji-eye taiji-eye-yin"></span>
            </div>
          </div>
          <div class="text-left">
            <p class="script-label">{{ loadingMessage }}</p>
            <p class="script-help">请稍候，阴阳轮转，正在推演修仙世界。</p>
            <p v-if="loadingProgressText" class="script-help">{{ loadingProgressText }}</p>
          </div>
        </div>
      </div>

      <div v-else-if="isLoading" class="mt-4">
        <LoadingIndicator
          :message="loadingMessage"
          detail="请稍候，正在处理请求"
          :progress="loadingProgress"
          :progress-text="loadingProgressText"
          size="lg"
        />
      </div>

      <StatusBanner
        v-if="error"
        class="mt-4"
        kind="error"
        title="操作失败"
        :message="error"
      />

      <footer class="script-footer mt-6">
        <button
          type="button"
          data-testid="back-btn"
          :disabled="isLoading"
          class="script-btn"
          @click="handleBack"
        >
          返回
        </button>
        <button
          type="button"
          data-testid="confirm-script-btn"
          :disabled="isLoading || !canConfirm"
          class="script-btn script-btn-primary"
          @click="handleConfirm"
        >
          {{ confirmButtonLabel }}
        </button>
      </footer>
      <p class="script-watermark" aria-hidden="true">小人物</p>
    </div>

    <div
      v-if="showRandomProfilePanel"
      class="script-overlay"
      @click.self="showRandomProfilePanel = false"
    >
      <section class="script-dialog">
        <header class="script-dialog-header">
          <h3 class="script-label">随机剧本自定义</h3>
          <button
            type="button"
            class="script-btn"
            @click="showRandomProfilePanel = false"
          >
            关闭
          </button>
        </header>
        <div class="script-dialog-body">
          <div class="script-card">
            <label class="script-label" for="profile-player-name">主角姓名</label>
            <input
              id="profile-player-name"
              v-model="randomProfile.playerName"
              type="text"
              maxlength="20"
              placeholder="默认：无名弟子"
              class="script-input"
            />
            <p class="script-help">用于覆盖随机剧本默认主角名。</p>
          </div>
          <div class="script-card">
            <label class="script-label" for="profile-root">主角灵根</label>
            <input
              id="profile-root"
              v-model="randomProfile.spiritualRoot"
              type="text"
              placeholder="可填多个：火，水，木，金，土（逗号分隔）"
              class="script-input"
            />
            <p class="script-help">留空则使用随机灵根；填写多个时会按顺序择其一作为主灵根。</p>
          </div>
          <div class="script-card">
            <label class="script-label" for="profile-background">主角出生背景</label>
            <textarea
              id="profile-background"
              v-model="randomProfile.background"
              rows="3"
              placeholder="例如：寒门药童、宗门弃徒、世家旁支"
              class="script-input"
            />
            <p class="script-help">用于开局叙事参考（后续扩展为更多规则影响）。</p>
          </div>
        </div>
        <footer class="script-footer mt-4">
          <button type="button" class="script-btn" @click="showRandomProfilePanel = false">返回</button>
          <button type="button" class="script-btn script-btn-primary" @click="confirmRandomProfile">
            确认创建
          </button>
        </footer>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue';
import { useRouter } from 'vue-router';
import { open } from '@tauri-apps/plugin-dialog';
import { invokeWithTimeout } from '../utils/tauriInvoke';
import { useGameStore } from '../stores/gameStore';
import LoadingIndicator from './LoadingIndicator.vue';
import StatusBanner from './StatusBanner.vue';
import { playClick } from '../utils/audioSystem';
import { Element } from '../types/game';
import type { ScriptType, Script } from '../types/game';
import { getUiTheme } from '../utils/uiTheme';

const router = useRouter();
const gameStore = useGameStore();
const isLoading = ref(false);
const activeThemeClass = ref(getUiTheme());
const loadingMessage = ref('加载中');
const loadingProgress = ref<number | null>(null);
const loadingProgressText = ref('');
const error = ref<string | null>(null);
const selectedType = ref<ScriptType | null>(null);
const sealWhisper = ref('');
const showRandomProfilePanel = ref(false);
const randomProfile = ref({
  playerName: '',
  spiritualRoot: '',
  background: '',
});

const elementAlias: Record<string, Element> = {
  fire: Element.Fire,
  '火': Element.Fire,
  water: Element.Water,
  '水': Element.Water,
  wood: Element.Wood,
  '木': Element.Wood,
  metal: Element.Metal,
  '金': Element.Metal,
  earth: Element.Earth,
  '土': Element.Earth,
};

const parseSpiritualRoots = (input: string): Element[] => {
  const tokens = input
    .split(/[,\uFF0C\u3001\s]+/)
    .map((token) => token.trim().toLowerCase())
    .filter(Boolean);
  const mapped = tokens
    .map((token) => elementAlias[token])
    .filter((element): element is Element => Boolean(element));
  return Array.from(new Set(mapped));
};

const formatDurationMs = (ms: number): string => {
  if (!Number.isFinite(ms) || ms <= 0) {
    return '0.00s';
  }
  return `${(ms / 1000).toFixed(2)}s`;
};

const sealWhispers = [
  '印启灵光：先定心，再择路。',
  '印记示意：道在脚下，莫问远山。',
  '天机一线：凡骨亦可登仙途。',
];
let sealWhisperTimer: ReturnType<typeof setTimeout> | null = null;

const handleSealClick = () => {
  playClick();
  const idx = Math.floor(Math.random() * sealWhispers.length);
  sealWhisper.value = sealWhispers[idx];
  if (sealWhisperTimer) {
    clearTimeout(sealWhisperTimer);
  }
  sealWhisperTimer = setTimeout(() => {
    sealWhisper.value = '';
    sealWhisperTimer = null;
  }, 2200);
};

interface ScriptTypeOption {
  type: ScriptType;
  title: string;
  icon: string;
  description: string;
  actionVerb: string;
  actionTail: string;
  available: boolean;
}

const scriptTypes = ref<ScriptTypeOption[]>([
  {
    type: 'custom' as ScriptType,
    title: '自定义剧本',
    icon: '笔',
    description: '加载自定义 JSON 剧本文件',
    actionVerb: '选择文件',
    actionTail: '后导入剧本',
    available: true,
  },
  {
    type: 'random_generated' as ScriptType,
    title: '随机生成',
    icon: '鱼',
    description: '使用 AI 生成随机修仙世界',
    actionVerb: '生成',
    actionTail: '专属修仙世界',
    available: true,
  },
  {
    type: 'existing_novel' as ScriptType,
    title: '现有小说',
    icon: '卷',
    description: '从现有修仙小说导入剧本',
    actionVerb: '导入',
    actionTail: '小说并选定主角',
    available: true,
  },
]);

const novelCharacters = ref<string[]>([]);
const selectedCharacter = ref<string | null>(null);
const selectedNovelPath = ref<string | null>(null);
const showCharacterSelect = ref(false);

const canConfirm = computed(() => {
  if (showCharacterSelect.value) {
    return Boolean(selectedCharacter.value && selectedNovelPath.value);
  }
  return selectedType.value != null;
});

const confirmButtonLabel = computed(() => {
  if (showCharacterSelect.value) {
    return '确认剧本';
  }
  if (selectedType.value === 'random_generated') {
    return '创建角色';
  }
  if (selectedType.value === 'custom') {
    return '选择剧本文件';
  }
  if (selectedType.value === 'existing_novel') {
    return '选择小说文件';
  }
  return '开始旅程';
});
const isRandomGenerating = computed(() =>
  selectedType.value === 'random_generated' && loadingMessage.value.includes('随机剧本'));

const flowHint = computed(() => {
  if (isLoading.value) return loadingMessage.value;
  if (showCharacterSelect.value) return '第 2 步：确认角色并导入';
  if (selectedType.value === 'existing_novel') return '第 1 步：选择小说文件';
  if (selectedType.value === 'custom') return '第 1 步：选择自定义剧本文件';
  if (selectedType.value === 'random_generated') return '第 1 步：生成随机剧本';
  return '第 1 步：选择剧本类型';
});

const selectScriptType = (type: ScriptType) => {
  playClick();
  error.value = null;
  resetNovelSelection();
  selectedType.value = type;
};

const loadRandomScript = async () => {
  try {
    isLoading.value = true;
    loadingMessage.value = '正在生成随机剧本';
    loadingProgress.value = 50;
    loadingProgressText.value = '生成进度 1/2';
    error.value = null;

    const script = await invokeWithTimeout<Script>(
      'generate_random_script',
      undefined,
      120000,
      '随机剧本生成超时，请稍后重试',
    );
    script.initial_state.player_name = randomProfile.value.playerName.trim() || '无名弟子';
    const customRoots = parseSpiritualRoots(randomProfile.value.spiritualRoot);
    if (customRoots.length > 0) {
      script.initial_state.player_spiritual_root.element = customRoots[0];
      script.initial_state.player_spiritual_root.elements = customRoots;
    }
    const initStartedAt = Date.now();
    await gameStore.initializeGame(script, script.initial_state.player_name);
    const elapsed = gameStore.lastInitializationDurationMs ?? (Date.now() - initStartedAt);
    loadingProgress.value = 100;
    loadingProgressText.value = `生成进度 2/2（创建角色 ${formatDurationMs(elapsed)}）`;
    router.push('/game');
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  } finally {
    isLoading.value = false;
    loadingMessage.value = '加载中';
    loadingProgress.value = null;
    loadingProgressText.value = '';
  }
};

const loadCustomScript = async () => {
  try {
    isLoading.value = true;
    loadingMessage.value = '正在加载自定义剧本';
    error.value = null;

    const selected = await open({
      multiple: false,
      filters: [
        {
          name: 'JSON',
          extensions: ['json'],
        },
      ],
    });

    if (!selected) {
      return;
    }

    const script = await invokeWithTimeout<Script>(
      'load_script',
      { scriptPath: selected },
      30000,
      '加载剧本超时，请重试',
    );

    await gameStore.initializeGame(script, undefined);
    router.push('/game');
  } catch (err) {
    error.value = err instanceof Error ? err.message : '加载剧本失败';
  } finally {
    isLoading.value = false;
    loadingMessage.value = '加载中';
  }
};

const prepareExistingNovel = async () => {
  try {
    isLoading.value = true;
    loadingMessage.value = '正在解析小说';
    error.value = null;

    const selected = await open({
      multiple: false,
      filters: [
        {
          name: 'Text',
          extensions: ['txt', 'md'],
        },
      ],
    });

    if (!selected) {
      return;
    }

    selectedNovelPath.value = selected;
    const characters = await invokeWithTimeout<string[]>(
      'parse_novel_characters',
      { novelPath: selected },
      60000,
      '解析小说超时，请检查文件或重试',
    );

    if (!characters.length) {
      throw new Error('未能从小说中解析出角色列表');
    }

    novelCharacters.value = characters;
    selectedCharacter.value = characters[0] ?? null;
    showCharacterSelect.value = true;
  } catch (err) {
    error.value = err instanceof Error ? err.message : '小说解析失败';
  } finally {
    isLoading.value = false;
    loadingMessage.value = '加载中';
  }
};

const confirmNovelSelection = async () => {
  if (!selectedNovelPath.value) {
    error.value = '请先选择小说文件';
    return;
  }

  if (!selectedCharacter.value) {
    error.value = '请选择一个角色';
    return;
  }

  try {
    isLoading.value = true;
    loadingMessage.value = '正在导入小说剧本';
    error.value = null;

    const script = await invokeWithTimeout<Script>(
      'load_existing_novel',
      {
        novelPath: selectedNovelPath.value,
        selectedCharacter: selectedCharacter.value,
      },
      90000,
      '导入小说超时，请重试',
    );

    await gameStore.initializeGame(script, undefined);
    router.push('/game');
  } catch (err) {
    error.value = err instanceof Error ? err.message : '小说导入失败';
  } finally {
    isLoading.value = false;
    loadingMessage.value = '加载中';
  }
};

const resetNovelSelection = () => {
  novelCharacters.value = [];
  selectedCharacter.value = null;
  selectedNovelPath.value = null;
  showCharacterSelect.value = false;
};

const handleBack = () => {
  playClick();
  resetNovelSelection();
  selectedType.value = null;
  router.push('/');
};

const handleConfirm = async () => {
  if (isLoading.value) {
    return;
  }
  if (showCharacterSelect.value) {
    await confirmNovelSelection();
    return;
  }
  if (!selectedType.value) {
    error.value = '请先选择剧本类型';
    return;
  }

  if (selectedType.value === 'custom') {
    await loadCustomScript();
    return;
  }
  if (selectedType.value === 'random_generated') {
    showRandomProfilePanel.value = true;
    return;
  }
  if (selectedType.value === 'existing_novel') {
    await prepareExistingNovel();
  }
};

const confirmRandomProfile = async () => {
  showRandomProfilePanel.value = false;
  await loadRandomScript();
};

onBeforeUnmount(() => {
  if (sealWhisperTimer) {
    clearTimeout(sealWhisperTimer);
    sealWhisperTimer = null;
  }
});
</script>

<style scoped>
.script-shell {
  font-family: 'Noto Serif SC', 'STKaiti', 'KaiTi', serif;
  color: var(--ink-text-primary);
  background: var(--script-shell-bg);
}

.script-paper {
  position: relative;
  background: var(--script-paper-bg);
  border: 1px solid var(--ink-border-soft);
  box-shadow: var(--ink-shadow-panel);
  overflow: hidden;
  border-radius: 18px;
}

.script-paper::before {
  content: '';
  position: absolute;
  inset: 0;
  background: var(--script-paper-overlay);
  pointer-events: none;
}

.script-header {
  position: relative;
  z-index: 1;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 24px;
  margin-top: 8px;
}

.script-main-title {
  margin: 0;
  font-size: clamp(2.2rem, 4vw, 3rem);
  color: var(--ink-text-primary);
  letter-spacing: 0.06em;
  text-shadow: var(--script-title-shadow);
}

.script-sub-title {
  margin: 6px 0 0;
  color: var(--ink-title-color);
  font-size: 14px;
  letter-spacing: 0.08em;
}

.script-cn-subtitle {
  margin: 8px 0 0;
  color: var(--ink-text-muted);
  font-size: 22px;
  letter-spacing: 0.08em;
}

.script-seal {
  position: relative;
  width: 70px;
  height: 70px;
  border: 2px solid var(--ink-accent-seal);
  color: var(--ink-accent-seal);
  border-radius: 8px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transform: rotate(-7deg) translate(-10px, 6px);
  background: color-mix(in srgb, var(--ink-accent-seal) 8%, transparent);
  font-size: 15px;
  box-shadow: 0 3px 10px color-mix(in srgb, var(--ink-accent-seal) 30%, transparent);
  transition: transform 180ms ease, box-shadow 180ms ease, background-color 180ms ease;
  animation: seal-float 2.8s ease-in-out infinite;
}

.script-seal::before {
  content: '';
  position: absolute;
  inset: -5px;
  border: 1px solid color-mix(in srgb, var(--ink-accent-seal) 44%, transparent);
  border-radius: 10px;
  opacity: 0;
  transform: scale(0.92);
  transition: opacity 180ms ease, transform 180ms ease;
}

.script-seal:hover {
  transform: rotate(-7deg) translate(-10px, 4px);
  background: color-mix(in srgb, var(--ink-accent-seal) 14%, transparent);
  box-shadow: 0 6px 16px color-mix(in srgb, var(--ink-accent-seal) 36%, transparent);
}

.script-seal:hover::before,
.script-seal:focus-visible::before {
  opacity: 1;
  transform: scale(1);
}

.script-seal:active {
  transform: rotate(-7deg) translate(-10px, 6px) scale(0.98);
}

.script-seal:focus-visible {
  outline: none;
  box-shadow:
    0 0 0 2px color-mix(in srgb, var(--ink-accent-seal) 36%, transparent),
    0 6px 16px color-mix(in srgb, var(--ink-accent-seal) 36%, transparent);
}

.script-seal-whisper {
  margin: 6px 0 0;
  color: var(--ink-text-muted);
  font-size: 12px;
  letter-spacing: 0.04em;
  opacity: 0.9;
}

.script-flow-hint {
  position: relative;
  z-index: 1;
  margin: 14px 0 0;
  color: var(--ink-text-cool);
  font-size: 12px;
}

.script-card {
  background: var(--ink-card-bg);
  border: 1px solid var(--ink-border-soft);
  border-radius: 14px;
  box-shadow: var(--ink-shadow-card);
  padding: 20px;
}

.script-label {
  color: var(--ink-title-color);
  font-size: 16px;
  font-weight: 700;
}

.script-input {
  margin-top: 8px;
  width: 100%;
  border-radius: 10px;
  border: 1px solid var(--ink-border-soft);
  background: var(--ink-paper);
  color: var(--ink-text-primary);
  padding: 10px 12px;
}

.script-input::placeholder {
  color: var(--ink-text-muted);
}

.script-help {
  margin-top: 8px;
  color: var(--ink-text-muted);
  font-size: 12px;
}

.script-grid {
  position: relative;
  z-index: 1;
  display: grid;
  gap: 24px;
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

.script-type-card {
  position: relative;
  text-align: left;
  background: var(--ink-card-bg);
  border: 1px solid var(--ink-border-soft);
  border-radius: 12px;
  padding: 20px;
  min-height: 170px;
  display: flex;
  flex-direction: column;
  transition: transform 150ms ease, box-shadow 150ms ease, border-color 150ms ease;
  box-shadow: var(--script-card-shadow);
}

.script-type-card::before,
.script-type-card::after {
  content: '';
  position: absolute;
  width: 14px;
  height: 14px;
  border: 1px solid color-mix(in srgb, var(--ink-title-color) 36%, transparent);
  pointer-events: none;
}

.script-type-card::before {
  top: 8px;
  left: 8px;
  border-right: none;
  border-bottom: none;
}

.script-type-card::after {
  right: 8px;
  bottom: 8px;
  border-top: none;
  border-left: none;
}

.script-type-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--script-card-shadow-hover);
}

.script-type-card-active {
  border-color: var(--ink-title-color);
}

.script-type-card-disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.script-type-title {
  margin: 0 0 0 10px;
  color: var(--ink-title-color);
  font-size: 20px;
}

.script-type-head {
  display: flex;
  align-items: center;
}

.script-type-icon {
  width: 30px;
  height: 30px;
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--ink-title-color) 48%, transparent);
  color: var(--ink-text-primary);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 15px;
}

.script-type-desc {
  margin: 8px 0 0;
  color: var(--ink-text-primary);
  font-size: 14px;
  line-height: 1.5;
}

.script-type-action {
  margin: auto 0 0;
  color: var(--ink-text-cool);
  font-size: 12px;
}

.script-positive {
  color: var(--ink-text-cool);
  font-weight: 700;
}

.script-type-soon {
  margin: 8px 0 0;
  color: var(--ink-text-muted);
  font-size: 12px;
}

.script-tip {
  position: relative;
  z-index: 1;
  margin: 12px 0 0;
  color: var(--ink-text-cool);
  font-size: 12px;
}

.script-radio-list {
  max-height: 220px;
  overflow-y: auto;
  padding-right: 4px;
  display: grid;
  gap: 8px;
}

.script-radio-item {
  display: flex;
  gap: 8px;
  align-items: center;
  border: 1px solid var(--ink-border-soft);
  border-radius: 8px;
  background: var(--ink-paper);
  padding: 8px 10px;
}

.script-footer {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  padding: 16px;
  background: var(--ink-card-bg-muted);
  border-radius: 12px;
  border: 1px solid var(--ink-border-soft);
}

.script-overlay {
  position: fixed;
  inset: 0;
  z-index: 60;
  background: var(--script-overlay-bg);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.script-dialog {
  width: min(100%, 920px);
  border-radius: 16px;
  background: var(--script-dialog-bg);
  border: 1px solid var(--ink-border-soft);
  box-shadow: var(--ink-shadow-panel);
  padding: 20px;
}

.script-dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.script-dialog-body {
  margin-top: 12px;
  display: grid;
  gap: 12px;
}

.taiji-loader-wrap {
  display: flex;
  align-items: center;
  gap: 14px;
}

.taiji-loader-stage {
  position: relative;
  width: 86px;
  height: 86px;
  display: grid;
  place-items: center;
}

.taiji-loader-stage::before {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--ink-title-color) 58%, transparent);
  box-shadow:
    inset 0 0 0 1px color-mix(in srgb, var(--ink-text-cool) 26%, transparent),
    0 0 10px color-mix(in srgb, var(--ink-text-cool) 20%, transparent);
  animation: taiji-aura 2.4s ease-in-out infinite;
}

.taiji-loader {
  position: relative;
  width: 66px;
  height: 66px;
  border-radius: 999px;
  background: linear-gradient(90deg, var(--ink-text-primary) 50%, var(--ink-card-bg) 50%);
  border: 1px solid var(--ink-border-accent);
  box-shadow:
    inset 0 0 0 1px color-mix(in srgb, var(--ink-text-primary) 16%, transparent),
    0 4px 10px color-mix(in srgb, var(--ink-text-primary) 30%, transparent);
  animation: taiji-spin 2.1s linear infinite;
}

.taiji-loader::before,
.taiji-loader::after {
  content: '';
  position: absolute;
  left: 50%;
  width: 33px;
  height: 33px;
  transform: translateX(-50%);
  border-radius: 999px;
  z-index: 1;
}

.taiji-loader::before {
  top: 0;
  background: var(--ink-text-primary);
}

.taiji-loader::after {
  bottom: 0;
  background: var(--ink-card-bg);
}

.taiji-eye {
  position: absolute;
  left: 50%;
  width: 9px;
  height: 9px;
  border-radius: 999px;
  transform: translateX(-50%);
  box-shadow: 0 0 0 1px color-mix(in srgb, var(--ink-text-primary) 24%, transparent);
  z-index: 2;
}

.taiji-eye-yang {
  top: 12px;
  background: var(--ink-card-bg);
}

.taiji-eye-yin {
  bottom: 12px;
  background: var(--ink-text-primary);
}

@keyframes taiji-spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

@keyframes taiji-aura {
  0%,
  100% {
    opacity: 0.45;
    transform: scale(0.96);
  }
  50% {
    opacity: 0.9;
    transform: scale(1);
  }
}

@keyframes seal-float {
  0%,
  100% {
    transform: rotate(-7deg) translate(-10px, 6px);
  }
  50% {
    transform: rotate(-7deg) translate(-10px, 4px);
  }
}

.script-btn {
  border-radius: 8px;
  border: 1px solid var(--ink-border-soft);
  background: var(--ink-card-bg);
  color: var(--ink-text-primary);
  padding: 10px 24px;
  font-size: 14px;
  transition: border-color 140ms ease, background-color 140ms ease, color 140ms ease, transform 140ms ease;
}

.script-btn:hover:not(:disabled) {
  border-color: var(--ink-title-color);
  transform: translateY(-1px);
}

.script-btn-primary {
  border-color: var(--ink-title-color);
  background: color-mix(in srgb, var(--ink-title-color) 26%, var(--ink-paper));
}

.script-btn-primary:hover:not(:disabled) {
  background: var(--ink-title-color);
  color: var(--ink-paper);
}

.script-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.script-input:focus-visible,
.script-btn:focus-visible {
  outline: none;
  border-color: var(--ink-title-color);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--ink-title-color) 34%, transparent);
}

.script-watermark {
  position: absolute;
  right: 20px;
  bottom: 16px;
  z-index: 1;
  margin: 0;
  color: color-mix(in srgb, var(--ink-text-muted) 46%, transparent);
  font-size: 13px;
  letter-spacing: 0.12em;
  pointer-events: none;
}

@media (max-width: 960px) {
  .script-grid {
    grid-template-columns: 1fr;
    gap: 16px;
  }

  .script-watermark {
    right: 12px;
    bottom: 10px;
  }

  .script-seal {
    width: 56px;
    height: 56px;
    font-size: 13px;
  }
}

@media (prefers-reduced-motion: reduce) {
  .script-type-card,
  .script-btn,
  .taiji-loader,
  .taiji-loader-stage::before,
  .script-seal {
    animation: none !important;
    transition: none !important;
  }
}
</style>
