<template>
  <div class="flex flex-wrap gap-2">
    <UiButton @click="$emit('open-character')">
      角色信息
    </UiButton>
    <UiButton @click="$emit('open-info')">
      信息面板
    </UiButton>
    <UiButton
      variant="primary"
      :disabled="!isGameInitialized"
      @click="$emit('open-save')"
    >
      保存
    </UiButton>
    <UiButton @click="$emit('open-load')">
      读取
    </UiButton>
  </div>
</template>

<script setup lang="ts">
import UiButton from '../shared/ui/UiButton.vue';

defineProps<{
  isGameInitialized: boolean;
}>();

defineEmits<{
  'open-character': [];
  'open-info': [];
  'open-save': [];
  'open-load': [];
}>();
</script>
