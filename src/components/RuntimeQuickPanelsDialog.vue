<template>
  <div
    v-if="isOpen"
    class="runtime-quick-overlay fixed inset-0 z-[120] flex items-center justify-center p-4"
    @click.self="$emit('close')"
  >
    <section class="runtime-quick-dialog w-full max-w-2xl rounded-2xl p-4 sm:p-5">
      <header class="runtime-quick-head">
        <div class="runtime-quick-tabs" role="tablist" aria-label="runtime quick panels">
          <button
            v-for="tab in tabs"
            :key="tab.id"
            type="button"
            role="tab"
            :aria-selected="tab.id === activeTab"
            class="runtime-quick-tab"
            :class="{ 'runtime-quick-tab-active': tab.id === activeTab }"
            @click="$emit('update:active-tab', tab.id)"
          >
            {{ tab.label }}
          </button>
        </div>
        <button type="button" class="runtime-quick-close" @click="$emit('close')">关闭</button>
      </header>
      <section class="runtime-quick-body">
        <h3 class="runtime-quick-title">{{ activePanel?.title || '' }}</h3>
        <p v-if="activePanel?.subtitle" class="runtime-quick-subtitle">{{ activePanel.subtitle }}</p>
        <label class="runtime-quick-search-wrap">
          <span class="runtime-quick-search-label">筛选</span>
          <input
            ref="searchInputRef"
            v-model.trim="searchKeyword"
            type="text"
            class="runtime-quick-search"
            :placeholder="`搜索${activePanel?.label || ''}`"
          />
        </label>
        <p v-if="copyFeedback" class="runtime-quick-feedback">{{ copyFeedback }}</p>
        <p v-if="!activePanel || filteredItems.length === 0" class="runtime-quick-empty">
          {{ activePanel?.emptyText || '暂无数据。' }}
        </p>
        <ul v-else class="runtime-quick-list">
          <li
            v-for="(item, idx) in filteredItems"
            :key="item.id"
            :data-quick-index="idx"
            class="runtime-quick-item"
            :class="{
              'runtime-quick-item-featured': item.featured,
              'runtime-quick-item-selected': idx === selectedItemIndex,
            }"
          >
            <p class="runtime-quick-item-title">
              <span>
                <template v-for="(part, pIdx) in highlightParts(item.title)" :key="`${item.id}-title-${pIdx}`">
                  <mark v-if="part.matched" class="runtime-quick-mark">{{ part.text }}</mark>
                  <span v-else>{{ part.text }}</span>
                </template>
              </span>
              <span v-if="item.badge" class="runtime-quick-item-badge">{{ item.badge }}</span>
            </p>
            <p v-if="item.description" class="runtime-quick-item-desc">
              <template v-for="(part, pIdx) in highlightParts(item.description)" :key="`${item.id}-desc-${pIdx}`">
                <mark v-if="part.matched" class="runtime-quick-mark">{{ part.text }}</mark>
                <span v-else>{{ part.text }}</span>
              </template>
            </p>
            <p v-if="item.meta" class="runtime-quick-item-meta">
              <template v-for="(part, pIdx) in highlightParts(item.meta)" :key="`${item.id}-meta-${pIdx}`">
                <mark v-if="part.matched" class="runtime-quick-mark">{{ part.text }}</mark>
                <span v-else>{{ part.text }}</span>
              </template>
            </p>
          </li>
        </ul>
      </section>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';

export type RuntimeQuickTab = 'backpack' | 'techniques' | 'factions' | 'world';

interface RuntimeQuickItem {
  id: string;
  title: string;
  description?: string;
  meta?: string;
  badge?: string;
  featured?: boolean;
}

interface RuntimeQuickPanel {
  id: RuntimeQuickTab;
  label: string;
  title: string;
  subtitle?: string;
  emptyText?: string;
  items: RuntimeQuickItem[];
}

interface HighlightPart {
  text: string;
  matched: boolean;
}

const props = defineProps<{
  isOpen: boolean;
  activeTab: RuntimeQuickTab;
  panels: RuntimeQuickPanel[];
}>();

defineEmits<{
  (event: 'close'): void;
  (event: 'update:active-tab', tab: RuntimeQuickTab): void;
}>();

const tabs = computed(() => props.panels.map((panel) => ({ id: panel.id, label: panel.label })));
const activePanel = computed(() => props.panels.find((panel) => panel.id === props.activeTab) ?? props.panels[0]);
const searchKeyword = ref('');
const searchInputRef = ref<HTMLInputElement | null>(null);
const selectedItemIndex = ref(0);
const copyFeedback = ref('');
let copyFeedbackTimer: number | null = null;

watch(
  () => props.activeTab,
  () => {
    searchKeyword.value = '';
    selectedItemIndex.value = 0;
  },
);

const normalizedKeyword = computed(() => searchKeyword.value.trim().toLowerCase());

const filteredItems = computed(() => {
  const panel = activePanel.value;
  if (!panel) return [];
  if (!normalizedKeyword.value) return panel.items;
  return panel.items.filter((item) => {
    const corpus = `${item.title} ${item.description ?? ''} ${item.meta ?? ''}`.toLowerCase();
    return corpus.includes(normalizedKeyword.value);
  });
});

watch(filteredItems, (items) => {
  if (items.length === 0) {
    selectedItemIndex.value = 0;
    return;
  }
  if (selectedItemIndex.value >= items.length) {
    selectedItemIndex.value = items.length - 1;
  }
});

const scrollSelectedIntoView = async () => {
  await nextTick();
  const el = document.querySelector<HTMLElement>(`[data-quick-index="${selectedItemIndex.value}"]`);
  el?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
};

const focusSearchInput = async () => {
  await nextTick();
  searchInputRef.value?.focus();
  searchInputRef.value?.select();
};

const setCopyFeedback = (text: string) => {
  copyFeedback.value = text;
  if (copyFeedbackTimer !== null) {
    window.clearTimeout(copyFeedbackTimer);
  }
  copyFeedbackTimer = window.setTimeout(() => {
    copyFeedback.value = '';
    copyFeedbackTimer = null;
  }, 1600);
};

const copySelectedWorldItem = async () => {
  const item = filteredItems.value[selectedItemIndex.value];
  if (!item) return;
  const summary = [item.title, item.description, item.meta]
    .filter((part) => Boolean(part && String(part).trim().length > 0))
    .join(' | ');
  if (!summary) {
    setCopyFeedback('Nothing to copy.');
    return;
  }
  if (!navigator.clipboard || !navigator.clipboard.writeText) {
    setCopyFeedback('Clipboard is not available.');
    return;
  }
  try {
    await navigator.clipboard.writeText(summary);
    setCopyFeedback('Copied current world summary.');
  } catch {
    setCopyFeedback('Copy failed. Please check clipboard permissions.');
  }
};

const handleWindowKeydown = (event: KeyboardEvent) => {
  if (!props.isOpen) return;
  const target = event.target instanceof HTMLElement ? event.target : null;
  const inInput = Boolean(
    target
    && (
      target.tagName === 'INPUT'
      || target.tagName === 'TEXTAREA'
      || target.isContentEditable
    ),
  );

  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'f') {
    event.preventDefault();
    void focusSearchInput();
    return;
  }
  if (filteredItems.value.length === 0) return;

  if (event.key === 'ArrowDown') {
    event.preventDefault();
    selectedItemIndex.value = Math.min(selectedItemIndex.value + 1, filteredItems.value.length - 1);
    void scrollSelectedIntoView();
    return;
  }
  if (event.key === 'ArrowUp') {
    event.preventDefault();
    selectedItemIndex.value = Math.max(selectedItemIndex.value - 1, 0);
    void scrollSelectedIntoView();
    return;
  }
  if (event.key === 'Enter' && props.activeTab === 'world') {
    event.preventDefault();
    void copySelectedWorldItem();
    return;
  }
  if (inInput && event.key === 'Escape') {
    event.preventDefault();
    searchInputRef.value?.blur();
  }
};

onMounted(() => {
  window.addEventListener('keydown', handleWindowKeydown);
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleWindowKeydown);
  if (copyFeedbackTimer !== null) {
    window.clearTimeout(copyFeedbackTimer);
    copyFeedbackTimer = null;
  }
});

const highlightParts = (value: string | undefined): HighlightPart[] => {
  const text = String(value ?? '');
  const key = normalizedKeyword.value;
  if (!text || !key) return [{ text, matched: false }];

  const lower = text.toLowerCase();
  const parts: HighlightPart[] = [];
  let cursor = 0;
  let found = lower.indexOf(key, cursor);
  while (found >= 0) {
    if (found > cursor) {
      parts.push({ text: text.slice(cursor, found), matched: false });
    }
    parts.push({ text: text.slice(found, found + key.length), matched: true });
    cursor = found + key.length;
    found = lower.indexOf(key, cursor);
  }
  if (cursor < text.length) {
    parts.push({ text: text.slice(cursor), matched: false });
  }
  return parts.length > 0 ? parts : [{ text, matched: false }];
};
</script>

<style scoped>
.runtime-quick-dialog {
  border: 1px solid var(--ink-border-soft);
  background: color-mix(in srgb, var(--ink-paper) 95%, var(--ink-card-bg));
  box-shadow: var(--ink-shadow-panel);
  backdrop-filter: blur(8px);
}

.runtime-quick-overlay {
  background: color-mix(in srgb, var(--ink-text-primary) 30%, transparent);
}

.runtime-quick-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.runtime-quick-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.runtime-quick-tab {
  border-radius: 10px;
  border: 1px solid var(--ink-border-accent);
  background: color-mix(in srgb, var(--ink-paper) 76%, var(--ink-card-bg));
  color: color-mix(in srgb, var(--ink-text-primary) 75%, var(--ink-text-muted));
  padding: 6px 12px;
  font-size: 13px;
}

.runtime-quick-tab-active {
  border-color: var(--ink-title-color);
  background: color-mix(in srgb, var(--ink-title-color) 24%, var(--ink-paper));
  color: color-mix(in srgb, var(--ink-title-color) 78%, var(--ink-text-primary));
}

.runtime-quick-close {
  border-radius: 10px;
  border: 1px solid var(--ink-border-accent);
  background: color-mix(in srgb, var(--ink-paper) 72%, var(--ink-card-bg));
  color: color-mix(in srgb, var(--ink-title-color) 78%, var(--ink-text-primary));
  padding: 6px 14px;
  font-size: 13px;
}

.runtime-quick-body {
  margin-top: 14px;
  max-height: min(62vh, 520px);
  overflow: auto;
  padding-right: 2px;
}

.runtime-quick-title {
  margin: 0;
  color: var(--ink-title-color);
  font-size: 21px;
  font-weight: 700;
  line-height: 1.25;
}

.runtime-quick-subtitle {
  margin: 6px 0 0;
  color: var(--ink-text-muted);
  font-size: 13px;
  line-height: 1.6;
}

.runtime-quick-search-wrap {
  margin-top: 10px;
  display: grid;
  gap: 4px;
}

.runtime-quick-search-label {
  color: var(--ink-text-muted);
  font-size: 12px;
  line-height: 1.4;
}

.runtime-quick-search {
  border-radius: 10px;
  border: 1px solid var(--ink-border-soft);
  background: color-mix(in srgb, var(--ink-paper) 70%, transparent);
  color: var(--ink-text-primary);
  padding: 7px 10px;
  font-size: 13px;
  line-height: 1.35;
}

.runtime-quick-empty {
  margin: 12px 0 0;
  color: var(--ink-text-muted);
  font-size: 14px;
}

.runtime-quick-feedback {
  margin: 8px 0 0;
  color: color-mix(in srgb, var(--ink-title-color) 78%, var(--ink-text-primary));
  font-size: 12px;
  line-height: 1.4;
}

.runtime-quick-list {
  margin: 12px 0 0;
  padding: 0;
  list-style: none;
  display: grid;
  gap: 10px;
}

.runtime-quick-item {
  border-radius: 12px;
  border: 1px solid var(--ink-border-soft);
  background: color-mix(in srgb, var(--ink-paper) 68%, transparent);
  padding: 10px 12px;
}

.runtime-quick-item-featured {
  border-color: color-mix(in srgb, var(--ink-title-color) 58%, var(--ink-border-accent));
  background: color-mix(in srgb, var(--ink-title-color) 14%, var(--ink-paper));
}

.runtime-quick-item-selected {
  border-color: var(--ink-title-color);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--ink-title-color) 34%, transparent);
}

.runtime-quick-item-title {
  margin: 0;
  color: var(--ink-text-primary);
  font-size: 15px;
  font-weight: 600;
  line-height: 1.5;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.runtime-quick-item-badge {
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--ink-title-color) 62%, var(--ink-border-accent));
  background: color-mix(in srgb, var(--ink-title-color) 22%, var(--ink-paper));
  color: color-mix(in srgb, var(--ink-title-color) 82%, var(--ink-text-primary));
  padding: 1px 8px;
  font-size: 11px;
  font-weight: 600;
  line-height: 1.4;
}

.runtime-quick-mark {
  border-radius: 4px;
  background: color-mix(in srgb, var(--ink-title-color) 48%, transparent);
  color: inherit;
  padding: 0 1px;
}

.runtime-quick-item-desc,
.runtime-quick-item-meta {
  margin: 4px 0 0;
  color: var(--ink-text-muted);
  font-size: 13px;
  line-height: 1.6;
  overflow-wrap: anywhere;
  word-break: break-word;
}

.runtime-quick-item-meta {
  color: color-mix(in srgb, var(--ink-title-color) 64%, var(--ink-text-primary));
}
</style>
