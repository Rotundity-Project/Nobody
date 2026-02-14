<template>
  <div
    v-if="errorInfo"
    class="error-display"
    :class="{ 'error-display-collapsed': isCollapsed }"
  >
    <div class="error-header" @click="isCollapsed = !isCollapsed">
      <div class="error-icon">
        <svg
          class="w-5 h-5"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
          />
        </svg>
      </div>
      <div class="error-title">{{ errorInfo.title }}</div>
      <div class="error-toggle">
        <svg
          class="w-5 h-5 transition-transform duration-200"
          :class="{ 'rotate-180': !isCollapsed }"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M19 9l-7 7-7-7"
          />
        </svg>
      </div>
    </div>

    <transition
      enter-active-class="transition-all duration-200 ease-out"
      enter-from-class="opacity-0 -translate-y-2"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition-all duration-150 ease-in"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 -translate-y-2"
    >
      <div v-if="!isCollapsed" class="error-body">
        <p class="error-message">{{ errorInfo.message }}</p>

        <div v-if="errorInfo.suggestions && errorInfo.suggestions.length > 0" class="error-suggestions">
          <p class="suggestions-title">建议操作：</p>
          <ul class="suggestions-list">
            <li v-for="(suggestion, index) in errorInfo.suggestions" :key="index">
              {{ suggestion }}
            </li>
          </ul>
        </div>

        <div class="error-actions">
          <button
            @click="handleRetry"
            class="error-action-button error-action-primary"
          >
            重试
          </button>
          <button
            @click="handleDismiss"
            class="error-action-button error-action-secondary"
          >
            关闭
          </button>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { useGameStore } from '../stores/gameStore';

interface ErrorInfo {
  title: string;
  message: string;
  suggestions?: string[];
}

const gameStore = useGameStore();
const isCollapsed = ref(false);

const errorInfo = computed<ErrorInfo | null>(() => {
  if (!gameStore.error) return null;
  return gameStore.errorMessage as ErrorInfo;
});

const handleRetry = () => {
  // 可以在这里添加重试逻辑
  gameStore.clearError();
};

const handleDismiss = () => {
  gameStore.clearError();
};
</script>

<style scoped>
.error-display {
  background: linear-gradient(135deg, rgba(239, 68, 68, 0.15) 0%, rgba(185, 28, 28, 0.1) 100%);
  border-left: 4px solid #ef4444;
  backdrop-filter: blur(8px);
  border-radius: 8px;
  overflow: hidden;
  transition: all 0.3s ease;
}

.error-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  cursor: pointer;
  user-select: none;
  transition: background-color 0.2s ease;
}

.error-header:hover {
  background-color: rgba(239, 68, 68, 0.1);
}

.error-icon {
  color: #ef4444;
  flex-shrink: 0;
}

.error-title {
  flex: 1;
  font-weight: 600;
  font-size: 14px;
  color: #fca5a5;
}

.error-toggle {
  color: rgba(248, 250, 252, 0.6);
  flex-shrink: 0;
  transition: color 0.2s ease;
}

.error-toggle:hover {
  color: #f8fafc;
}

.error-body {
  padding: 16px;
  border-top: 1px solid rgba(239, 68, 68, 0.2);
}

.error-message {
  color: #fecaca;
  font-size: 13px;
  line-height: 1.6;
  margin-bottom: 12px;
}

.error-suggestions {
  margin-bottom: 16px;
}

.suggestions-title {
  font-size: 12px;
  font-weight: 600;
  color: #fca5a5;
  margin-bottom: 8px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.suggestions-list {
  list-style: none;
  padding: 0;
  margin: 0;
  font-size: 13px;
  color: #e5e7eb;
}

.suggestions-list li {
  padding: 4px 0;
  padding-left: 16px;
  position: relative;
  line-height: 1.5;
}

.suggestions-list li::before {
  content: '•';
  position: absolute;
  left: 4px;
  color: #ef4444;
}

.error-actions {
  display: flex;
  gap: 8px;
}

.error-action-button {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  border: none;
}

.error-action-primary {
  background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
  color: white;
}

.error-action-primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(239, 68, 68, 0.4);
}

.error-action-primary:active {
  transform: translateY(0);
}

.error-action-secondary {
  background: rgba(100, 116, 139, 0.3);
  color: #e5e7eb;
}

.error-action-secondary:hover {
  background: rgba(100, 116, 139, 0.5);
}

.error-display-collapsed .error-body {
  display: none;
}
</style>
