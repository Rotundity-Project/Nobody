<template>
  <StatusBanner
    v-if="error"
    kind="error"
    title="系统提示"
    :message="error"
  />
  <StatusBanner
    v-else-if="showAutoAdvanceHint"
    kind="auto_advance"
    title="自动推进中"
    message="当前状态无需玩家输入。"
  />
  <StatusBanner
    v-else-if="autoAdvanceStopHint"
    kind="validation"
    title="自动推进已暂停"
    :message="autoAdvanceStopHint"
  />
</template>

<script setup lang="ts">
import StatusBanner from './StatusBanner.vue';

defineProps<{
  error: string | null;
  showAutoAdvanceHint: boolean;
  autoAdvanceStopHint: string;
}>();
</script>
