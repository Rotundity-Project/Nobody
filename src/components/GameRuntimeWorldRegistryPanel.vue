<template>
  <section class="runtime-card">
    <div class="flex items-center justify-between gap-2">
      <h3 class="runtime-card-title">世界属性表</h3>
      <button type="button" class="runtime-bottom-btn px-2 py-1 text-[11px]" @click="emit('refresh')">
        刷新
      </button>
    </div>
    <p class="runtime-sub-text">会话：{{ sessionLabel }}</p>
    <p class="runtime-sub-text">来源：{{ sourceLabel }}</p>

    <div class="runtime-dev-muted mt-2 grid grid-cols-2 gap-2 text-xs">
      <p>人物：{{ counts.characters }}</p>
      <p>地图点：{{ counts.map_nodes }}</p>
      <p>地图边：{{ counts.map_edges }}</p>
      <p>功法：{{ counts.techniques }}</p>
      <p>背包：{{ counts.inventory_items }}</p>
      <p>势力：{{ counts.factions }}</p>
      <p>剧情态：{{ counts.story_state }}</p>
      <p>事实：{{ counts.world_facts }}</p>
    </div>

    <details class="mt-2">
      <summary class="runtime-dev-muted cursor-pointer text-xs">查看 JSON 预览</summary>
      <pre class="runtime-dev-preview runtime-dev-text mt-2 max-h-40 overflow-auto rounded-lg p-2 text-[10px] leading-4">{{ preview }}</pre>
    </details>

    <details class="mt-2">
      <summary class="runtime-dev-muted cursor-pointer text-xs">提交 Patch(JSON)</summary>
      <textarea
        :value="patchInput"
        class="runtime-dev-field runtime-dev-text mt-2 min-h-[120px] w-full rounded-lg border p-2 text-[11px] leading-4"
        spellcheck="false"
        @input="onPatchInput"
      />
      <div class="mt-2 flex items-center gap-2">
        <button
          type="button"
          class="runtime-bottom-btn px-2 py-1 text-[11px]"
          :disabled="patchSubmitting"
          @click="emit('apply-patch')"
        >
          {{ patchSubmitting ? '提交中...' : '提交补丁' }}
        </button>
        <button type="button" class="runtime-bottom-btn px-2 py-1 text-[11px]" @click="emit('reset-template')">
          重置模板
        </button>
      </div>
      <p v-if="patchError" class="runtime-dev-error mt-1 text-xs">{{ patchError }}</p>
    </details>

    <details class="mt-2">
      <summary class="runtime-dev-muted cursor-pointer text-xs">按表追加一行</summary>
      <div class="mt-2 space-y-2 text-xs">
        <label class="flex flex-col gap-1">
          <span class="runtime-dev-muted">目标表</span>
          <select
            :value="selectedTable"
            class="runtime-dev-field runtime-dev-text rounded-lg border px-2 py-1 text-[12px]"
            @change="onSelectedTableChange"
          >
            <option v-for="item in tableOptions" :key="item" :value="item">{{ item }}</option>
          </select>
        </label>

        <label class="flex flex-col gap-1">
          <span class="runtime-dev-muted">行 JSON（对象）</span>
          <textarea
            :value="rowInput"
            class="runtime-dev-field runtime-dev-text min-h-[96px] w-full rounded-lg border p-2 text-[11px] leading-4"
            spellcheck="false"
            @input="onRowInput"
          />
        </label>

        <div class="flex items-center gap-2">
          <button type="button" class="runtime-bottom-btn px-2 py-1 text-[11px]" :disabled="patchSubmitting" @click="emit('append-row')">
            追加一行
          </button>
          <button type="button" class="runtime-bottom-btn px-2 py-1 text-[11px]" @click="emit('load-first-template')">
            从首行载入模板
          </button>
          <button type="button" class="runtime-bottom-btn px-2 py-1 text-[11px]" @click="emit('load-minimal-template')">
            最小合法模板
          </button>
        </div>

        <label class="flex flex-col gap-1">
          <span class="runtime-dev-muted">目标索引（用于替换/删除）</span>
          <input
            :value="selectedIndex"
            type="number"
            min="0"
            class="runtime-dev-field runtime-dev-text w-28 rounded-lg border px-2 py-1 text-[12px]"
            @input="onSelectedIndexInput"
          />
        </label>

        <div class="flex items-center gap-2">
          <button type="button" class="runtime-bottom-btn px-2 py-1 text-[11px]" @click="emit('load-row-by-index')">载入该行</button>
          <button type="button" class="runtime-bottom-btn px-2 py-1 text-[11px]" :disabled="patchSubmitting" @click="emit('replace-row')">替换该行</button>
          <button type="button" class="runtime-bottom-btn px-2 py-1 text-[11px]" :disabled="patchSubmitting" @click="emit('delete-row')">删除该行</button>
        </div>

        <label class="flex flex-col gap-1">
          <span class="runtime-dev-muted">主键字段（用于按主键更新/新增）</span>
          <input
            :value="keyField"
            type="text"
            class="runtime-dev-field runtime-dev-text w-40 rounded-lg border px-2 py-1 text-[12px]"
            @input="onKeyFieldInput"
          />
        </label>

        <div class="flex items-center gap-2">
          <button type="button" class="runtime-bottom-btn px-2 py-1 text-[11px]" :disabled="patchSubmitting" @click="emit('upsert-by-key')">
            按主键更新/新增
          </button>
        </div>

        <p v-if="rowError" class="runtime-dev-error text-xs">{{ rowError }}</p>

        <div class="runtime-dev-preview mt-1 rounded-lg p-2">
          <p class="runtime-dev-muted text-[11px]">当前表预览（{{ rowItems.length }} 行）</p>
          <ul class="runtime-dev-text mt-1 max-h-28 overflow-auto space-y-1 text-[11px]">
            <li v-for="item in rowItemsPaged" :key="`${item.index}-${item.label}`">[{{ item.index }}] {{ item.label }}</li>
          </ul>
          <div class="mt-1 flex items-center gap-2">
            <button type="button" class="runtime-bottom-btn px-2 py-1 text-[11px]" :disabled="!canPrevPage" @click="emit('prev-page')">上一页</button>
            <button type="button" class="runtime-bottom-btn px-2 py-1 text-[11px]" :disabled="!canNextPage" @click="emit('next-page')">下一页</button>
          </div>
        </div>
      </div>
    </details>
  </section>
</template>

<script setup lang="ts">
type RegistryTable =
  | 'characters'
  | 'map_nodes'
  | 'map_edges'
  | 'techniques'
  | 'inventory_items'
  | 'factions'
  | 'story_state'
  | 'world_facts';

type RegistryCounts = {
  characters: number;
  map_nodes: number;
  map_edges: number;
  techniques: number;
  inventory_items: number;
  factions: number;
  story_state: number;
  world_facts: number;
};

type RegistryRowItem = {
  index: number;
  label: string;
};

const props = defineProps<{
  sessionLabel: string;
  sourceLabel: string;
  counts: RegistryCounts;
  preview: string;
  patchInput: string;
  patchSubmitting: boolean;
  patchError: string;
  selectedTable: RegistryTable;
  tableOptions: readonly RegistryTable[];
  rowInput: string;
  rowError: string;
  selectedIndex: number;
  keyField: string;
  rowItems: RegistryRowItem[];
  rowItemsPaged: RegistryRowItem[];
  canPrevPage: boolean;
  canNextPage: boolean;
}>();

const emit = defineEmits<{
  refresh: [];
  'reset-template': [];
  'apply-patch': [];
  'append-row': [];
  'load-first-template': [];
  'load-minimal-template': [];
  'load-row-by-index': [];
  'replace-row': [];
  'delete-row': [];
  'upsert-by-key': [];
  'prev-page': [];
  'next-page': [];
  'update:patch-input': [value: string];
  'update:selected-table': [value: RegistryTable];
  'update:row-input': [value: string];
  'update:selected-index': [value: number];
  'update:key-field': [value: string];
}>();

const onPatchInput = (event: Event) => {
  emit('update:patch-input', (event.target as HTMLTextAreaElement).value ?? '');
};

const onSelectedTableChange = (event: Event) => {
  const next = ((event.target as HTMLSelectElement).value ?? props.selectedTable) as RegistryTable;
  emit('update:selected-table', next);
};

const onRowInput = (event: Event) => {
  emit('update:row-input', (event.target as HTMLTextAreaElement).value ?? '');
};

const onSelectedIndexInput = (event: Event) => {
  emit('update:selected-index', Number((event.target as HTMLInputElement).value ?? 0));
};

const onKeyFieldInput = (event: Event) => {
  emit('update:key-field', (event.target as HTMLInputElement).value ?? '');
};
</script>

<style scoped>
.runtime-card {
  position: relative;
  border-radius: 14px;
  border: 1px solid var(--ink-border-strong);
  background: var(--panel-bg, var(--ink-card-bg));
  box-shadow: var(--ink-shadow-card);
  padding: 20px;
  background-image: var(--runtime-card-sheen);
}

.runtime-card-title {
  margin: 0;
  color: var(--ink-title-color);
  font-size: 18px;
  font-weight: 600;
  letter-spacing: 0.01em;
  line-height: 1.35;
  font-family: 'Noto Serif SC', 'Source Han Serif SC', 'Songti SC', serif;
  display: inline-block;
}

.runtime-sub-text {
  margin: 0;
  color: var(--ink-text-muted);
  font-size: 14px;
  line-height: 1.6;
  letter-spacing: 0.01em;
}

.runtime-dev-muted {
  color: var(--runtime-dev-muted);
}

.runtime-dev-text {
  color: var(--runtime-dev-text);
}

.runtime-dev-field {
  border-color: var(--runtime-dev-border);
  background: var(--runtime-dev-field-bg);
}

.runtime-dev-preview {
  background: var(--runtime-dev-preview-bg);
}

.runtime-dev-error {
  color: var(--runtime-dev-error);
}

.runtime-bottom-btn {
  border-radius: 8px;
  border: 1px solid var(--runtime-btn-border);
  border-top-color: var(--runtime-btn-border-top);
  border-bottom-color: var(--runtime-btn-border-bottom);
  background: var(--runtime-btn-bg);
  color: var(--ink-text-primary);
  padding: 8px 18px;
  box-shadow: var(--runtime-btn-shadow);
  transition: border-color 180ms ease, background-color 180ms ease, box-shadow 180ms ease, transform 120ms ease;
}

.runtime-bottom-btn:hover {
  border-color: var(--runtime-btn-hover-border);
  background: var(--runtime-btn-hover-bg);
  box-shadow: var(--runtime-btn-hover-shadow);
}

.runtime-bottom-btn:active {
  transform: scale(0.98);
}
</style>
