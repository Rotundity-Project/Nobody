<template>
  <section
    v-if="isGameRunning"
    class="rounded-2xl border border-amber-500/30 bg-slate-900/70 p-5 space-y-3 shadow-xl"
  >
    <header class="flex items-center justify-between">
      <h3 class="text-lg font-display text-amber-200">经历整理与导出</h3>
      <span class="text-xs text-slate-400">事件数：{{ eventCount }}</span>
    </header>

    <div class="space-y-2">
      <label class="text-sm text-slate-300">导出标题</label>
      <input
        v-model="novelTitle"
        class="w-full rounded border border-slate-600 bg-slate-800 px-3 py-2 text-sm text-white outline-none focus:border-amber-400"
        placeholder="修仙旅程记录"
      />
    </div>

    <div class="flex items-center gap-2">
      <button
        @click="handleGenerate"
        :disabled="isGenerating"
        class="rounded bg-amber-500 px-3 py-2 text-sm text-slate-900 transition hover:bg-amber-400 disabled:cursor-not-allowed disabled:bg-slate-600"
      >
        {{ isGenerating ? '整理中...' : '整理经历' }}
      </button>
      <button
        @click="handleExport"
        :disabled="!novel || isExporting"
        class="rounded bg-emerald-500 px-3 py-2 text-sm text-slate-900 transition hover:bg-emerald-400 disabled:cursor-not-allowed disabled:bg-slate-600"
      >
        {{ isExporting ? '导出中...' : '导出经历 TXT' }}
      </button>
    </div>

    <LoadingIndicator
      v-if="isGenerating || isExporting"
      :message="loadingMessage"
      detail="正在整理玩家经历并生成章节目录..."
      size="sm"
    />
    <StatusBanner
      v-if="statusMessage"
      kind="success"
      title="状态"
      :message="statusMessage"
    />
    <StatusBanner
      v-if="errorMessage"
      kind="error"
      title="错误"
      :message="errorMessage"
    />

    <div v-if="novel" class="max-h-64 overflow-y-auto rounded border border-slate-700 bg-slate-950/60 p-3">
      <h4 class="text-sm font-semibold text-amber-200">{{ novel.title }}</h4>
      <p class="mt-2 text-xs text-slate-400">章节数：{{ novel.chapters.length }}</p>
      <div v-if="tocEntries.length > 0" class="mt-3 rounded border border-slate-800 bg-slate-900/60 p-2">
        <p class="text-xs uppercase tracking-[0.2em] text-amber-200/80">目录</p>
        <p
          v-for="entry in tocEntries"
          :key="`${entry.index}-${entry.title}`"
          class="mt-1 text-xs text-slate-300"
        >
          {{ entry.index }}. {{ entry.title }} - {{ entry.summary }}
        </p>
      </div>
      <article
        v-for="chapter in novel.chapters"
        :key="chapter.index"
        class="mt-3 border-t border-slate-700 pt-2"
      >
        <h5 class="text-sm font-medium text-slate-200">{{ chapter.title }}</h5>
        <p class="mt-1 whitespace-pre-wrap text-sm text-slate-300 font-story">
          {{ chapter.content }}
        </p>
      </article>
    </div>
  </section>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import { computed, ref } from 'vue';
import LoadingIndicator from './LoadingIndicator.vue';
import StatusBanner from './StatusBanner.vue';
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
  toc?: TocEntry[];
  total_events: number;
}

interface TocEntry {
  index: number;
  title: string;
  summary: string;
  source_event_count: number;
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
const loadingMessage = ref('处理中...');

const eventCount = computed(() => props.eventCount ?? 0);
const tocEntries = computed(() => {
  if (!novel.value) {
    return [];
  }
  if (novel.value.toc && novel.value.toc.length > 0) {
    return novel.value.toc;
  }
  return novel.value.chapters.map((chapter) => ({
    index: chapter.index,
    title: chapter.title,
    summary: chapter.content.slice(0, 36),
    source_event_count: chapter.source_event_ids.length,
  }));
});

const handleGenerate = async () => {
  errorMessage.value = '';
  statusMessage.value = '正在根据玩家经历整理章节...';
  isGenerating.value = true;
  loadingMessage.value = '正在整理经历...';
  try {
    const generated = await invoke<Novel>('generate_novel', {
      title: novelTitle.value.trim() || '修仙旅程记录',
    });
    novel.value = generated;
    statusMessage.value = `已整理 ${generated.chapters.length} 章并生成目录。`;
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
    statusMessage.value = '';
  } finally {
    isGenerating.value = false;
    loadingMessage.value = '处理中...';
  }
};

const handleExport = async () => {
  if (!novel.value) {
    return;
  }

  errorMessage.value = '';
  statusMessage.value = '正在准备导出...';
  isExporting.value = true;
  loadingMessage.value = '正在导出小说...';
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
    loadingMessage.value = '处理中...';
  }
};
</script>
