<template>
  <div
    ref="menuRootRef"
    class="relative"
  >
    <button
      type="button"
      class="ink-menu-trigger"
      @click="$emit('toggle-menu')"
    >
      系统中枢
    </button>
    <div
      v-if="isOpen"
      class="ink-menu-panel absolute right-0 z-20 mt-2 w-72 space-y-2 rounded-xl p-4"
    >
      <button
        type="button"
        class="ink-menu-item"
        @click="$emit('open-shortcuts')"
      >
        快捷键
      </button>
      <button
        type="button"
        class="ink-menu-item ink-menu-item-accent"
        @click="$emit('open-llm')"
      >
        LLM 设置
      </button>
      <button
        type="button"
        class="ink-menu-item"
        @click="$emit('open-story-settings')"
      >
        剧情设置
      </button>
      <button
        type="button"
        class="ink-menu-item"
        @click="$emit('open-consistency')"
      >
        一致性设置
      </button>
      <button
        type="button"
        class="ink-menu-item"
        @click="$emit('toggle-audio')"
      >
        音量设置
      </button>
      <div
        v-if="showAudioPanel"
        class="mt-1 rounded-lg border border-black/10 bg-black/5 p-3"
      >
        <AudioControlPanel />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import AudioControlPanel from './AudioControlPanel.vue';

const props = defineProps<{
  isOpen: boolean;
  showAudioPanel: boolean;
}>();

const emit = defineEmits<{
  'toggle-menu': [];
  'close-menu': [];
  'toggle-audio': [];
  'open-shortcuts': [];
  'open-llm': [];
  'open-story-settings': [];
  'open-consistency': [];
}>();

const menuRootRef = ref<HTMLElement | null>(null);

const handleDocumentClick = (event: MouseEvent) => {
  if (!props.isOpen) return;
  const target = event.target as Node | null;
  if (menuRootRef.value && target && !menuRootRef.value.contains(target)) {
    emit('close-menu');
  }
};

onMounted(() => {
  window.addEventListener('mousedown', handleDocumentClick);
});

onUnmounted(() => {
  window.removeEventListener('mousedown', handleDocumentClick);
});
</script>
