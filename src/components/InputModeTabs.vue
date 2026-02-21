<template>
  <div
    v-if="visible"
    class="inline-flex items-center gap-2 rounded-[10px] border border-[#d9c0b0] bg-[#f5f0e8] p-1"
  >
    <button
      type="button"
      class="ink-tab-btn rounded-[8px] px-3 py-1 text-sm"
      :class="mode === 'options' ? 'ink-tab-btn-active' : 'ink-tab-btn-idle'"
      @click="$emit('switch-mode', 'options')"
    >
      选项
    </button>
    <button
      type="button"
      class="ink-tab-btn rounded-[8px] px-3 py-1 text-sm"
      :class="mode === 'freeText' ? 'ink-tab-btn-active' : 'ink-tab-btn-idle'"
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

<style scoped>
.ink-tab-btn {
  border: 1px solid transparent;
  transition: border-color 180ms ease, background-color 180ms ease, color 180ms ease;
}

.ink-tab-btn-active {
  border-color: #b78c4a;
  background: #faf7f2;
  color: #2d2a24;
}

.ink-tab-btn-idle {
  color: #5e5a54;
}

.ink-tab-btn-idle:hover {
  border-color: #d9c0b0;
  background: #f8f3ea;
  color: #2d2a24;
}
</style>
