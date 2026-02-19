<template>
  <SaveLoadDialog
    :is-open="showSaveDialog"
    mode="save"
    @close="$emit('close-save')"
    @saved="$emit('saved', $event)"
  />

  <SaveLoadDialog
    :is-open="showLoadDialog"
    mode="load"
    @close="$emit('close-load')"
    @loaded="$emit('loaded', $event)"
  />

  <KeyboardShortcutsDialog
    :is-open="showShortcutsDialog"
    @close="$emit('close-shortcuts')"
  />
  <LLMConfigDialog
    :is-open="showLLMDialog"
    @close="$emit('close-llm')"
  />
  <StorySettingsDialog
    :is-open="showStorySettings"
    :settings="storySettings"
    @close="$emit('close-story-settings')"
    @save="$emit('save-story-settings', $event)"
  />
  <ConsistencySettingsDialog
    :is-open="showConsistencySettings"
    :policy="consistencyPolicy"
    @close="$emit('close-consistency')"
    @save="$emit('save-consistency', $event)"
    @reset="$emit('reset-consistency')"
  />
</template>

<script setup lang="ts">
import type { ConsistencyPolicy } from '../types/game';
import type { StorySettings } from '../utils/storySettings';
import ConsistencySettingsDialog from './ConsistencySettingsDialog.vue';
import KeyboardShortcutsDialog from './KeyboardShortcutsDialog.vue';
import LLMConfigDialog from './LLMConfigDialog.vue';
import SaveLoadDialog from './SaveLoadDialog.vue';
import StorySettingsDialog from './StorySettingsDialog.vue';

defineProps<{
  showSaveDialog: boolean;
  showLoadDialog: boolean;
  showShortcutsDialog: boolean;
  showLLMDialog: boolean;
  showStorySettings: boolean;
  showConsistencySettings: boolean;
  storySettings: StorySettings;
  consistencyPolicy: ConsistencyPolicy;
}>();

defineEmits<{
  (event: 'close-save'): void;
  (event: 'saved', slotId: number): void;
  (event: 'close-load'): void;
  (event: 'loaded', slotId: number): void;
  (event: 'close-shortcuts'): void;
  (event: 'close-llm'): void;
  (event: 'close-story-settings'): void;
  (event: 'save-story-settings', settings: StorySettings): void;
  (event: 'close-consistency'): void;
  (event: 'save-consistency', policy: ConsistencyPolicy): void;
  (event: 'reset-consistency'): void;
}>();
</script>
