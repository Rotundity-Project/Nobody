<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4"
    @click.self="$emit('close')"
  >
    <div class="panel-surface w-full max-w-4xl rounded-2xl p-6">
      <div class="mb-4 flex items-center justify-between">
        <h3 class="text-xl font-display text-amber-100">信息面板</h3>
        <button
          class="rounded bg-slate-700 px-3 py-1 text-sm text-slate-200"
          @click="$emit('close')"
        >
          关闭
        </button>
      </div>

      <div class="mb-4 flex flex-wrap gap-2">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          class="rounded px-3 py-1 text-sm transition-colors"
          :class="activeTab === tab.id ? 'bg-amber-500 text-slate-900' : 'bg-slate-700 text-slate-200 hover:bg-slate-600'"
          @click="activeTab = tab.id"
        >
          {{ tab.label }}
        </button>
      </div>

      <section
        v-if="activeTab === 'character'"
        class="space-y-2 text-sm text-slate-200"
      >
        <p>姓名：{{ playerName }}</p>
        <p>境界：{{ playerRealm }}</p>
        <p>战力：{{ playerCombatPower }}</p>
        <p>位置：{{ playerLocation }}</p>
      </section>

      <section
        v-else-if="activeTab === 'progress'"
        class="space-y-2 text-sm text-slate-200"
      >
        <p>章节：{{ chapterProgress }}</p>
        <p>章节互动：{{ chapterInteraction }}</p>
        <p>剧情段落：{{ segmentCount }}</p>
        <p>当前状态：{{ isWaitingForInput ? '等待玩家输入' : '自动推进中' }}</p>
      </section>

      <section
        v-else-if="activeTab === 'map'"
        class="space-y-3 text-sm text-slate-200"
      >
        <p>当前位置：{{ currentLocationId || playerLocation }}</p>
        <div
          v-if="worldLocations.length === 0"
          class="text-slate-400"
        >
          暂无地图节点
        </div>
        <div
          v-else
          class="grid gap-2 sm:grid-cols-2"
        >
          <div
            v-for="loc in worldLocations"
            :key="loc.id"
            class="rounded border border-slate-700 bg-slate-900/50 p-2"
          >
            <p class="font-medium text-slate-100">
              {{ loc.name || loc.id }}
              <span
                v-if="loc.id === currentLocationId"
                class="ml-1 rounded bg-amber-500 px-1.5 py-0.5 text-[10px] text-slate-900"
              >
                当前
              </span>
            </p>
            <p class="text-xs text-slate-400">id: {{ loc.id }}</p>
            <p class="text-xs text-slate-300">
              灵气强度 {{ Number(loc.spiritual_energy).toFixed(2) }} / 风险 {{ locationRiskLabel(loc.spiritual_energy) }}
            </p>
          </div>
        </div>
      </section>

      <section
        v-else-if="activeTab === 'export'"
        class="space-y-2"
      >
        <NovelExporter
          :is-game-running="isGameRunning"
          :event-count="eventCount"
        />
      </section>

      <section
        v-else-if="activeTab === 'debug'"
        class="space-y-2 text-xs text-slate-300"
      >
        <p
          v-if="!isDevMode"
          class="text-slate-400"
        >
          当前为非开发模式，调试信息已隐藏。
        </p>
        <template v-else>
          <p>章节：{{ debugChapter }}</p>
          <p>选项来源：{{ debugOptionSource || 'n/a' }}</p>
          <p>等待输入：{{ isWaitingForInput ? 'yes' : 'no' }}</p>
          <p>一致性风险分：{{ debugRiskScore ?? 'n/a' }}</p>
          <p class="whitespace-pre-wrap text-slate-400">
            诊断：{{ debugDiagnostics || '无' }}
          </p>
        </template>
      </section>

      <section
        v-else
        class="space-y-3"
      >
        <StatusBanner
          v-if="systemError"
          kind="error"
          title="系统提示"
          :message="systemError"
        />
        <p
          v-else
          class="text-sm text-slate-400"
        >
          当前无系统提示。
        </p>
        <button
          v-if="systemError"
          class="rounded bg-red-700 px-3 py-1 text-sm text-white hover:bg-red-600"
          @click="$emit('clearError')"
        >
          清除提示
        </button>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import NovelExporter from './NovelExporter.vue';
import StatusBanner from './StatusBanner.vue';

type TabId = 'character' | 'progress' | 'map' | 'export' | 'debug' | 'system';

defineProps<{
  isOpen: boolean;
  playerName: string;
  playerRealm: string;
  playerCombatPower: string;
  playerLocation: string;
  chapterProgress: string;
  chapterInteraction: string;
  segmentCount: number;
  isWaitingForInput: boolean;
  worldLocations: Array<{
    id: string;
    name: string;
    spiritual_energy: number;
  }>;
  currentLocationId: string;
  isGameRunning: boolean;
  eventCount: number;
  isDevMode: boolean;
  debugChapter: string;
  debugOptionSource: string;
  debugRiskScore: number | null;
  debugDiagnostics: string;
  systemError: string | null;
}>();

defineEmits<{
  close: [];
  clearError: [];
}>();

const tabs: Array<{ id: TabId; label: string }> = [
  { id: 'character', label: '角色快照' },
  { id: 'progress', label: '剧情进度' },
  { id: 'map', label: '地图行程' },
  { id: 'export', label: '经历导出' },
  { id: 'debug', label: 'Debug Context' },
  { id: 'system', label: '系统提示' },
];

const activeTab = ref<TabId>('character');

const locationRiskLabel = (spiritualEnergy: number): string => {
  if (spiritualEnergy >= 0.8) return '高';
  if (spiritualEnergy >= 0.4) return '中';
  return '低';
};
</script>
