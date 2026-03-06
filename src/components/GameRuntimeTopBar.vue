<template>
  <header class="runtime-topbar">
    <div class="runtime-top-left">
      <button type="button" class="runtime-seal-btn" @click="$emit('back-to-menu')">返</button>
      <p class="runtime-brand">NOBODY</p>
    </div>
    <div class="runtime-top-center">
      <p class="runtime-top-main">
        <span class="runtime-chapter-number">{{ chapterIndexLabel }}</span>
        <span class="runtime-chapter-name">{{ chapterNameLabel }}</span>
      </p>
      <span class="runtime-state-badge" :class="interactionStateToneClass">
        <span class="runtime-state-dot" aria-hidden="true"></span>
        <span class="runtime-state-label">{{ interactionStateLabel }}</span>
      </span>
    </div>
    <div class="runtime-top-right">
      <p>{{ gameTimeLabel }}</p>
      <p class="runtime-resource-pill">灵石 · {{ spiritStoneLabel }}</p>
      <p v-if="characterCreationDurationLabel" class="runtime-top-meta">
        创建角色耗时 · {{ characterCreationDurationLabel }}
      </p>
    </div>
  </header>
</template>

<script setup lang="ts">
defineProps<{
  chapterIndexLabel: string;
  chapterNameLabel: string;
  interactionStateToneClass: string;
  interactionStateLabel: string;
  gameTimeLabel: string;
  spiritStoneLabel: string;
  characterCreationDurationLabel: string;
}>();

defineEmits<{
  'back-to-menu': [];
}>();
</script>

<style scoped>
.runtime-topbar {
  height: 56px;
  border-radius: 12px;
  border: 1px solid var(--ink-border-soft);
  border-bottom-color: var(--runtime-topbar-border-bottom);
  background: var(--ink-card-bg);
  padding: 0 18px;
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
  gap: 14px;
}

.runtime-top-left {
  display: flex;
  align-items: center;
  gap: 10px;
  justify-self: start;
}

.runtime-brand {
  margin: 0;
  color: var(--ink-text-primary);
  font-size: 19px;
  font-weight: 600;
  letter-spacing: 0.08em;
}

.runtime-seal-btn {
  width: 30px;
  height: 30px;
  border-radius: 6px;
  border: 1px solid var(--ink-accent-main);
  background: var(--runtime-seal-btn-bg);
  color: var(--ink-accent-main);
  transition: border-color 180ms ease, background-color 180ms ease, box-shadow 180ms ease, transform 120ms ease;
}

.runtime-seal-btn:hover {
  border-color: var(--runtime-btn-hover-border);
  background: var(--runtime-btn-hover-bg);
  box-shadow: var(--runtime-btn-hover-shadow);
}

.runtime-seal-btn:active {
  transform: scale(0.98);
}

.runtime-top-center {
  display: flex;
  align-items: center;
  gap: 10px;
}

.runtime-top-main {
  margin: 0;
  display: inline-flex;
  align-items: baseline;
  gap: 10px;
  font-size: 18px;
  line-height: 1.2;
}

.runtime-chapter-number {
  color: var(--ink-title-color);
  font-size: 18px;
  font-weight: 700;
}

.runtime-chapter-name {
  color: var(--ink-text-primary);
  font-size: 18px;
  font-weight: 600;
}

.runtime-state-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  border-radius: 16px;
  border: 1px solid var(--ink-border-accent);
  background: var(--ink-card-bg);
  padding: 3px 10px;
}

.runtime-state-dot {
  width: 6px;
  height: 6px;
  border-radius: 999px;
  background: currentColor;
  opacity: 0.9;
}

.runtime-state-label {
  font-size: 12px;
  font-weight: 600;
}

.runtime-state-cool {
  color: var(--ink-text-cool);
}

.runtime-state-gold {
  color: var(--ink-title-color);
}

.runtime-state-ink {
  color: var(--ink-text-ink);
}

.runtime-state-ember {
  color: var(--ink-accent-main);
}

.runtime-top-right {
  justify-self: end;
  display: flex;
  align-items: center;
  gap: 10px;
  color: var(--ink-text-muted);
  font-size: 12px;
}

.runtime-resource-pill {
  border-radius: 999px;
  border: 1px solid var(--ink-border-accent);
  background: var(--runtime-resource-pill-bg);
  color: var(--ink-title-color);
  padding: 4px 11px;
  font-size: 12px;
  line-height: 1.3;
  box-shadow: var(--runtime-resource-pill-shadow);
}

.runtime-top-meta {
  margin: 0;
  color: var(--ink-text-muted);
  font-size: 14px;
  line-height: 1.4;
}

@media (max-width: 1180px) {
  .runtime-topbar {
    grid-template-columns: 1fr;
    height: auto;
    padding: 10px 14px;
  }

  .runtime-top-right {
    justify-self: start;
  }
}
</style>
