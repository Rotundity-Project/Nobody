<template>
  <div class="space-y-3 text-center">
    <LoadingIndicator
      :message="message"
      detail="请稍候，剧情正在推进..."
      size="lg"
    />
    <div
      v-if="canStopAutoAdvance"
      class="space-y-2"
    >
      <p class="text-xs text-slate-400">自动推进中，可随时中断并回到手动控制。</p>
      <button
        class="rounded bg-slate-700 px-3 py-1 text-xs text-slate-100 transition-colors hover:bg-slate-600"
        @click="$emit('stop-auto-advance')"
      >
        中断自动推进
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import LoadingIndicator from './LoadingIndicator.vue';

withDefaults(defineProps<{
  message: string;
  canStopAutoAdvance?: boolean;
}>(), {
  canStopAutoAdvance: false,
});

defineEmits<{
  'stop-auto-advance': [];
}>();
</script>
