<template>
  <section class="runtime-card">
    <h3 class="runtime-card-title">
      系统中枢
    </h3>
    <p class="runtime-chapter-title">
      <span class="runtime-chapter-number">{{ chapterIndexLabel }}</span>
      <span class="runtime-chapter-name">{{ chapterNameLabel }}</span>
    </p>
    <p class="runtime-sub-text">
      所在：{{ currentLocationLabel }}
    </p>
    <div
      class="runtime-rhythm-badge"
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
  </section>

  <section class="runtime-card">
    <h3 class="runtime-card-title">
      人物
    </h3>
    <p class="runtime-body-text">
      {{ playerRealmLabel }}
    </p>
    <div
      v-if="playerRootElements.length > 0"
      class="runtime-root-line"
    >
      <div
        v-for="item in playerRootElements"
        :key="item.element"
        class="runtime-root-item"
      >
        <span
          class="runtime-root-icon"
          :class="item.colorClass"
          aria-hidden="true"
        >
          <svg
            v-if="item.element === 'Earth'"
            viewBox="0 0 24 24"
            class="h-5 w-5"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M3 18h18L16 8h-8L3 18Z"
            />
          </svg>
          <svg
            v-else-if="item.element === 'Metal'"
            viewBox="0 0 24 24"
            class="h-5 w-5"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
          >
            <circle
              cx="12"
              cy="12"
              r="6.5"
            />
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M12 5.5v13M5.5 12h13"
            />
          </svg>
          <svg
            v-else-if="item.element === 'Wood'"
            viewBox="0 0 24 24"
            class="h-5 w-5"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M12 20V8"
            />
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M12 10c3.5 0 5-2 5-4-3 0-5 1.8-5 4Z"
            />
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M12 13c-3.5 0-5-2-5-4 3 0 5 1.8 5 4Z"
            />
          </svg>
          <svg
            v-else-if="item.element === 'Water'"
            viewBox="0 0 24 24"
            class="h-5 w-5"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M12 4c3.6 4.2 5.5 7 5.5 9.5A5.5 5.5 0 0 1 12 19a5.5 5.5 0 0 1-5.5-5.5C6.5 11 8.4 8.2 12 4Z"
            />
          </svg>
          <svg
            v-else
            viewBox="0 0 24 24"
            class="h-5 w-5"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M12 4c2.5 2 4.5 4.2 4.5 6.8 0 3.4-2.5 5.8-4.5 9.2-2-3.4-4.5-5.8-4.5-9.2C7.5 8.2 9.5 6 12 4Z"
            />
          </svg>
        </span>
        <span
          class="runtime-root-name"
          :class="item.colorClass"
        >{{ item.label }}</span>
      </div>
      <span class="runtime-root-type">{{ playerRootTypeLabel }}</span>
    </div>
    <p
      v-else
      class="runtime-sub-text"
    >
      {{ playerRootLabel }}
    </p>
  </section>
</template>

<script setup lang="ts">
type RootItem = {
  element: string;
  label: string;
  colorClass: string;
};

defineProps<{
  chapterIndexLabel: string;
  chapterNameLabel: string;
  currentLocationLabel: string;
  chapterRhythmLabel: string;
  rhythmToneClass: string;
  playerRealmLabel: string;
  playerRootElements: RootItem[];
  playerRootTypeLabel: string;
  playerRootLabel: string;
}>();
</script>

<style scoped>
.runtime-card {
  position: relative;
  border-radius: 14px;
  border: 1px solid var(--runtime-side-card-border, var(--ink-border-strong));
  background: var(--runtime-side-card-bg, var(--panel-bg, var(--ink-card-bg)));
  box-shadow: var(--runtime-side-card-shadow, var(--ink-shadow-card));
  padding: 20px;
  background-image: var(--runtime-card-sheen);
}

.runtime-card-title {
  margin: 0;
  color: var(--ink-title-color);
  font-size: 18px;
  font-weight: 600;
  letter-spacing: 0.01em;
  line-height: 1.35;
  font-family: 'Noto Serif SC', 'Source Han Serif SC', 'Songti SC', serif;
  display: inline-block;
}

.runtime-chapter-title {
  margin: 0;
  display: inline-flex;
  align-items: baseline;
  gap: 10px;
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

.runtime-sub-text {
  margin: 0;
  color: var(--ink-text-muted);
  font-size: 14px;
  line-height: 1.6;
  letter-spacing: 0.01em;
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

.runtime-body-text {
  margin: 0;
  color: var(--ink-text-primary);
  font-size: 16px;
  line-height: 1.65;
  letter-spacing: 0.005em;
}

.runtime-root-line {
  margin-top: 0;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
}

.runtime-root-item {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.runtime-root-icon {
  display: inline-flex;
  width: 20px;
  height: 20px;
  align-items: center;
  justify-content: center;
}

.runtime-root-name {
  font-size: 14px;
  line-height: 1.4;
}

.runtime-root-type {
  color: var(--ink-text-muted);
  font-size: 14px;
  line-height: 1.5;
}

.runtime-root-earth {
  color: var(--runtime-root-earth);
}

.runtime-root-metal {
  color: var(--ink-title-color);
}

.runtime-root-wood {
  color: var(--ink-text-ink);
}

.runtime-root-water {
  color: var(--runtime-root-water);
}

.runtime-root-fire {
  color: var(--ink-accent-main);
}
</style>

