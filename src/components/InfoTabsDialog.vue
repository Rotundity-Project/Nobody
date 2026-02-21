<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 z-50 bg-black/25"
    @click.self="$emit('close')"
  >
    <aside class="ink-info-drawer absolute inset-y-0 right-0 w-full sm:w-[92vw] lg:w-[68vw] xl:w-[56vw] max-w-3xl p-4 sm:p-5">
      <div class="ink-info-panel h-full overflow-y-auto rounded-2xl p-4 sm:p-5">
        <div class="mb-4 flex items-center justify-between">
          <div>
            <p class="text-xs uppercase tracking-[0.25em] text-[#6b655d]">世界层</p>
            <h3 class="text-xl font-display text-[#b78c4a]">信息抽屉</h3>
          </div>
          <UiButton size="sm" class="ink-ui-btn" @click="$emit('close')">关闭</UiButton>
        </div>

        <div class="mb-4 flex flex-wrap gap-2">
          <UiButton
            v-for="tab in tabs"
            :key="tab.id"
            size="sm"
            class="ink-ui-btn"
            :variant="activeTab === tab.id ? 'primary' : 'neutral'"
            @click="activeTab = tab.id"
          >
            {{ tab.label }}
          </UiButton>
        </div>

        <section v-if="activeTab === 'character'" class="space-y-2 text-sm text-[#4d4943]">
          <p>姓名：{{ playerName }}</p>
          <p>境界：{{ playerRealm }}</p>
          <div v-if="playerRootElementsSafe.length > 0" class="info-root-row">
            <span>灵根：</span>
            <div class="info-root-list">
              <span
                v-for="item in playerRootElementsSafe"
                :key="`${item.element}-${item.label}`"
                class="info-root-item"
              >
                <span class="info-root-icon" :class="rootElementColorClass(item.element)" aria-hidden="true">
                  <svg
                    v-if="item.element === 'Earth'"
                    viewBox="0 0 24 24"
                    class="h-5 w-5"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.8"
                  >
                    <path stroke-linecap="round" stroke-linejoin="round" d="M3 18h18L16 8h-8L3 18Z" />
                  </svg>
                  <svg
                    v-else-if="item.element === 'Metal'"
                    viewBox="0 0 24 24"
                    class="h-5 w-5"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.8"
                  >
                    <circle cx="12" cy="12" r="6.5" />
                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 5.5v13M5.5 12h13" />
                  </svg>
                  <svg
                    v-else-if="item.element === 'Wood'"
                    viewBox="0 0 24 24"
                    class="h-5 w-5"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.8"
                  >
                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 20V8" />
                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 10c3.5 0 5-2 5-4-3 0-5 1.8-5 4Z" />
                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 13c-3.5 0-5-2-5-4 3 0 5 1.8 5 4Z" />
                  </svg>
                  <svg
                    v-else-if="item.element === 'Water'"
                    viewBox="0 0 24 24"
                    class="h-5 w-5"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.8"
                  >
                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 4c3.6 4.2 5.5 7 5.5 9.5A5.5 5.5 0 0 1 12 19a5.5 5.5 0 0 1-5.5-5.5C6.5 11 8.4 8.2 12 4Z" />
                  </svg>
                  <svg
                    v-else
                    viewBox="0 0 24 24"
                    class="h-5 w-5"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.8"
                  >
                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 4c2.5 2 4.5 4.2 4.5 6.8 0 3.4-2.5 5.8-4.5 9.2-2-3.4-4.5-5.8-4.5-9.2C7.5 8.2 9.5 6 12 4Z" />
                  </svg>
                </span>
                <span class="info-root-name" :class="rootElementColorClass(item.element)">{{ item.label }}</span>
              </span>
            </div>
            <span class="text-[#6b655d]">{{ playerRootLabelSafe }}</span>
          </div>
          <p>位置：{{ playerLocation }}</p>
        </section>

        <section v-else-if="activeTab === 'progress'" class="space-y-2 text-sm text-[#4d4943]">
          <p>章节：{{ chapterProgress }}</p>
          <p>章节交互：{{ chapterInteraction }}</p>
          <p>剧情段落：{{ segmentCount }}</p>
          <p>当前状态：{{ isWaitingForInput ? '等待玩家输入' : '自动推进中' }}</p>
        </section>

        <section v-else-if="activeTab === 'map'" class="space-y-3 text-sm text-[#4d4943]">
          <p>当前位置：{{ normalizedCurrentLocationLabel }}</p>
          <p class="text-xs text-[#6b655d]">可达节点：{{ reachableNodeCount }} / {{ resolvedMapNodes.length }}</p>
          <div v-if="worldLocations.length === 0" class="text-[#6b655d]">暂无地图节点</div>
          <div v-else class="grid gap-2 sm:grid-cols-2">
            <div
              v-for="loc in resolvedMapNodes"
              :key="loc.id"
              class="rounded border border-[#d9d0c0] bg-[#f2ebdf] p-2"
            >
              <p class="font-medium text-[#2d2a24]">
                {{ loc.name }}
                <span
                  v-if="loc.id === currentLocationId"
                  class="ml-1 rounded bg-[#b78c4a] px-1.5 py-0.5 text-[10px] text-[#f8f3ea]"
                >
                  当前
                </span>
              </p>
              <p class="text-xs text-[#4d4943]">灵气强度 {{ Number(loc.spiritual_energy).toFixed(2) }} / 风险 {{ loc.riskLabel }}</p>
              <p class="text-[11px] text-[#6b655d]">灵气差 {{ Number(loc.energyGap).toFixed(2) }}</p>
              <p v-if="typeof loc.estimatedSteps === 'number'" class="text-[11px] text-[#6b655d]">预计步数 {{ loc.estimatedSteps }}</p>
              <p v-if="loc.suggestedPath.length > 1" class="text-[11px] text-[#6b655d]">建议路径 {{ loc.suggestedPathLabels.join(' -> ') }}</p>
              <p class="mt-1 text-[11px]" :class="loc.reachable ? 'text-emerald-300' : 'text-amber-300'">
                {{ loc.reachable ? '可达' : '暂不可达' }}
              </p>
              <UiButton
                v-if="loc.id !== currentLocationId"
                class="mt-2 ink-ui-btn"
                size="sm"
                variant="info"
                :disabled="isTraveling || !loc.reachable"
                @click="$emit('travel', loc.id)"
              >
                {{ loc.reachable ? '前往此地' : '需分段行进' }}
              </UiButton>
            </div>
          </div>
        </section>

        <section v-else-if="activeTab === 'review'" class="space-y-2 text-sm text-[#4d4943]">
          <p class="text-[#6b655d]">最近战斗复盘</p>
          <div v-if="recentCombatExplanations.length === 0" class="text-[#6b655d]">暂无战斗复盘记录</div>
          <ul v-else class="space-y-2">
            <li
              v-for="(item, idx) in recentCombatExplanations"
              :key="`review-${idx}-${item}`"
              class="rounded border border-[#d9d0c0] bg-[#f2ebdf] p-2 text-xs text-[#4d4943]"
            >
              {{ item }}
            </li>
          </ul>
        </section>

        <section v-else-if="activeTab === 'export'" class="space-y-2">
          <NovelExporter :is-game-running="isGameRunning" :event-count="eventCount" />
        </section>

        <section v-else-if="activeTab === 'debug'" class="space-y-2 text-xs text-[#4d4943]">
          <p v-if="!isDevMode" class="text-[#6b655d]">当前为非开发模式，调试信息已隐藏。</p>
          <template v-else>
            <p>章节：{{ debugChapter }}</p>
            <p>选项来源：{{ debugOptionSource || '无' }}</p>
            <p v-if="debugOptionHint">来源说明：{{ debugOptionHint }}</p>
            <p>等待输入：{{ isWaitingForInput ? '是' : '否' }}</p>
            <p>一致性风险分：{{ debugRiskScore ?? '无' }}</p>
            <p class="whitespace-pre-wrap text-[#6b655d]">诊断：{{ debugDiagnostics || '无' }}</p>
          </template>
        </section>

        <section v-else class="space-y-3">
          <StatusBanner v-if="systemError" kind="error" title="系统提示" :message="systemError" />
          <p v-else class="text-sm text-[#6b655d]">当前无系统提示。</p>
          <UiButton v-if="systemError" size="sm" variant="danger" class="ink-ui-btn-danger" @click="$emit('clearError')">清除提示</UiButton>
        </section>
      </div>
    </aside>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import UiButton from '../shared/ui/UiButton.vue';
import NovelExporter from './NovelExporter.vue';
import StatusBanner from './StatusBanner.vue';
import { formatLocationLabel } from '../shared/locationLabel';

type TabId = 'character' | 'progress' | 'map' | 'review' | 'export' | 'debug' | 'system';
type RootElementKey = 'Fire' | 'Water' | 'Wood' | 'Metal' | 'Earth';

type WorldLocation = {
  id: string;
  name: string;
  spiritual_energy: number;
};

type MapOverviewNodeInput = {
  location_id: string;
  name: string;
  spiritual_energy: number;
  energy_gap: number;
  reachable: boolean;
  risk_tier: string;
  estimated_steps?: number;
  suggested_path?: string[];
};

type MapOverviewNodeView = {
  id: string;
  name: string;
  spiritual_energy: number;
  energyGap: number;
  reachable: boolean;
  riskLabel: string;
  estimatedSteps?: number;
  suggestedPath: string[];
  suggestedPathLabels: string[];
};

const props = defineProps<{
  isOpen: boolean;
  playerName: string;
  playerRealm: string;
  playerCombatPower: string;
  playerRootLabel?: string;
  playerRootElements?: Array<{ element: RootElementKey; label: string }>;
  playerLocation: string;
  chapterProgress: string;
  chapterInteraction: string;
  segmentCount: number;
  isWaitingForInput: boolean;
  worldLocations: WorldLocation[];
  reachableLocationIds: string[];
  mapOverview: MapOverviewNodeInput[];
  recentCombatExplanations: string[];
  currentLocationId: string;
  currentLocationLabel: string;
  isTraveling: boolean;
  isGameRunning: boolean;
  eventCount: number;
  isDevMode: boolean;
  debugChapter: string;
  debugOptionSource: string;
  debugOptionHint?: string;
  debugRiskScore: number | null;
  debugDiagnostics: string;
  systemError: string | null;
}>();

defineEmits<{
  close: [];
  clearError: [];
  travel: [locationId: string];
}>();

const tabs: Array<{ id: TabId; label: string }> = [
  { id: 'character', label: '角色快照' },
  { id: 'progress', label: '剧情进度' },
  { id: 'map', label: '地图行程' },
  { id: 'review', label: '战斗复盘' },
  { id: 'export', label: '经历导出' },
  { id: 'debug', label: '调试上下文' },
  { id: 'system', label: '系统提示' },
];

const activeTab = ref<TabId>('character');
const playerRootElementsSafe = computed(() => props.playerRootElements ?? []);
const playerRootLabelSafe = computed(() => props.playerRootLabel ?? '灵根');

const rootElementColorClass = (element: RootElementKey): string => {
  if (element === 'Earth') return 'info-root-earth';
  if (element === 'Metal') return 'info-root-metal';
  if (element === 'Wood') return 'info-root-wood';
  if (element === 'Water') return 'info-root-water';
  return 'info-root-fire';
};

const isChineseText = (value: string): boolean => /[\u4e00-\u9fff]/.test(value);
const normalizeLocationDisplayName = (name: string, id: string): string => {
  const trimmedName = name.trim();
  if (trimmedName.length > 0 && isChineseText(trimmedName)) {
    return trimmedName;
  }
  if (trimmedName.length > 0) {
    return formatLocationLabel(trimmedName);
  }
  return formatLocationLabel(id);
};

const locationRiskLabel = (spiritualEnergy: number): string => {
  if (spiritualEnergy >= 0.8) return '高';
  if (spiritualEnergy >= 0.4) return '中';
  return '低';
};

const riskTierLabel = (riskTier: string): string => {
  if (riskTier === 'high') return '高';
  if (riskTier === 'medium') return '中';
  if (riskTier === 'low') return '低';
  return riskTier;
};

const mapOverviewNodes = (
  mapOverview: MapOverviewNodeInput[],
  worldLocations: WorldLocation[],
  currentLocationId: string,
  reachableLocationIds: string[],
): MapOverviewNodeView[] => {
  const locationNameMap = new Map<string, string>();
  for (const item of worldLocations) {
    if (item.id && item.name) {
      locationNameMap.set(item.id, normalizeLocationDisplayName(item.name, item.id));
    }
  }
  for (const item of mapOverview) {
    if (item.location_id && item.name) {
      locationNameMap.set(
        item.location_id,
        normalizeLocationDisplayName(item.name, item.location_id),
      );
    }
  }

  if (mapOverview.length > 0) {
    return mapOverview.map((item) => ({
      id: item.location_id,
      name: normalizeLocationDisplayName(item.name, item.location_id),
      spiritual_energy: item.spiritual_energy,
      energyGap: item.energy_gap,
      reachable: item.reachable || item.location_id === currentLocationId,
      riskLabel: riskTierLabel(item.risk_tier),
      estimatedSteps: item.estimated_steps,
      suggestedPath: item.suggested_path ?? [],
      suggestedPathLabels: (item.suggested_path ?? []).map(
        (pathId) => locationNameMap.get(pathId) ?? formatLocationLabel(pathId),
      ),
    }));
  }

  return worldLocations.map((loc) => ({
    id: loc.id,
    name: normalizeLocationDisplayName(loc.name, loc.id),
    spiritual_energy: loc.spiritual_energy,
    energyGap: 0,
    reachable: loc.id === currentLocationId || reachableLocationIds.includes(loc.id),
    riskLabel: locationRiskLabel(loc.spiritual_energy),
    estimatedSteps: undefined,
    suggestedPath: [],
    suggestedPathLabels: [],
  }));
};

const resolvedMapNodes = computed(() =>
  mapOverviewNodes(
    props.mapOverview,
    props.worldLocations,
    props.currentLocationId,
    props.reachableLocationIds,
  ),
);

const normalizedCurrentLocationLabel = computed(() =>
  formatLocationLabel(props.currentLocationLabel || props.playerLocation),
);

const reachableNodeCount = computed(
  () => resolvedMapNodes.value.filter((node) => node.reachable).length,
);
</script>

<style scoped>
.info-root-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}

.ink-info-drawer {
  border-left: 1px solid #d9d0c0;
  background: rgba(250, 247, 242, 0.96);
  backdrop-filter: blur(8px);
}

.ink-info-panel {
  background: #f5f0e8;
  border: 1px solid #d9d0c0;
  box-shadow: 0 10px 28px rgba(45, 42, 36, 0.08);
}

.ink-ui-btn :deep(button),
.ink-ui-btn {
  border: 1px solid #b7a88c !important;
  background: #f8f3ea !important;
  color: #2d2a24 !important;
}

.ink-ui-btn:hover :deep(button),
.ink-ui-btn:hover {
  border-color: #b78c4a !important;
  background: #faf7f2 !important;
}

.ink-ui-btn-danger :deep(button),
.ink-ui-btn-danger {
  border: 1px solid #b23e3e !important;
  background: #fdf5f4 !important;
  color: #9a3434 !important;
}

.info-root-list {
  display: inline-flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
}

.info-root-item {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.info-root-icon {
  display: inline-flex;
  width: 20px;
  height: 20px;
  align-items: center;
  justify-content: center;
}

.info-root-name {
  font-size: 13px;
  line-height: 1;
}

.info-root-earth {
  color: #9b7a56;
}

.info-root-metal {
  color: #c8a661;
}

.info-root-wood {
  color: #2f6a5d;
}

.info-root-water {
  color: #4a7fa9;
}

.info-root-fire {
  color: #c05252;
}
</style>
