<template>
  <section
    v-if="isGameRunning"
    class="novel-export-panel rounded-2xl p-5 shadow-xl space-y-3"
  >
    <header class="flex items-center justify-between">
      <h3 class="text-lg font-display novel-title">经历整理与导出</h3>
      <span class="text-xs novel-meta">事件数：{{ eventCount }}</span>
    </header>

    <div class="space-y-2">
      <label class="text-sm novel-meta-strong">导出标题</label>
      <input
        v-model="novelTitle"
        class="novel-input w-full rounded px-3 py-2 text-sm outline-none"
        placeholder="修仙旅程记录"
      />
    </div>

    <div class="flex items-center gap-2">
      <button
        @click="handleGenerate"
        :disabled="isGenerating"
        class="novel-btn novel-btn-primary rounded px-3 py-2 text-sm transition disabled:cursor-not-allowed"
      >
        {{ isGenerating ? '整理中...' : '整理经历' }}
      </button>
      <button
        @click="handleExport"
        :disabled="!novel || isExporting"
        class="novel-btn novel-btn-secondary rounded px-3 py-2 text-sm transition disabled:cursor-not-allowed"
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

    <div v-if="novel" class="novel-preview max-h-64 overflow-y-auto rounded p-3">
      <h4 class="text-sm font-semibold novel-title">{{ novel.title }}</h4>
      <p class="mt-2 text-xs novel-meta">章节数：{{ novel.chapters.length }}</p>
      <div v-if="tocEntries.length > 0" class="novel-toc mt-3 rounded p-2">
        <p class="text-xs uppercase tracking-[0.2em] novel-meta-strong">目录</p>
        <p
          v-for="entry in tocEntries"
          :key="`${entry.index}-${entry.title}`"
          class="mt-1 text-xs novel-meta-strong"
        >
          {{ entry.index }}. {{ entry.title }} - {{ entry.summary }}
        </p>
      </div>
      <article
        v-for="chapter in novel.chapters"
        :key="chapter.index"
        class="novel-chapter mt-3 pt-2"
      >
        <h5 class="text-sm font-medium novel-meta-strong">{{ chapter.title }}</h5>
        <p class="mt-1 whitespace-pre-wrap text-sm novel-meta-strong font-story">
          {{ chapter.content }}
        </p>
      </article>
    </div>
  </section>
</template>

<script setup lang="ts">
import { save } from '@tauri-apps/plugin-dialog';
import { computed, ref } from 'vue';
import LoadingIndicator from './LoadingIndicator.vue';
import StatusBanner from './StatusBanner.vue';
import { buildNovelExportFilename } from '../utils/novelExporter';
import { isTauriRuntime } from '../platform/runtimeEnv';
import { invokeRuntime } from '../utils/tauriInvoke';

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
    const generated = await invokeRuntime<Novel>('generate_novel', {
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
    const defaultPath = buildNovelExportFilename(novel.value.title);
    let selectedPath: string | null = defaultPath;
    if (isTauriRuntime()) {
      selectedPath = await save({
        defaultPath,
        filters: [{ name: '文本文件', extensions: ['txt'] }],
      });
      if (!selectedPath) {
        statusMessage.value = '已取消导出。';
        return;
      }
    }

    await invokeRuntime('export_novel', {
      novel: novel.value,
      outputPath: selectedPath,
    });
    statusMessage.value = isTauriRuntime() ? `已导出到：${selectedPath}` : `已下载：${selectedPath}`;
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
    statusMessage.value = '';
  } finally {
    isExporting.value = false;
    loadingMessage.value = '处理中...';
  }
};
</script>

<style scoped>
.novel-export-panel {
  border: 1px solid var(--ink-border-soft);
  background: var(--novel-panel-bg);
}

.novel-title {
  color: var(--ink-title-color);
}

.novel-meta {
  color: var(--ink-text-muted);
}

.novel-meta-strong {
  color: var(--ink-text-primary);
}

.novel-input {
  border: 1px solid var(--ink-border-soft);
  background: var(--novel-input-bg);
  color: var(--ink-text-primary);
}

.novel-input:focus {
  border-color: var(--ink-title-color);
  box-shadow: 0 0 0 2px var(--novel-input-focus-ring);
}

.novel-btn {
  border: 1px solid var(--ink-border-accent);
  color: var(--ink-text-primary);
}

.novel-btn-primary {
  background: var(--novel-btn-primary-bg);
}

.novel-btn-secondary {
  background: var(--novel-btn-secondary-bg);
}

.novel-btn:hover:not(:disabled) {
  border-color: var(--ink-title-color);
  background: var(--ink-paper);
}

.novel-btn:disabled {
  opacity: 0.55;
}

.novel-preview {
  border: 1px solid var(--ink-border-soft);
  background: var(--novel-preview-bg);
}

.novel-toc {
  border: 1px solid var(--novel-toc-border);
  background: var(--novel-toc-bg);
}

.novel-chapter {
  border-top: 1px solid var(--ink-border-soft);
}
</style>
