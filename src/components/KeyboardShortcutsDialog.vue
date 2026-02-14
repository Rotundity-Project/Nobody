<template>
  <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4" @click.self="$emit('close')">
    <div class="w-full max-w-2xl panel-surface rounded-2xl p-6">
      <div class="mb-4 flex items-center justify-between">
        <h3 class="text-xl font-display text-amber-100">键盘快捷键</h3>
        <button class="rounded bg-slate-700 px-3 py-1 text-sm text-slate-200" @click="$emit('close')">关闭</button>
      </div>

      <div class="space-y-4">
        <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
          <div class="space-y-2">
            <h4 class="text-sm font-semibold text-amber-200">通用快捷键</h4>
            <div class="space-y-1">
              <div class="flex justify-between text-sm">
                <span class="text-slate-300">关闭弹窗</span>
                <kbd class="px-2 py-1 bg-slate-800 rounded text-slate-200 text-xs">ESC</kbd>
              </div>
              <div class="flex justify-between text-sm">
                <span class="text-slate-300">保存游戏</span>
                <div>
                  <kbd class="px-2 py-1 bg-slate-800 rounded text-slate-200 text-xs">Ctrl</kbd>
                  <kbd class="px-2 py-1 bg-slate-800 rounded text-slate-200 text-xs ml-1">S</kbd>
                </div>
              </div>
            </div>
          </div>

          <div class="space-y-2">
            <h4 class="text-sm font-semibold text-amber-200">选项模式</h4>
            <div class="space-y-1">
              <div class="flex justify-between text-sm">
                <span class="text-slate-300">选择选项 1-5</span>
                <div>
                  <kbd class="px-2 py-1 bg-slate-800 rounded text-slate-200 text-xs">1</kbd>
                  <kbd class="px-2 py-1 bg-slate-800 rounded text-slate-200 text-xs ml-1">-5</kbd>
                </div>
              </div>
            </div>
          </div>

          <div class="space-y-2">
            <h4 class="text-sm font-semibold text-amber-200">自由输入模式</h4>
            <div class="space-y-1">
              <div class="flex justify-between text-sm">
                <span class="text-slate-300">提交输入</span>
                <kbd class="px-2 py-1 bg-slate-800 rounded text-slate-200 text-xs">Enter</kbd>
              </div>
            </div>
          </div>

          <div class="space-y-2">
            <h4 class="text-sm font-semibold text-amber-200">导航</h4>
            <div class="space-y-1">
              <div class="flex justify-between text-sm">
                <span class="text-slate-300">滚动到底部</span>
                <button
                  @click="scrollToBottom"
                  class="px-2 py-1 bg-amber-600 rounded text-white text-xs hover:bg-amber-500"
                >
                  点击按钮
                </button>
              </div>
            </div>
          </div>
        </div>

        <p class="text-xs text-slate-500 mt-4">
          提示：快捷键仅在游戏界面可用。在输入框中输入时，快捷键会被禁用。
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';

interface Props {
  isOpen: boolean;
}

defineProps<Props>();
defineEmits<{ close: [] }>();

const router = useRouter();

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
