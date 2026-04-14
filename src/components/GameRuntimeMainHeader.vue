<template>
  <div class="runtime-main-header">
    <div class="runtime-main-scene">
      <span class="runtime-main-chapter">{{ chapterIndexLabel }}</span>
      <span
        v-if="showSceneGlyph"
        class="runtime-main-scene-icon"
        aria-hidden="true"
      >
        <svg
          viewBox="0 0 24 24"
          class="h-4 w-4"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M4 20h16"
          />
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M6 20V9h12v11"
          />
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M12 4l7 5H5l7-5Z"
          />
        </svg>
      </span>
      <span class="runtime-main-scene-name">{{ sceneHeadlineLabel }}</span>
    </div>
    <div
      class="runtime-rhythm-badge runtime-rhythm-badge-compact"
      :class="rhythmToneClass"
    >
      <span
        class="runtime-rhythm-icon"
        aria-hidden="true"
      >
        <svg
          v-if="chapterRhythmLabel === '推演'"
          viewBox="0 0 24 24"
          class="h-4 w-4"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M4 16c2-5 4-8 8-8s6 3 8 8"
          />
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M7 10h10"
          />
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M12 6v12"
          />
        </svg>
        <svg
          v-else-if="chapterRhythmLabel === '凝思'"
          viewBox="0 0 24 24"
          class="h-4 w-4"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M6 16c1.2-2.2 2.8-3.6 6-3.6 3.4 0 5 1.4 6 3.6"
          />
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M6 12c1.2-2.2 2.8-3.6 6-3.6 3.4 0 5 1.4 6 3.6"
          />
        </svg>
        <svg
          v-else
          viewBox="0 0 24 24"
          class="h-4 w-4"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M4 12c2.8-2.2 4.4-2.2 7.2 0 2.8 2.2 4.4 2.2 7.2 0"
          />
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M6 8c1.6-1.4 2.8-1.4 4.4 0"
          />
        </svg>
      </span>
      <span class="runtime-rhythm-label">节奏</span>
      <span class="runtime-rhythm-value">{{ chapterRhythmLabel }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  chapterIndexLabel: string;
  sceneHeadlineLabel: string;
  showSceneGlyph: boolean;
  chapterRhythmLabel: string;
  rhythmToneClass: string;
}>();
</script>

<style scoped>
.runtime-main-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 0 16px;
  margin-bottom: 8px;
}

.runtime-main-scene {
  display: inline-flex;
  align-items: center;
  gap: 10px;
}

.runtime-main-chapter {
  color: var(--ink-title-color);
  font-size: 18px;
  font-weight: 600;
  letter-spacing: 0.01em;
  font-family: 'Noto Serif SC', 'Source Han Serif SC', 'Songti SC', serif;
}

.runtime-main-scene-icon {
  color: var(--ink-title-color);
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.runtime-main-scene-name {
  color: var(--ink-text-primary);
  font-size: 18px;
  font-weight: 600;
  letter-spacing: 0.005em;
  font-family: 'Noto Serif SC', 'Source Han Serif SC', 'Songti SC', serif;
}

.runtime-rhythm-badge {
  margin-top: 0;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  border: 1px solid var(--ink-border-accent);
  border-radius: 16px;
  background: var(--ink-card-bg);
  padding: 4px 12px;
}

.runtime-rhythm-badge-compact {
  margin-top: 0;
  padding: 3px 10px;
}

.runtime-rhythm-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.runtime-rhythm-label {
  color: var(--ink-text-muted);
  font-size: 12px;
  letter-spacing: 0.02em;
}

.runtime-rhythm-value {
  font-size: 14px;
  font-weight: 700;
}

.runtime-rhythm-cool .runtime-rhythm-icon,
.runtime-rhythm-cool .runtime-rhythm-value {
  color: var(--ink-text-cool);
}

.runtime-rhythm-gold .runtime-rhythm-icon,
.runtime-rhythm-gold .runtime-rhythm-value {
  color: var(--ink-title-color);
}

.runtime-rhythm-ink .runtime-rhythm-icon,
.runtime-rhythm-ink .runtime-rhythm-value {
  color: var(--ink-text-ink);
}

@media (max-width: 1180px) {
  .runtime-main-header {
    flex-wrap: wrap;
    gap: 8px;
    padding: 0 0 12px;
  }
}
</style>
