<template>
  <div class="text-center">
    <LoadingIndicator
      :message="message"
      detail="请稍候，剧情正在推进..."
      size="lg"
    />
  </div>
</template>

<script setup lang="ts">
import LoadingIndicator from './LoadingIndicator.vue';

defineProps<{
  message: string;
}>();
</script>
