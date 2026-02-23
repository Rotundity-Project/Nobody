<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 z-[70] flex items-center justify-center bg-black/30 p-4"
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
        <button type="button" class="runtime-quick-close" @click="$emit('close')">Close</button>
      </header>
      <section class="runtime-quick-body">
        <h3 class="runtime-quick-title">{{ activePanel?.title || '' }}</h3>
        <p v-if="activePanel?.subtitle" class="runtime-quick-subtitle">{{ activePanel.subtitle }}</p>
        <label class="runtime-quick-search-wrap">
          <span class="runtime-quick-search-label">Filter</span>
          <input
            ref="searchInputRef"
            v-model.trim="searchKeyword"
            type="text"
            class="runtime-quick-search"
            :placeholder="`Search ${activePanel?.label || ''}`"
          />
        </label>
        <p v-if="copyFeedback" class="runtime-quick-feedback">{{ copyFeedback }}</p>
        <p v-if="!activePanel || filteredItems.length === 0" class="runtime-quick-empty">
          {{ activePanel?.emptyText || 'No data.' }}
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
  border: 1px solid #d8ccb9;
  background: linear-gradient(140deg, rgba(251, 248, 242, 0.98), rgba(244, 238, 229, 0.96));
  box-shadow: 0 16px 36px rgba(45, 42, 36, 0.2);
  backdrop-filter: blur(8px);
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
  border: 1px solid #c6b295;
  background: linear-gradient(180deg, #f8f4ec, #efe7da);
  color: #5d4f3b;
  padding: 6px 12px;
  font-size: 13px;
}

.runtime-quick-tab-active {
  border-color: #b78c4a;
  background: linear-gradient(180deg, #f4ecd9, #ead7b5);
  color: #6b4f2f;
}

.runtime-quick-close {
  border-radius: 10px;
  border: 1px solid #c8ad84;
  background: rgba(245, 235, 219, 0.8);
  color: #6b4f2f;
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
  color: #9a6e31;
  font-size: 21px;
  font-weight: 700;
  line-height: 1.25;
}

.runtime-quick-subtitle {
  margin: 6px 0 0;
  color: #6b655d;
  font-size: 13px;
  line-height: 1.6;
}

.runtime-quick-search-wrap {
  margin-top: 10px;
  display: grid;
  gap: 4px;
}

.runtime-quick-search-label {
  color: #6b655d;
  font-size: 12px;
  line-height: 1.4;
}

.runtime-quick-search {
  border-radius: 10px;
  border: 1px solid #d7c9b4;
  background: rgba(255, 255, 255, 0.72);
  color: #3d372f;
  padding: 7px 10px;
  font-size: 13px;
  line-height: 1.35;
}

.runtime-quick-empty {
  margin: 12px 0 0;
  color: #6b655d;
  font-size: 14px;
}

.runtime-quick-feedback {
  margin: 8px 0 0;
  color: #6b4f2f;
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
  border: 1px solid #dbcdb9;
  background: rgba(255, 255, 255, 0.68);
  padding: 10px 12px;
}

.runtime-quick-item-featured {
  border-color: #cdb179;
  background: linear-gradient(180deg, rgba(255, 251, 240, 0.88), rgba(247, 238, 218, 0.7));
}

.runtime-quick-item-selected {
  border-color: #b78c4a;
  box-shadow: 0 0 0 2px rgba(183, 140, 74, 0.2);
}

.runtime-quick-item-title {
  margin: 0;
  color: #2d2a24;
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
  border: 1px solid #c9ab75;
  background: rgba(237, 222, 190, 0.65);
  color: #7b5e33;
  padding: 1px 8px;
  font-size: 11px;
  font-weight: 600;
  line-height: 1.4;
}

.runtime-quick-mark {
  border-radius: 4px;
  background: rgba(236, 203, 132, 0.55);
  color: inherit;
  padding: 0 1px;
}

.runtime-quick-item-desc,
.runtime-quick-item-meta {
  margin: 4px 0 0;
  color: #5e5a54;
  font-size: 13px;
  line-height: 1.6;
  overflow-wrap: anywhere;
  word-break: break-word;
}

.runtime-quick-item-meta {
  color: #6f5937;
}
</style>
