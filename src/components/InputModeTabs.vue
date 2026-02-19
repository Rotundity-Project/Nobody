<template>
  <div v-if="visible" class="flex items-center gap-2">
    <button
      class="rounded px-3 py-1"
      :class="mode === 'options' ? 'bg-amber-500 text-slate-900' : 'bg-slate-700 text-gray-300'"
      @click="$emit('switch-mode', 'options')"
    >
      选项
    </button>
    <button
      class="rounded px-3 py-1"
      :class="mode === 'freeText' ? 'bg-amber-500 text-slate-900' : 'bg-slate-700 text-gray-300'"
      @click="$emit('switch-mode', 'freeText')"
    >
      自由输入
    </button>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  visible: boolean;
  mode: 'options' | 'freeText';
}>();

defineEmits<{
  'switch-mode': [mode: 'options' | 'freeText'];
}>();
</script>
