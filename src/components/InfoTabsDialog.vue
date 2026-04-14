<template>
  <div
    v-if="isOpen"
    class="info-overlay fixed inset-0 z-50"
    @click.self="$emit('close')"
  >
    <aside class="ink-info-drawer absolute inset-y-0 right-0 w-full sm:w-[92vw] lg:w-[68vw] xl:w-[56vw] max-w-3xl p-4 sm:p-5">
      <div class="ink-info-panel h-full overflow-y-auto rounded-2xl p-4 sm:p-5">
        <div class="mb-4 flex items-center justify-between">
          <div>
            <p class="info-text-muted text-xs uppercase tracking-[0.25em]">
              世界层
            </p>
            <h3 class="info-title text-xl font-display">
              信息抽屉
            </h3>
          </div>
          <UiButton
            size="sm"
            class="ink-ui-btn"
            @click="$emit('close')"
          >
            关闭
          </UiButton>
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

        <section
          v-if="activeTab === 'character'"
          class="info-text-body space-y-2 text-sm"
        >
          <p>姓名：{{ playerName }}</p>
          <p>境界：{{ playerRealm }}</p>
          <div
            v-if="playerRootElementsSafe.length > 0"
            class="info-root-row"
          >
            <span>灵根：</span>
            <div class="info-root-list">
              <span
                v-for="item in playerRootElementsSafe"
                :key="`${item.element}-${item.label}`"
                class="info-root-item"
              >
                <span
                  class="info-root-icon"
                  :class="rootElementColorClass(item.element)"
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
                  class="info-root-name"
                  :class="rootElementColorClass(item.element)"
                >{{ item.label }}</span>
              </span>
            </div>
            <span class="info-text-muted">{{ playerRootLabelSafe }}</span>
          </div>
          <p>位置：{{ playerLocation }}</p>
        </section>

        <section
          v-else-if="activeTab === 'progress'"
          class="info-text-body space-y-2 text-sm"
        >
          <p>章节：{{ chapterProgress }}</p>
          <p>章节交互：{{ chapterInteraction }}</p>
          <p>剧情段落：{{ segmentCount }}</p>
          <p>当前状态：{{ isWaitingForInput ? '等待玩家输入' : '自动推进中' }}</p>
        </section>

        <section
          v-else-if="activeTab === 'map'"
          class="info-text-body space-y-3 text-sm"
        >
          <p>当前位置：{{ normalizedCurrentLocationLabel }}</p>
          <p class="info-text-muted text-xs">
            可达节点：{{ reachableNodeCount }} / {{ resolvedMapNodes.length }}
          </p>
          <div
            v-if="worldLocations.length === 0"
            class="info-text-muted"
          >
            暂无地图节点
          </div>
          <div
            v-else
            class="grid gap-2 sm:grid-cols-2"
          >
            <div
              v-for="loc in resolvedMapNodes"
              :key="loc.id"
              class="info-map-card rounded border p-2"
            >
              <p class="info-text-strong font-medium">
                {{ loc.name }}
                <span
                  v-if="loc.id === currentLocationId"
                  class="info-current-badge ml-1 rounded px-1.5 py-0.5 text-[10px]"
                >
                  当前
                </span>
              </p>
              <p class="info-text-body text-xs">
                灵气强度 {{ Number(loc.spiritual_energy).toFixed(2) }} / 风险 {{ loc.riskLabel }}
              </p>
              <p class="info-text-muted text-[11px]">
                灵气差 {{ Number(loc.energyGap).toFixed(2) }}
              </p>
              <p
                v-if="typeof loc.estimatedSteps === 'number'"
                class="info-text-muted text-[11px]"
              >
                预计步数 {{ loc.estimatedSteps }}
              </p>
              <p
                v-if="loc.suggestedPath.length > 1"
                class="info-text-muted text-[11px]"
              >
                建议路径 {{ loc.suggestedPathLabels.join(' -> ') }}
              </p>
              <p
                class="mt-1 text-[11px]"
                :class="loc.reachable ? 'info-reach-ok' : 'info-reach-blocked'"
              >
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

        <section
          v-else-if="activeTab === 'review'"
          class="info-text-body space-y-2 text-sm"
        >
          <p class="info-text-muted">
            最近战斗复盘
          </p>
          <div
            v-if="recentCombatExplanations.length === 0"
            class="info-text-muted"
          >
            暂无战斗复盘记录
          </div>
          <ul
            v-else
            class="space-y-2"
          >
            <li
              v-for="(item, idx) in recentCombatExplanations"
              :key="`review-${idx}-${item}`"
              class="info-map-card info-text-body rounded border p-2 text-xs"
            >
              {{ item }}
            </li>
          </ul>
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
          class="info-text-body space-y-2 text-xs"
        >
          <p
            v-if="!isDevMode"
            class="info-text-muted"
          >
            当前为非开发模式，调试信息已隐藏。
          </p>
          <template v-else>
            <p>章节：{{ debugChapter }}</p>
            <p>选项来源：{{ debugOptionSource || '无' }}</p>
            <p v-if="debugOptionHint">
              来源说明：{{ debugOptionHint }}
            </p>
            <p>等待输入：{{ isWaitingForInput ? '是' : '否' }}</p>
            <p>一致性风险分：{{ debugRiskScore ?? '无' }}</p>
            <div class="info-map-card rounded border p-2">
              <p class="info-text-muted text-xs">
                NoName 模式
              </p>
              <div class="mt-2 flex flex-wrap gap-2">
                <UiButton
                  v-for="item in noNameModeOptions"
                  :key="item.value"
                  size="sm"
                  class="ink-ui-btn"
                  :variant="noNameMode === item.value ? 'primary' : 'neutral'"
                  @click="$emit('setNoNameMode', item.value)"
                >
                  {{ item.label }}
                </UiButton>
              </div>
            </div>
            <div class="grid gap-2 sm:grid-cols-2">
              <div class="info-map-card rounded border p-2">
                <p class="info-text-muted text-xs">
                  最新提案
                </p>
                <p class="mt-1 text-sm">
                  {{ debugNoNameProposalTitle || '无' }}
                </p>
                <p class="info-text-muted mt-1">
                  状态：{{ debugNoNameProposalStatus || '无' }}
                </p>
                <p class="info-text-muted mt-1">
                  目标段：{{ debugNoNameTargetSegment || '无' }}
                </p>
                <p class="info-text-muted mt-1 whitespace-pre-wrap">
                  预期效果：{{ debugNoNameIntendedEffect || '无' }}
                </p>
                <p class="info-text-muted mt-1">
                  作用域：{{ debugNoNameScopes.length > 0 ? debugNoNameScopes.join(' / ') : '无' }}
                </p>
              </div>
              <div class="info-map-card rounded border p-2">
                <p class="info-text-muted text-xs">
                  Apply 结果
                </p>
                <p class="mt-1 text-sm">
                  {{ debugNoNameApplyOutcome || '无' }}
                </p>
                <p
                  v-if="debugNoNameApplyReason"
                  class="info-text-muted mt-1 whitespace-pre-wrap"
                >
                  {{ debugNoNameApplyReason }}
                </p>
              </div>
            </div>
            <div class="info-map-card rounded border p-2">
              <p class="info-text-muted text-xs">
                状态迁移
              </p>
              <p
                v-if="debugNoNameTransitions.length === 0"
                class="mt-2 text-xs info-text-muted"
              >
                暂无状态迁移
              </p>
              <ul
                v-else
                class="mt-2 space-y-1"
              >
                <li
                  v-for="(item, index) in debugNoNameTransitions"
                  :key="`transition-${index}-${item}`"
                  class="rounded border px-2 py-1 info-text-body"
                >
                  {{ item }}
                </li>
              </ul>
            </div>
            <div class="info-map-card rounded border p-2">
              <p class="info-text-muted text-xs">
                应用计划
              </p>
              <p
                v-if="debugNoNamePlans.length === 0"
                class="mt-2 text-xs info-text-muted"
              >
                暂无应用计划
              </p>
              <ul
                v-else
                class="mt-2 space-y-1"
              >
                <li
                  v-for="(item, index) in debugNoNamePlans"
                  :key="`plan-${index}-${item.target}-${item.decision}`"
                  class="rounded border px-2 py-1"
                >
                  <p class="info-text-body">
                    #{{ item.order }} · {{ item.target }} · {{ item.decision }} · P{{ item.priority }}
                  </p>
                  <p
                    v-if="item.note"
                    class="info-text-muted mt-1 whitespace-pre-wrap"
                  >
                    {{ item.note }}
                  </p>
                </li>
              </ul>
            </div>
            <div class="info-map-card rounded border p-2">
              <p class="info-text-muted text-xs">
                应用执行
              </p>
              <p
                v-if="debugNoNameExecutions.length === 0"
                class="mt-2 text-xs info-text-muted"
              >
                暂无应用执行
              </p>
              <ul
                v-else
                class="mt-2 space-y-1"
              >
                <li
                  v-for="(item, index) in debugNoNameExecutions"
                  :key="`execution-${index}-${item.target}-${item.outcome}`"
                  class="rounded border px-2 py-1"
                >
                  <p class="info-text-body">
                    {{ item.target }} · {{ item.outcome }}
                  </p>
                  <p
                    v-if="item.note"
                    class="info-text-muted mt-1 whitespace-pre-wrap"
                  >
                    {{ item.note }}
                  </p>
                </li>
              </ul>
            </div>
            <p class="info-text-muted whitespace-pre-wrap">
              诊断：{{ debugDiagnostics || '无' }}
            </p>
            <p class="info-text-muted whitespace-pre-wrap">
              Agent：{{ debugNoNameTrace || '无' }}
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
            class="info-text-muted text-sm"
          >
            当前无系统提示。
          </p>
          <UiButton
            v-if="systemError"
            size="sm"
            variant="danger"
            class="ink-ui-btn-danger"
            @click="$emit('clearError')"
          >
            清除提示
          </UiButton>
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

type NoNameExecutionView = {
  target: string;
  outcome: string;
  note?: string | null;
};

type NoNamePlanView = {
  target: string;
  decision: string;
  priority: number;
  note?: string | null;
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
  debugNoNameTrace?: string;
  noNameMode?: string;
  debugNoNameProposalTitle?: string;
  debugNoNameProposalStatus?: string;
  debugNoNameTargetSegment?: string;
  debugNoNameIntendedEffect?: string;
  debugNoNameApplyOutcome?: string;
  debugNoNameApplyReason?: string;
  debugNoNameScopes?: string[];
  debugNoNameTransitions?: string[];
  debugNoNamePlans?: NoNamePlanView[];
  debugNoNameExecutions?: NoNameExecutionView[];
  systemError: string | null;
}>();

defineEmits<{
  close: [];
  clearError: [];
  travel: [locationId: string];
  setNoNameMode: [mode: string];
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

const noNameMode = computed(() => props.noNameMode ?? 'observeOnly');
const noNameModeOptions = [
  { value: 'disabled', label: '关闭' },
  { value: 'observeOnly', label: '观察' },
  { value: 'assisted', label: '辅助' },
];
const debugNoNameTransitions = computed(() => props.debugNoNameTransitions ?? []);
const debugNoNamePlans = computed(() => props.debugNoNamePlans ?? []);
const debugNoNameExecutions = computed(() => props.debugNoNameExecutions ?? []);
const debugNoNameScopes = computed(() => props.debugNoNameScopes ?? []);
</script>

<style scoped>
.info-root-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}

.info-overlay {
  background: var(--info-overlay-bg);
}

.ink-info-drawer {
  border-left: 1px solid var(--info-drawer-border);
  background: var(--info-drawer-bg);
  backdrop-filter: blur(10px) saturate(1.03);
  transform: translateX(0);
  animation: info-drawer-slide-in 220ms var(--ease-ink, ease) both;
  will-change: transform, opacity;
}

.ink-info-panel {
  background: var(--info-panel-bg);
  border: 1px solid var(--info-panel-border);
  box-shadow: var(--info-panel-shadow);
  overscroll-behavior: contain;
}

.ink-ui-btn :deep(button),
.ink-ui-btn {
  border: 1px solid var(--ink-border-accent) !important;
  background: var(--info-ui-btn-bg) !important;
  color: var(--ink-text-primary) !important;
  box-shadow: var(--info-ui-btn-shadow);
}

.ink-ui-btn:hover :deep(button),
.ink-ui-btn:hover {
  border-color: var(--ink-title-color) !important;
  background: var(--info-ui-btn-hover-bg) !important;
}

.ink-ui-btn-danger :deep(button),
.ink-ui-btn-danger {
  border: 1px solid var(--ink-accent-main) !important;
  background: var(--info-ui-btn-danger-bg) !important;
  color: var(--info-ui-btn-danger-text) !important;
}

.info-title {
  color: var(--ink-title-color);
  letter-spacing: 0.02em;
}

.info-text-strong {
  color: var(--ink-text-primary);
}

.info-text-body {
  color: var(--info-text-body);
  line-height: 1.62;
}

.info-text-muted {
  color: var(--ink-text-muted);
}

.info-map-card {
  border-color: var(--ink-border-soft);
  background: var(--info-map-card-surface);
  box-shadow: var(--info-map-card-shadow);
}

.info-current-badge {
  background: var(--info-current-badge-bg);
  color: var(--info-current-badge-text);
}

.info-reach-ok {
  color: var(--info-reach-ok-text);
}

.info-reach-blocked {
  color: var(--info-reach-blocked-text);
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
  color: var(--runtime-root-earth);
}

.info-root-metal {
  color: var(--ink-title-color);
}

.info-root-wood {
  color: var(--ink-text-ink);
}

.info-root-water {
  color: var(--runtime-root-water);
}

.info-root-fire {
  color: var(--ink-accent-main);
}

@keyframes info-drawer-slide-in {
  from {
    opacity: 0;
    transform: translateX(14px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

@media (max-width: 768px) {
  .ink-info-drawer {
    inset: auto 0 0 0;
    max-width: none;
    width: 100%;
    padding: 10px;
  }

  .ink-info-panel {
    border-radius: 14px;
    max-height: min(82vh, 760px);
  }
}

@media (prefers-reduced-motion: reduce) {
  .ink-info-drawer {
    animation: none !important;
  }
}
</style>
