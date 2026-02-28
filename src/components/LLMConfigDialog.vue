<template>
  <div v-if="inline" class="llm-inline-wrap">
    <div class="space-y-3">
      <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
        <label class="llm-label text-sm">
          Endpoint
          <input v-model="form.endpoint" class="llm-input" />
        </label>
        <label class="llm-label text-sm">
          模型ID（model）
          <input v-model="form.model" class="llm-input" />
        </label>
      </div>
      <p class="llm-note text-xs">提示：这里填写 provider 的 modelId（如 `xopkimik25`），不是展示名称。</p>

      <label class="llm-label text-sm">
        API Key
        <input v-model="form.apiKey" type="password" class="llm-input" />
      </label>

      <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
        <label class="llm-label text-sm">
          maxTokens
          <input v-model.number="form.maxTokens" type="number" min="1" class="llm-input" />
        </label>
        <label class="llm-label text-sm">
          temperature
          <input v-model.number="form.temperature" type="number" min="0" max="2" step="0.1" class="llm-input" />
        </label>
      </div>

      <p class="llm-note text-xs">当前状态：{{ statusText }}</p>
      <LoadingIndicator
        v-if="busy"
        :message="loadingMessage"
        detail="正在与模型服务交互..."
        size="sm"
      />
      <p v-if="message" class="llm-message text-sm">{{ message }}</p>
      <p v-if="error" class="llm-error text-sm">{{ error }}</p>

      <div class="flex flex-wrap gap-2">
        <button class="llm-btn llm-btn-primary" @click="saveConfig" :disabled="busy || !isFormValid">保存配置</button>
        <button class="llm-btn llm-btn-success" @click="testConnection" :disabled="busy">测试连接</button>
        <button class="llm-btn" @click="loadStatus" :disabled="busy">刷新状态</button>
        <button class="llm-btn" @click="clearConfig" :disabled="busy">清除运行时配置</button>
      </div>
      <p v-if="!isFormValid" class="llm-warning text-sm">{{ formValidation.join('；') }}</p>
    </div>
  </div>

  <div v-else-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center bg-black/25 p-4">
    <section class="llm-modal w-full max-w-4xl rounded-2xl p-5">
      <header class="mb-3 flex items-center justify-between">
        <h3 class="llm-title text-xl font-display">LLM 模型配置</h3>
        <button class="llm-btn" @click="$emit('close')">关闭</button>
      </header>
      <div class="llm-dialog-grid">
        <aside class="llm-left">
          <p class="llm-sub-title">设置分组</p>
          <div class="mt-2 grid gap-2">
            <button type="button" class="llm-chip llm-chip-active text-left">模型配置</button>
          </div>
        </aside>
        <section class="llm-right">
          <div class="llm-inline-wrap p-3">
            <div class="space-y-3">
              <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
                <label class="llm-label text-sm">Endpoint<input v-model="form.endpoint" class="llm-input" /></label>
                <label class="llm-label text-sm">模型ID（model）<input v-model="form.model" class="llm-input" /></label>
              </div>
              <p class="llm-note text-xs">提示：这里填写 provider 的 modelId（如 `xopkimik25`），不是展示名称。</p>
              <label class="llm-label text-sm">API Key<input v-model="form.apiKey" type="password" class="llm-input" /></label>
              <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
                <label class="llm-label text-sm">maxTokens<input v-model.number="form.maxTokens" type="number" min="1" class="llm-input" /></label>
                <label class="llm-label text-sm">temperature<input v-model.number="form.temperature" type="number" min="0" max="2" step="0.1" class="llm-input" /></label>
              </div>
              <p class="llm-note text-xs">当前状态：{{ statusText }}</p>
              <LoadingIndicator v-if="busy" :message="loadingMessage" detail="正在与模型服务交互..." size="sm" />
              <p v-if="message" class="llm-message text-sm">{{ message }}</p>
              <p v-if="error" class="llm-error text-sm">{{ error }}</p>
              <div class="flex flex-wrap gap-2">
                <button class="llm-btn llm-btn-primary" @click="saveConfig" :disabled="busy || !isFormValid">保存配置</button>
                <button class="llm-btn llm-btn-success" @click="testConnection" :disabled="busy">测试连接</button>
                <button class="llm-btn" @click="loadStatus" :disabled="busy">刷新状态</button>
                <button class="llm-btn" @click="clearConfig" :disabled="busy">清除运行时配置</button>
              </div>
              <p v-if="!isFormValid" class="llm-warning text-sm">{{ formValidation.join('；') }}</p>
            </div>
          </div>
        </section>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { invokeWithTimeout } from '../utils/tauriInvoke';
import { computed, reactive, ref, watch } from 'vue';
import LoadingIndicator from './LoadingIndicator.vue';
import { playClick } from '../utils/audioSystem';

interface LLMConfigStatus {
  configured: boolean;
  source: string;
  endpoint?: string;
  model?: string;
  max_tokens?: number;
  temperature?: number;
}

const props = withDefaults(defineProps<{ isOpen: boolean; inline?: boolean }>(), {
  inline: false,
});
defineEmits<{ close: [] }>();

const form = reactive({
  endpoint: 'https://api.siliconflow.cn/v1/chat/completions',
  apiKey: '',
  model: 'deepseek-ai/DeepSeek-V3.2',
  maxTokens: 1024,
  temperature: 0.7,
});

const API_KEY_STORAGE = 'nobody_llm_api_key';

const status = ref<LLMConfigStatus | null>(null);
const busy = ref(false);
const error = ref('');
const message = ref('');
const loadingMessage = ref('处理中...');

const statusText = computed(() => {
  if (!status.value) return '未读取';
  if (!status.value.configured) return '未配置';
  return `已配置（来源: ${status.value.source}，模型: ${status.value.model ?? '-'}）`;
});

watch(
  () => props.isOpen,
  (open) => {
    if (open) {
      void loadStatus();
    }
  },
);

const loadStatus = async () => {
  busy.value = true;
  error.value = '';
  message.value = '';
  loadingMessage.value = '正在读取配置状态...';
  try {
    const result = await invokeWithTimeout<LLMConfigStatus>(
      'get_llm_config_status',
      undefined,
      8000,
      '读取配置状态超时，请稍后重试',
    );
    status.value = result;
    if (result.configured) {
      form.endpoint = result.endpoint ?? form.endpoint;
      form.model = result.model ?? form.model;
      form.maxTokens = result.max_tokens ?? form.maxTokens;
      form.temperature = result.temperature ?? form.temperature;
    }
    if (!form.apiKey) {
      const cached = window.localStorage.getItem(API_KEY_STORAGE);
      if (cached) {
        form.apiKey = cached;
      }
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
    loadingMessage.value = '处理中...';
  }
};

const formValidation = computed(() => {
  const errors: string[] = [];
  if (!form.endpoint.trim()) {
    errors.push('Endpoint 不能为空');
  }
  if (!form.model.trim()) {
    errors.push('模型ID不能为空');
  }
  if (form.maxTokens < 1 || form.maxTokens > 32000) {
    errors.push('maxTokens 必须在 1-32000 之间');
  }
  if (form.temperature < 0 || form.temperature > 2) {
    errors.push('temperature 必须在 0-2 之间');
  }
  return errors;
});

const isFormValid = computed(() => formValidation.value.length === 0);

const saveConfig = async () => {
  if (!isFormValid.value) {
    error.value = formValidation.value.join('；');
    return;
  }

  busy.value = true;
  error.value = '';
  message.value = '';
  loadingMessage.value = '正在保存配置...';
  playClick();
  try {
    const msg = await invokeWithTimeout<string>(
      'set_llm_config',
      {
        input: {
          endpoint: form.endpoint.trim(),
          apiKey: form.apiKey.trim(),
          model: form.model.trim(),
          maxTokens: form.maxTokens,
          temperature: form.temperature,
        },
      },
      10000,
      '保存配置超时，请检查网络或重试',
    );
    if (form.apiKey.trim()) {
      window.localStorage.setItem(API_KEY_STORAGE, form.apiKey.trim());
    } else {
      window.localStorage.removeItem(API_KEY_STORAGE);
    }
    message.value = msg;
    await loadStatus();
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
    loadingMessage.value = '处理中...';
  }
};

const testConnection = async () => {
  busy.value = true;
  error.value = '';
  message.value = '';
  loadingMessage.value = '正在测试连接...';
  playClick();
  try {
    const text = await invokeWithTimeout<string>(
      'test_llm_connection',
      undefined,
      20000,
      '测试连接超时，请检查模型服务',
    );
    message.value = `连接成功：${text}`;
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
    loadingMessage.value = '处理中...';
  }
};

const clearConfig = async () => {
  busy.value = true;
  error.value = '';
  message.value = '';
  loadingMessage.value = '正在清除配置...';
  playClick();
  try {
    const msg = await invokeWithTimeout<string>(
      'clear_llm_config',
      undefined,
      8000,
      '清除配置超时，请稍后重试',
    );
    message.value = msg;
    form.apiKey = '';
    window.localStorage.removeItem(API_KEY_STORAGE);
    await loadStatus();
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
    loadingMessage.value = '处理中...';
  }
};
</script>

<style scoped>
.llm-modal,
.llm-inline-wrap {
  background: var(--ink-card-bg, var(--panel-bg));
  border: 1px solid var(--ink-border-soft, var(--ink-border));
}

.llm-dialog-grid {
  display: grid;
  grid-template-columns: 220px minmax(0, 1fr);
  gap: 14px;
}

.llm-left,
.llm-right {
  border: 1px solid var(--ink-border-soft, var(--ink-border));
  border-radius: 12px;
  background: var(--ink-card-bg-soft, var(--ink-card-bg));
  padding: 12px;
}

.llm-sub-title {
  margin: 0;
  color: var(--ink-title-color);
  font-size: 12px;
  letter-spacing: 0.06em;
}

.llm-chip {
  border: 1px solid var(--ink-border-accent, var(--ink-border-strong));
  border-radius: 8px;
  background: var(--ink-card-bg, var(--panel-bg));
  color: var(--ink-text-primary);
  padding: 6px 10px;
  font-size: 13px;
}

.llm-chip-active {
  border-color: var(--ink-title-color);
}

.llm-input {
  margin-top: 4px;
  width: 100%;
  border-radius: 8px;
  border: 1px solid var(--ink-border-soft, var(--ink-border));
  background: var(--ink-paper, var(--ink-card-bg));
  padding: 8px 10px;
  color: var(--ink-text-primary);
}

.llm-btn {
  border: 1px solid var(--ink-border-accent, var(--ink-border-strong));
  border-radius: 8px;
  background: var(--ink-paper-elevated, var(--ink-paper, var(--ink-card-bg)));
  color: var(--ink-text-primary);
  padding: 7px 12px;
  font-size: 13px;
}

.llm-btn:hover {
  border-color: var(--ink-title-color);
  background: var(--ink-paper, var(--ink-card-bg));
}

.llm-btn-primary {
  border-color: var(--ink-title-color);
  background: color-mix(in srgb, var(--ink-accent-note, var(--ink-title-color)) 18%, var(--ink-paper, var(--ink-card-bg)));
}

.llm-btn-success {
  border-color: var(--ink-text-cool);
  background: color-mix(in srgb, var(--ink-text-cool) 18%, var(--ink-paper, var(--ink-card-bg)));
  color: var(--ink-text-ink, var(--ink-text-cool));
}

.llm-label {
  color: var(--ink-text-muted);
}

.llm-note {
  color: color-mix(in srgb, var(--ink-text-muted) 92%, transparent);
}

.llm-message {
  color: var(--ink-text-cool);
}

.llm-error {
  color: var(--ink-accent-main);
}

.llm-warning {
  color: var(--ink-title-color);
}

.llm-title {
  color: var(--ink-title-color);
}

.llm-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

@media (max-width: 900px) {
  .llm-dialog-grid {
    grid-template-columns: 1fr;
  }
}
</style>
