<template>
  <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4">
    <div class="panel-surface w-full max-w-3xl rounded-2xl p-6">
      <div class="mb-4 flex items-center justify-between">
        <h3 class="text-xl font-display text-amber-100">一致性策略</h3>
        <button class="rounded bg-slate-700 px-3 py-1 text-sm text-slate-200" @click="$emit('close')">
          关闭
        </button>
      </div>

      <div class="grid gap-4 text-sm text-slate-300 md:grid-cols-2">
        <label class="block">
          近期去重窗口
          <input v-model.number="local.recent_window" type="number" min="1" max="8" class="mt-1 w-full rounded border border-slate-600 bg-slate-800 px-3 py-2 text-white" />
        </label>
        <label class="block">
          跨章节去重窗口
          <input v-model.number="local.cross_chapter_window" type="number" min="1" max="8" class="mt-1 w-full rounded border border-slate-600 bg-slate-800 px-3 py-2 text-white" />
        </label>
        <label class="block">
          近期重复阈值 (0.5-0.999)
          <input v-model.number="local.duplicate_recent_threshold" type="number" min="0.5" max="0.999" step="0.01" class="mt-1 w-full rounded border border-slate-600 bg-slate-800 px-3 py-2 text-white" />
        </label>
        <label class="block">
          跨章节重复阈值 (0.5-0.999)
          <input v-model.number="local.duplicate_cross_chapter_threshold" type="number" min="0.5" max="0.999" step="0.01" class="mt-1 w-full rounded border border-slate-600 bg-slate-800 px-3 py-2 text-white" />
        </label>
      </div>

      <div class="mt-6 rounded-lg border border-slate-700/70 bg-slate-900/60 p-4">
        <p class="text-sm font-medium text-amber-100">规则权重</p>
        <div class="mt-3 space-y-2">
          <label v-for="item in ruleItems" :key="item.code" class="flex items-center justify-between gap-3">
            <span class="text-xs text-slate-300">{{ item.label }}</span>
            <input
              v-model.number="local.code_weights[item.code]"
              type="number"
              min="1"
              max="30"
              class="w-24 rounded border border-slate-600 bg-slate-800 px-2 py-1 text-right text-white"
            />
          </label>
        </div>
      </div>

      <div class="mt-4 rounded-lg border border-emerald-700/40 bg-emerald-900/20 p-4">
        <p class="text-sm font-medium text-emerald-200">风险分预览（模拟）</p>
        <div class="mt-2 flex flex-wrap gap-2">
          <button
            v-for="item in ruleItems"
            :key="`preview-${item.code}`"
            class="rounded border px-2 py-1 text-xs"
            :class="previewSelected[item.code] ? 'border-emerald-300 bg-emerald-700/50 text-white' : 'border-slate-600 bg-slate-800 text-slate-300'"
            @click="togglePreview(item.code)"
          >
            {{ item.label }}
          </button>
        </div>
        <p class="mt-3 text-sm text-emerald-100">当前模拟风险分：{{ previewRiskScore }}</p>
      </div>

      <button class="mt-5 w-full rounded bg-amber-500 px-4 py-2 font-medium text-slate-900" @click="handleSave">
        保存策略
      </button>
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
}>();

const ruleItems = [
  { code: 'empty_plot_text', label: '空剧情段落' },
  { code: 'duplicate_segment', label: '近期重复段落' },
  { code: 'duplicate_cross_chapter', label: '跨章节重复' },
  { code: 'waiting_without_options', label: '无选项等待输入' },
  { code: 'realm_power_conflict', label: '境界战力冲突' },
  { code: 'title_drift', label: '角色称谓漂移' },
  { code: 'location_transition_untracked', label: '地点切换未同步' },
  { code: 'chapter_goal_weak', label: '章节目标薄弱' },
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

