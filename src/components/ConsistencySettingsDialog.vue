<template>
  <div v-if="isOpen" class="consistency-overlay fixed inset-0 z-50 flex items-center justify-center p-4">
    <div class="panel-surface consistency-panel w-full max-w-3xl rounded-2xl p-6">
      <div class="mb-4 flex items-center justify-between">
        <h3 class="text-xl font-display consistency-title">一致性策略</h3>
        <button class="consistency-close-btn rounded px-3 py-1 text-sm" @click="$emit('close')">
          关闭
        </button>
      </div>

      <div class="consistency-label grid gap-4 text-sm md:grid-cols-2">
        <label class="block">
          近期去重窗口
          <input v-model.number="local.recent_window" type="number" min="1" max="8" class="consistency-input mt-1 w-full rounded px-3 py-2" />
        </label>
        <label class="block">
          跨章节去重窗口
          <input v-model.number="local.cross_chapter_window" type="number" min="1" max="8" class="consistency-input mt-1 w-full rounded px-3 py-2" />
        </label>
        <label class="block">
          近期重复阈值 (0.5-0.999)
          <input v-model.number="local.duplicate_recent_threshold" type="number" min="0.5" max="0.999" step="0.01" class="consistency-input mt-1 w-full rounded px-3 py-2" />
        </label>
        <label class="block">
          跨章节重复阈值 (0.5-0.999)
          <input v-model.number="local.duplicate_cross_chapter_threshold" type="number" min="0.5" max="0.999" step="0.01" class="consistency-input mt-1 w-full rounded px-3 py-2" />
        </label>
      </div>

      <div class="consistency-card mt-6 rounded-lg p-4">
        <p class="text-sm font-medium consistency-title">规则权重</p>
        <div class="mt-3 space-y-2">
          <label v-for="item in ruleItems" :key="item.code" class="flex items-center justify-between gap-3">
            <span class="text-xs consistency-label">{{ item.label }}</span>
            <input
              v-model.number="local.code_weights[item.code]"
              type="number"
              min="1"
              max="30"
              class="consistency-input w-24 rounded px-2 py-1 text-right"
            />
          </label>
        </div>
      </div>

      <div class="consistency-preview-card mt-4 rounded-lg p-4">
        <p class="text-sm font-medium consistency-preview-title">风险分预览（模拟）</p>
        <div class="mt-2 flex flex-wrap gap-2">
          <button
            v-for="item in ruleItems"
            :key="`preview-${item.code}`"
            class="consistency-preview-chip rounded border px-2 py-1 text-xs"
            :class="previewSelected[item.code] ? 'consistency-preview-chip-active' : 'consistency-preview-chip-default'"
            @click="togglePreview(item.code)"
          >
            {{ item.label }}
          </button>
        </div>
        <p class="mt-3 text-sm consistency-preview-value">当前模拟风险分：{{ previewRiskScore }}</p>
      </div>

      <div class="mt-5 grid grid-cols-2 gap-3">
        <button class="consistency-reset-btn w-full rounded px-4 py-2 font-medium" @click="emit('reset')">
          恢复默认
        </button>
        <button class="consistency-save-btn w-full rounded px-4 py-2 font-medium" @click="handleSave">
          保存策略
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, watch } from 'vue';
import type { ConsistencyPolicy } from '../types/game';

const props = defineProps<{
  isOpen: boolean;
  policy: ConsistencyPolicy;
}>();

const emit = defineEmits<{
  close: [];
  save: [policy: ConsistencyPolicy];
  reset: [];
}>();

const ruleItems = [
  { code: 'empty_plot_text', label: '空剧情段落' },
  { code: 'duplicate_segment', label: '近期重复段落' },
  { code: 'duplicate_cross_chapter', label: '跨章节重复' },
  { code: 'waiting_without_options', label: '无选项等待输入' },
  { code: 'realm_power_conflict', label: '境界战力冲突' },
  { code: 'title_drift', label: '角色称谓漂移' },
  { code: 'location_transition_untracked', label: '地点切换未同步' },
  { code: 'chapter_goal_weak', label: '章节目标偏弱' },
  { code: 'chapter_summary_missing', label: '章节摘要缺失' },
];

const local = reactive<ConsistencyPolicy>({
  recent_window: props.policy.recent_window,
  cross_chapter_window: props.policy.cross_chapter_window,
  duplicate_recent_threshold: props.policy.duplicate_recent_threshold,
  duplicate_cross_chapter_threshold: props.policy.duplicate_cross_chapter_threshold,
  weight_warning: props.policy.weight_warning,
  weight_critical: props.policy.weight_critical,
  code_weights: { ...props.policy.code_weights },
});

const previewSelected = reactive<Record<string, boolean>>({});

const syncLocal = (next: ConsistencyPolicy) => {
  local.recent_window = next.recent_window;
  local.cross_chapter_window = next.cross_chapter_window;
  local.duplicate_recent_threshold = next.duplicate_recent_threshold;
  local.duplicate_cross_chapter_threshold = next.duplicate_cross_chapter_threshold;
  local.weight_warning = next.weight_warning;
  local.weight_critical = next.weight_critical;
  local.code_weights = { ...next.code_weights };
};

watch(
  () => props.policy,
  (next) => {
    syncLocal(next);
  },
  { deep: true },
);

const togglePreview = (code: string) => {
  previewSelected[code] = !previewSelected[code];
};

const previewRiskScore = computed(() =>
  ruleItems.reduce((acc, item) => {
    if (!previewSelected[item.code]) {
      return acc;
    }
    return acc + Number(local.code_weights[item.code] ?? local.weight_warning);
  }, 0),
);

const handleSave = () => {
  emit('save', {
    ...local,
    code_weights: { ...local.code_weights },
  });
  emit('close');
};
</script>

<style scoped>
.consistency-overlay {
  background: var(--settings-overlay-bg);
}

.consistency-panel {
  border: 1px solid var(--ink-border-soft);
}

.consistency-title {
  color: var(--ink-title-color);
}

.consistency-label {
  color: var(--ink-text-primary);
}

.consistency-close-btn {
  border: 1px solid var(--ink-border-soft);
  background: var(--settings-btn-muted-bg);
  color: var(--ink-text-primary);
}

.consistency-input {
  border: 1px solid var(--ink-border-soft);
  background: var(--settings-input-bg);
  color: var(--ink-text-primary);
}

.consistency-card {
  border: 1px solid var(--ink-border-soft);
  background: var(--settings-card-bg);
}

.consistency-preview-card {
  border: 1px solid var(--settings-preview-border);
  background: var(--settings-preview-bg);
}

.consistency-preview-title {
  color: var(--settings-preview-title);
}

.consistency-preview-chip {
  transition: background-color 160ms ease, border-color 160ms ease;
}

.consistency-preview-chip-default {
  border-color: var(--ink-border-soft);
  background: var(--settings-chip-bg);
  color: var(--ink-text-primary);
}

.consistency-preview-chip-active {
  border-color: var(--settings-chip-active-border);
  background: var(--settings-chip-active-bg);
  color: var(--ink-text-primary);
}

.consistency-preview-value {
  color: var(--settings-preview-value);
}

.consistency-reset-btn {
  border: 1px solid var(--ink-border-soft);
  background: var(--settings-btn-muted-bg);
  color: var(--ink-text-primary);
}

.consistency-save-btn {
  border: 1px solid var(--ink-border-accent);
  background: var(--settings-btn-accent-strong-bg);
  color: var(--ink-text-primary);
}
</style>
