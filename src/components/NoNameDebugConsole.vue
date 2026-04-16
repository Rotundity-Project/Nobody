<template>
  <div
    v-if="isOpen"
    class="noname-debug-console-overlay"
    @click.self="$emit('close')"
  >
    <section class="noname-debug-console-shell">
      <header class="noname-debug-console-header">
        <div>
          <p class="noname-debug-console-eyebrow">
            NoName Agent
          </p>
          <h2 class="noname-debug-console-title">
            独立调试台
          </h2>
          <p class="noname-debug-console-meta">
            当前模式：{{ noNameMode }} · 历史 Trace：{{ traces.length }}
          </p>
        </div>
        <div class="noname-debug-console-actions">
          <button
            v-if="isDevMode"
            type="button"
            class="noname-debug-console-btn"
            @click="$emit('clear-traces')"
          >
            清空 Trace
          </button>
          <button
            type="button"
            class="noname-debug-console-btn noname-debug-console-btn-primary"
            @click="$emit('close')"
          >
            关闭
          </button>
        </div>
      </header>

      <div class="noname-debug-console-mode-strip">
        <button
          v-for="item in modeOptions"
          :key="item.value"
          type="button"
          class="noname-debug-console-mode-btn"
          :class="{ 'is-active': noNameMode === item.value }"
          @click="$emit('set-no-name-mode', item.value)"
        >
          {{ item.label }}
        </button>
      </div>

      <div class="noname-debug-console-layout">
        <aside class="noname-debug-console-sidebar">
          <p class="noname-debug-console-section-title">
            Trace 列表
          </p>
          <div
            v-if="traces.length === 0"
            class="noname-debug-console-empty"
          >
            暂无 NoName Trace。
          </div>
          <div
            v-else
            class="noname-debug-console-trace-list"
          >
            <button
              v-for="(trace, index) in traces"
              :key="trace.traceId"
              type="button"
              class="noname-debug-console-trace-btn"
              :class="{ 'is-active': index === selectedIndex }"
              @click="selectedIndex = index"
            >
              <span class="noname-debug-console-trace-title">{{ trace.traceId }}</span>
              <span class="noname-debug-console-trace-meta">
                {{ trace.mode }} · {{ trace.turnId }} · {{ trace.elapsedMs }} ms
              </span>
            </button>
          </div>
        </aside>

        <main class="noname-debug-console-main">
          <AgentTracePanel
            :trace="selectedTrace"
            :selected-index="selectedIndex"
            :total-count="traces.length"
            :active-mode="noNameMode"
          />
        </main>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import AgentTracePanel from './AgentTracePanel.vue';
import type { NoNameMode, NoNameTrace } from '../types/game';

const props = withDefaults(defineProps<{
  isOpen: boolean;
  traces: NoNameTrace[];
  noNameMode: NoNameMode;
  isDevMode?: boolean;
}>(), {
  isDevMode: false,
});

defineEmits<{
  (event: 'close'): void;
  (event: 'clear-traces'): void;
  (event: 'set-no-name-mode', mode: NoNameMode): void;
}>();

const modeOptions: Array<{ value: NoNameMode; label: string }> = [
  { value: 'disabled', label: '关闭' },
  { value: 'observeOnly', label: '观察' },
  { value: 'assisted', label: '辅助' },
];

const manualSelectedIndex = ref(-1);
const selectedIndex = computed({
  get() {
    if (props.traces.length === 0) {
      return 0;
    }
    if (manualSelectedIndex.value >= 0 && manualSelectedIndex.value < props.traces.length) {
      return manualSelectedIndex.value;
    }
    return props.traces.length - 1;
  },
  set(value: number) {
    manualSelectedIndex.value = value;
  },
});

watch(
  () => props.traces.length,
  (length) => {
    if (length === 0) {
      manualSelectedIndex.value = 0;
      return;
    }
    if (manualSelectedIndex.value >= length) {
      manualSelectedIndex.value = length - 1;
    }
  },
);

const selectedTrace = computed(() => {
  if (props.traces.length === 0) {
    return null;
  }
  const index = selectedIndex.value;
  return props.traces[index] ?? props.traces[props.traces.length - 1];
});
</script>

<style scoped>
.noname-debug-console-overlay {
  position: fixed;
  inset: 0;
  z-index: 70;
  background: rgba(17, 12, 8, 0.34);
  backdrop-filter: blur(10px);
  padding: 22px;
}

.noname-debug-console-shell {
  height: 100%;
  border-radius: 28px;
  border: 1px solid var(--ink-border-soft);
  background:
    radial-gradient(circle at top right, rgba(215, 173, 92, 0.18), transparent 28%),
    linear-gradient(180deg, rgba(255, 252, 247, 0.96), rgba(249, 244, 235, 0.97));
  box-shadow: 0 30px 80px rgba(22, 14, 8, 0.24);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.noname-debug-console-header {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: flex-start;
  padding: 22px 24px 16px;
  border-bottom: 1px solid color-mix(in srgb, var(--ink-border-soft) 76%, transparent);
}

.noname-debug-console-eyebrow {
  margin: 0;
  font-size: 12px;
  letter-spacing: 0.2em;
  text-transform: uppercase;
  color: var(--ink-text-muted);
}

.noname-debug-console-title {
  margin: 8px 0 0;
  font-size: 30px;
  color: var(--ink-title-color);
}

.noname-debug-console-meta {
  margin: 8px 0 0;
  color: var(--ink-text-muted);
}

.noname-debug-console-actions,
.noname-debug-console-mode-strip {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.noname-debug-console-actions {
  justify-content: flex-end;
}

.noname-debug-console-btn,
.noname-debug-console-mode-btn,
.noname-debug-console-trace-btn {
  border: 1px solid var(--ink-border-accent);
  background: color-mix(in srgb, var(--ink-card-bg) 90%, transparent);
  color: var(--ink-text-primary);
  transition: background-color 180ms ease, border-color 180ms ease, transform 120ms ease;
}

.noname-debug-console-btn {
  border-radius: 12px;
  padding: 10px 14px;
}

.noname-debug-console-btn-primary,
.noname-debug-console-mode-btn.is-active,
.noname-debug-console-trace-btn.is-active {
  background: color-mix(in srgb, var(--ink-accent-main) 18%, var(--ink-card-bg));
  border-color: var(--ink-accent-main);
}

.noname-debug-console-mode-strip {
  padding: 0 24px 18px;
}

.noname-debug-console-mode-btn {
  border-radius: 999px;
  padding: 8px 12px;
}

.noname-debug-console-layout {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 280px minmax(0, 1fr);
  gap: 0;
}

.noname-debug-console-sidebar {
  border-right: 1px solid color-mix(in srgb, var(--ink-border-soft) 76%, transparent);
  padding: 18px 18px 22px;
  overflow-y: auto;
}

.noname-debug-console-main {
  padding: 18px 22px 22px;
  overflow-y: auto;
}

.noname-debug-console-section-title {
  margin: 0 0 12px;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.16em;
  color: var(--ink-text-muted);
}

.noname-debug-console-empty {
  border: 1px dashed var(--ink-border-soft);
  border-radius: 16px;
  padding: 16px;
  color: var(--ink-text-muted);
}

.noname-debug-console-trace-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.noname-debug-console-trace-btn {
  text-align: left;
  border-radius: 16px;
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.noname-debug-console-trace-title {
  font-size: 14px;
}

.noname-debug-console-trace-meta {
  font-size: 12px;
  color: var(--ink-text-muted);
}

@media (max-width: 1024px) {
  .noname-debug-console-overlay {
    padding: 10px;
  }

  .noname-debug-console-layout {
    grid-template-columns: 1fr;
  }

  .noname-debug-console-sidebar {
    border-right: 0;
    border-bottom: 1px solid color-mix(in srgb, var(--ink-border-soft) 76%, transparent);
  }
}
</style>
