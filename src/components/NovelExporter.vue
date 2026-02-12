<template>
  <section
    v-if="isGameRunning"
    class="rounded-xl border border-amber-500/40 bg-slate-800/80 p-4 space-y-3"
  >
    <header class="flex items-center justify-between">
      <h3 class="text-lg font-semibold text-amber-300">小说生成与导出</h3>
      <span class="text-xs text-slate-400">事件数：{{ eventCount }}</span>
    </header>

    <div class="space-y-2">
      <label class="text-sm text-slate-300">小说标题</label>
      <input
        v-model="novelTitle"
        class="w-full rounded border border-slate-600 bg-slate-700 px-3 py-2 text-sm text-white outline-none focus:border-amber-400"
        placeholder="修仙旅程记录"
      />
    </div>

    <div class="flex items-center gap-2">
      <button
        @click="handleGenerate"
        :disabled="isGenerating"
        class="rounded bg-amber-600 px-3 py-2 text-sm text-white transition hover:bg-amber-500 disabled:cursor-not-allowed disabled:bg-slate-600"
      >
        {{ isGenerating ? '生成中...' : '生成小说' }}
      </button>
      <button
        @click="handleExport"
        :disabled="!novel || isExporting"
        class="rounded bg-emerald-600 px-3 py-2 text-sm text-white transition hover:bg-emerald-500 disabled:cursor-not-allowed disabled:bg-slate-600"
      >
        {{ isExporting ? '导出中...' : '导出 TXT' }}
      </button>
    </div>

    <p v-if="statusMessage" class="text-sm text-slate-300">{{ statusMessage }}</p>
    <p v-if="errorMessage" class="text-sm text-red-300">{{ errorMessage }}</p>

    <div v-if="novel" class="max-h-64 overflow-y-auto rounded border border-slate-700 bg-slate-900/70 p-3">
      <h4 class="text-sm font-semibold text-amber-200">{{ novel.title }}</h4>
      <p class="mt-2 text-xs text-slate-400">章节数：{{ novel.chapters.length }}</p>
      <article
        v-for="chapter in novel.chapters"
        :key="chapter.index"
        class="mt-3 border-t border-slate-700 pt-2"
      >
        <h5 class="text-sm font-medium text-slate-200">{{ chapter.title }}</h5>
        <p class="mt-1 whitespace-pre-wrap text-sm text-slate-300">{{ chapter.content }}</p>
      </article>
    </div>
  </section>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import { computed, ref } from 'vue';
import { buildNovelExportFilename } from '../utils/novelExporter';

interface Chapter {
  index: number;
  title: string;
  content: string;
  source_event_ids: number[];
}

interface Novel {
  title: string;
  chapters: Chapter[];
  total_events: number;
}

const props = withDefaults(
  defineProps<{
    isGameRunning: boolean;
    eventCount?: number;
  }>(),
  {
    eventCount: 0,
  },
);

const novelTitle = ref('修仙旅程记录');
const novel = ref<Novel | null>(null);
const isGenerating = ref(false);
const isExporting = ref(false);
const errorMessage = ref('');
const statusMessage = ref('');

const eventCount = computed(() => props.eventCount ?? 0);

const handleGenerate = async () => {
  errorMessage.value = '';
  statusMessage.value = '正在根据事件历史生成小说...';
  isGenerating.value = true;
  try {
    const generated = await invoke<Novel>('generate_novel', {
      title: novelTitle.value.trim() || '修仙旅程记录',
    });
    novel.value = generated;
    statusMessage.value = `已生成 ${generated.chapters.length} 章。`;
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
    statusMessage.value = '';
  } finally {
    isGenerating.value = false;
  }
};

const handleExport = async () => {
  if (!novel.value) {
    return;
  }

  errorMessage.value = '';
  statusMessage.value = '正在准备导出...';
  isExporting.value = true;
  try {
    const selectedPath = await save({
      defaultPath: buildNovelExportFilename(novel.value.title),
      filters: [{ name: '文本文件', extensions: ['txt'] }],
    });

    if (!selectedPath) {
      statusMessage.value = '已取消导出。';
      return;
    }

    await invoke('export_novel', {
      novel: novel.value,
      outputPath: selectedPath,
    });
    statusMessage.value = `已导出到：${selectedPath}`;
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
    statusMessage.value = '';
  } finally {
    isExporting.value = false;
  }
};
</script>
