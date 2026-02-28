<template>
  <div class="runtime-bottom-bar">
    <InkQuickActionDock
      :is-game-initialized="isGameInitialized"
      @open-character="$emit('open-character')"
      @open-info="$emit('open-info')"
      @open-backpack="$emit('open-backpack')"
      @open-techniques="$emit('open-techniques')"
      @open-factions="$emit('open-factions')"
      @open-world="$emit('open-world')"
      @open-save="$emit('open-save')"
      @open-load="$emit('open-load')"
    />
    <div class="runtime-bottom-right">
      <button type="button" class="runtime-bottom-btn" @click="$emit('toggle-theme')">
        主题：{{ activeThemeLabel }}
      </button>
      <button type="button" class="runtime-bottom-btn" @click="$emit('open-settings')">系统设置</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import InkQuickActionDock from './InkQuickActionDock.vue';

defineProps<{
  isGameInitialized: boolean;
  activeThemeLabel: string;
}>();

defineEmits<{
  'open-character': [];
  'open-info': [];
  'open-backpack': [];
  'open-techniques': [];
  'open-factions': [];
  'open-world': [];
  'open-save': [];
  'open-load': [];
  'toggle-theme': [];
  'open-settings': [];
}>();
</script>

<style scoped>
.runtime-bottom-bar {
  position: relative;
  z-index: 10;
  margin-top: 18px;
  min-height: 64px;
  border-top: 1px solid var(--runtime-bottom-bar-border-top);
  border-radius: 12px;
  background: var(--panel-bg-soft, var(--ink-card-bg-soft));
  padding: 11px 18px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  pointer-events: auto;
}

.runtime-bottom-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.runtime-bottom-bar :deep(.ink-dock) {
  gap: 8px;
  padding: 0;
  background: transparent;
  border: 0;
  box-shadow: none;
}

.runtime-bottom-bar :deep(.ink-dock-btn) {
  min-width: 60px;
  border-radius: 10px;
  border: 1px solid var(--runtime-dock-btn-border);
  border-top-color: var(--runtime-dock-btn-border-top);
  border-bottom-color: var(--runtime-dock-btn-border-bottom);
  background: var(--runtime-dock-btn-bg);
  padding: 7px 14px;
  font-size: 14px;
  line-height: 1.1;
  box-shadow: var(--runtime-dock-btn-shadow);
}

.runtime-bottom-bar :deep(.ink-dock-btn:hover) {
  border-color: var(--runtime-dock-btn-hover-border);
  background: var(--runtime-dock-btn-hover-bg);
  box-shadow: var(--runtime-dock-btn-hover-shadow);
}

.runtime-bottom-bar :deep(.ink-dock-btn-primary) {
  border-color: var(--runtime-dock-btn-primary-border);
  background: var(--runtime-dock-btn-primary-bg);
  color: var(--runtime-dock-btn-primary-text);
}

.runtime-bottom-btn {
  border-radius: 8px;
  border: 1px solid var(--runtime-btn-border);
  border-top-color: var(--runtime-btn-border-top);
  border-bottom-color: var(--runtime-btn-border-bottom);
  background: var(--runtime-btn-bg);
  color: var(--ink-text-primary);
  padding: 8px 18px;
  box-shadow: var(--runtime-btn-shadow);
  transition: border-color 180ms ease, background-color 180ms ease, box-shadow 180ms ease, transform 120ms ease;
}

.runtime-bottom-btn:hover {
  border-color: var(--runtime-btn-hover-border);
  background: var(--runtime-btn-hover-bg);
  box-shadow: var(--runtime-btn-hover-shadow);
}

.runtime-bottom-btn:active {
  transform: scale(0.98);
}

@media (max-width: 1180px) {
  .runtime-bottom-bar {
    flex-direction: column;
    align-items: stretch;
  }
}
</style>
