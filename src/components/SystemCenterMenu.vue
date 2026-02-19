<template>
  <div ref="menuRootRef" class="relative">
    <button
      class="rounded-lg bg-slate-700 px-4 py-2 text-white transition-colors duration-200 hover:bg-slate-600"
      @click="$emit('toggle-menu')"
    >
      系统中心
    </button>
    <div
      v-if="isOpen"
      class="panel-surface absolute right-0 z-20 mt-2 w-72 space-y-2 rounded-xl p-4"
    >
      <button
        class="w-full rounded-md bg-slate-800 px-3 py-2 text-left text-sm text-slate-100 transition-colors hover:bg-slate-700"
        @click="$emit('open-shortcuts')"
      >
        快捷键
      </button>
      <button
        class="w-full rounded-md bg-emerald-600 px-3 py-2 text-left text-sm text-slate-900 transition-colors hover:bg-emerald-500"
        @click="$emit('open-llm')"
      >
        LLM 设置
      </button>
      <button
        class="w-full rounded-md bg-slate-800 px-3 py-2 text-left text-sm text-slate-100 transition-colors hover:bg-slate-700"
        @click="$emit('open-story-settings')"
      >
        剧情设置
      </button>
      <button
        class="w-full rounded-md bg-slate-800 px-3 py-2 text-left text-sm text-slate-100 transition-colors hover:bg-slate-700"
        @click="$emit('open-consistency')"
      >
        一致性设置
      </button>
      <button
        class="w-full rounded-md bg-slate-800 px-3 py-2 text-left text-sm text-slate-100 transition-colors hover:bg-slate-700"
        @click="$emit('toggle-audio')"
      >
        音量设置
      </button>
      <div
        v-if="showAudioPanel"
        class="mt-1 rounded-lg border border-slate-700 bg-slate-900/50 p-3"
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
