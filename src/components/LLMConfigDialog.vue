<template>
  <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4">
    <div class="w-full max-w-2xl rounded-xl border border-slate-700 bg-slate-900 p-6">
      <div class="mb-4 flex items-center justify-between">
        <h3 class="text-xl font-semibold text-white">LLM 模型配置</h3>
        <button class="rounded bg-slate-700 px-3 py-1 text-sm text-gray-200" @click="$emit('close')">关闭</button>
      </div>

      <div class="space-y-3">
        <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
          <label class="text-sm text-gray-300">
            Endpoint
            <input v-model="form.endpoint" class="mt-1 w-full rounded border border-slate-600 bg-slate-800 px-3 py-2 text-white" />
          </label>
          <label class="text-sm text-gray-300">
            模型名称
            <input v-model="form.model" class="mt-1 w-full rounded border border-slate-600 bg-slate-800 px-3 py-2 text-white" />
          </label>
        </div>

        <label class="text-sm text-gray-300">
          API Key
          <input v-model="form.apiKey" type="password" class="mt-1 w-full rounded border border-slate-600 bg-slate-800 px-3 py-2 text-white" />
        </label>

        <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
          <label class="text-sm text-gray-300">
            maxTokens
            <input v-model.number="form.maxTokens" type="number" min="1" class="mt-1 w-full rounded border border-slate-600 bg-slate-800 px-3 py-2 text-white" />
          </label>
          <label class="text-sm text-gray-300">
            temperature
            <input v-model.number="form.temperature" type="number" min="0" max="2" step="0.1" class="mt-1 w-full rounded border border-slate-600 bg-slate-800 px-3 py-2 text-white" />
          </label>
        </div>

        <p class="text-xs text-slate-400">当前状态：{{ statusText }}</p>
        <p v-if="message" class="text-sm text-emerald-300">{{ message }}</p>
        <p v-if="error" class="text-sm text-red-300">{{ error }}</p>

        <div class="flex flex-wrap gap-2">
          <button class="rounded bg-purple-600 px-3 py-2 text-sm text-white" @click="saveConfig" :disabled="busy">保存配置</button>
          <button class="rounded bg-blue-600 px-3 py-2 text-sm text-white" @click="testConnection" :disabled="busy">测试连接</button>
          <button class="rounded bg-amber-600 px-3 py-2 text-sm text-white" @click="loadStatus" :disabled="busy">刷新状态</button>
          <button class="rounded bg-gray-700 px-3 py-2 text-sm text-white" @click="clearConfig" :disabled="busy">清除运行时配置</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { computed, reactive, ref, watch } from 'vue';

interface LLMConfigStatus {
  configured: boolean;
  source: string;
  endpoint?: string;
  model?: string;
  max_tokens?: number;
  temperature?: number;
}

const props = defineProps<{ isOpen: boolean }>();
defineEmits<{ close: [] }>();

const form = reactive({
  endpoint: 'https://api.siliconflow.cn/v1/chat/completions',
  apiKey: '',
  model: 'deepseek-ai/DeepSeek-V3.2',
  maxTokens: 1024,
  temperature: 0.7,
});

const status = ref<LLMConfigStatus | null>(null);
const busy = ref(false);
const error = ref('');
const message = ref('');

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
  try {
    const result = await invoke<LLMConfigStatus>('get_llm_config_status');
    status.value = result;
    if (result.configured) {
      form.endpoint = result.endpoint ?? form.endpoint;
      form.model = result.model ?? form.model;
      form.maxTokens = result.max_tokens ?? form.maxTokens;
      form.temperature = result.temperature ?? form.temperature;
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
};

const saveConfig = async () => {
  busy.value = true;
  error.value = '';
  message.value = '';
  try {
    const msg = await invoke<string>('set_llm_config', {
      input: {
        endpoint: form.endpoint,
        apiKey: form.apiKey,
        model: form.model,
        maxTokens: form.maxTokens,
        temperature: form.temperature,
      },
    });
    message.value = msg;
    await loadStatus();
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
};

const testConnection = async () => {
  busy.value = true;
  error.value = '';
  message.value = '';
  try {
    const text = await invoke<string>('test_llm_connection');
    message.value = `连接成功：${text}`;
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
};

const clearConfig = async () => {
  busy.value = true;
  error.value = '';
  message.value = '';
  try {
    const msg = await invoke<string>('clear_llm_config');
    message.value = msg;
    form.apiKey = '';
    await loadStatus();
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
};
</script>
