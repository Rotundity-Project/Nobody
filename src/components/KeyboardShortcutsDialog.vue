<template>
  <div
    v-if="isOpen"
    class="shortcuts-overlay fixed inset-0 z-50 flex items-center justify-center p-4"
    @click.self="$emit('close')"
  >
    <div class="w-full max-w-2xl panel-surface shortcuts-panel rounded-2xl p-6">
      <div class="mb-4 flex items-center justify-between">
        <h3 class="text-xl font-display shortcuts-title">键盘快捷键</h3>
        <button
          class="shortcuts-close-btn rounded px-3 py-1 text-sm"
          @click="$emit('close')"
        >
          关闭
        </button>
      </div>

      <div class="space-y-4">
        <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
          <div class="space-y-2">
            <h4 class="text-sm font-semibold shortcuts-subtitle">通用快捷键</h4>
            <div class="space-y-1">
              <div class="flex justify-between text-sm">
                <span class="shortcuts-text">关闭弹窗</span>
                <kbd class="shortcuts-kbd px-2 py-1 rounded text-xs">ESC</kbd>
              </div>
              <div class="flex justify-between text-sm">
                <span class="shortcuts-text">保存游戏</span>
                <div>
                  <kbd class="shortcuts-kbd px-2 py-1 rounded text-xs">Ctrl</kbd>
                  <kbd class="shortcuts-kbd ml-1 px-2 py-1 rounded text-xs">S</kbd>
                </div>
              </div>
            </div>
          </div>

          <div class="space-y-2">
            <h4 class="text-sm font-semibold shortcuts-subtitle">选项模式</h4>
            <div class="space-y-1">
              <div class="flex justify-between text-sm">
                <span class="shortcuts-text">选择选项 1-5</span>
                <div>
                  <kbd class="shortcuts-kbd px-2 py-1 rounded text-xs">1</kbd>
                  <kbd class="shortcuts-kbd ml-1 px-2 py-1 rounded text-xs">-5</kbd>
                </div>
              </div>
            </div>
          </div>

          <div class="space-y-2">
            <h4 class="text-sm font-semibold shortcuts-subtitle">自由输入模式</h4>
            <div class="space-y-1">
              <div class="flex justify-between text-sm">
                <span class="shortcuts-text">提交输入</span>
                <kbd class="shortcuts-kbd px-2 py-1 rounded text-xs">Enter</kbd>
              </div>
            </div>
          </div>

          <div class="space-y-2">
            <h4 class="text-sm font-semibold shortcuts-subtitle">导航</h4>
            <div class="space-y-1">
              <div class="flex justify-between text-sm">
                <span class="shortcuts-text">滚动到底部</span>
                <button
                  class="shortcuts-action-btn px-2 py-1 rounded text-xs"
                  @click="scrollToBottom"
                >
                  点击按钮
                </button>
              </div>
            </div>
          </div>
        </div>

        <p class="shortcuts-hint mt-4 text-xs">
          提示：快捷键仅在游戏界面可用。在输入框中输入时，快捷键会被禁用。
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  isOpen: boolean;
}

defineProps<Props>();
defineEmits<{ close: [] }>();

const scrollToBottom = () => {
  const storyElement = document.querySelector('[class*="overflow-y-auto"]');
  if (storyElement instanceof HTMLElement) {
    storyElement.scrollTo({
      top: storyElement.scrollHeight,
      behavior: 'smooth',
    });
  }
};
</script>

<style scoped>
.shortcuts-overlay {
  background: var(--settings-overlay-bg);
}

.shortcuts-panel {
  border: 1px solid var(--ink-border-soft);
}

.shortcuts-title,
.shortcuts-subtitle {
  color: var(--ink-title-color);
}

.shortcuts-text {
  color: var(--ink-text-primary);
}

.shortcuts-close-btn {
  border: 1px solid var(--ink-border-soft);
  background: var(--settings-btn-muted-bg);
  color: var(--ink-text-primary);
}

.shortcuts-kbd {
  border: 1px solid var(--ink-border-soft);
  background: var(--settings-input-bg);
  color: var(--ink-text-primary);
}

.shortcuts-action-btn {
  border: 1px solid var(--ink-border-accent);
  background: var(--settings-btn-accent-bg);
  color: var(--ink-text-primary);
}

.shortcuts-action-btn:hover {
  border-color: var(--ink-title-color);
  background: var(--ink-paper);
}

.shortcuts-hint {
  color: var(--ink-text-muted);
}
</style>
