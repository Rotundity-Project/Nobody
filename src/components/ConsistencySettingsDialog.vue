<template>
  <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4">
    <div class="panel-surface w-full max-w-xl rounded-2xl p-6">
      <div class="mb-4 flex items-center justify-between">
        <h3 class="text-xl font-display text-amber-100">一致性策略</h3>
        <button class="rounded bg-slate-700 px-3 py-1 text-sm text-slate-200" @click="$emit('close')">
          关闭
        </button>
      </div>

      <div class="space-y-4 text-sm text-slate-300">
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

      <button class="mt-5 w-full rounded bg-amber-500 px-4 py-2 font-medium text-slate-900" @click="handleSave">
        保存策略
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, watch } from 'vue';
import type { ConsistencyPolicy } from '../types/game';

const props = defineProps<{
  isOpen: boolean;
  policy: ConsistencyPolicy;
}>();

const emit = defineEmits<{
  close: [];
  save: [policy: ConsistencyPolicy];
}>();

const local = reactive<ConsistencyPolicy>({
  recent_window: props.policy.recent_window,
  cross_chapter_window: props.policy.cross_chapter_window,
  duplicate_recent_threshold: props.policy.duplicate_recent_threshold,
  duplicate_cross_chapter_threshold: props.policy.duplicate_cross_chapter_threshold,
  weight_warning: props.policy.weight_warning,
  weight_critical: props.policy.weight_critical,
  code_weights: { ...props.policy.code_weights },
});

watch(
  () => props.policy,
  (next) => {
    local.recent_window = next.recent_window;
    local.cross_chapter_window = next.cross_chapter_window;
    local.duplicate_recent_threshold = next.duplicate_recent_threshold;
    local.duplicate_cross_chapter_threshold = next.duplicate_cross_chapter_threshold;
    local.weight_warning = next.weight_warning;
    local.weight_critical = next.weight_critical;
    local.code_weights = { ...next.code_weights };
  },
  { deep: true },
);

const handleSave = () => {
  emit('save', {
    ...local,
    code_weights: { ...local.code_weights },
  });
  emit('close');
};
</script>

