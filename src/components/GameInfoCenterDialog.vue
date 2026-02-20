<template>
  <InfoTabsDialog
    :is-open="isOpen"
    :player-name="gameStore.playerCharacter?.name || '无名弟子'"
    :player-realm="playerRealmLabel"
    :player-combat-power="playerCombatPowerLabel"
    :player-location="playerLocationLabel"
    :chapter-progress="chapterProgressLabel"
    :chapter-interaction="chapterInteractionLabel"
    :segment-count="gameStore.plotState?.segment_count ?? 0"
    :is-waiting-for-input="gameStore.isWaitingForInput"
    :world-locations="worldLocationList"
    :reachable-location-ids="gameStore.reachableLocationIds"
    :map-overview="gameStore.mapOverview"
    :recent-combat-explanations="recentCombatReview"
    :current-location-id="currentLocationId"
    :current-location-label="currentLocationLabel"
    :is-traveling="travelPending"
    :is-game-running="gameStore.isGameInitialized"
    :event-count="gameStore.gameState?.event_history?.length ?? 0"
    :is-dev-mode="isDevMode"
    :debug-chapter="`${gameStore.plotState?.current_chapter?.index ?? 0} / ${gameStore.plotState?.current_chapter?.title ?? 'n/a'}`"
    :debug-option-source="optionSourceLabel || 'n/a'"
    :debug-option-hint="optionSourceHint || ''"
    :debug-risk-score="consistencyRiskScore"
    :debug-diagnostics="gameStore.plotState?.last_generation_diagnostics || ''"
    :system-error="gameStore.error"
    @close="$emit('close')"
    @clear-error="$emit('clear-error')"
    @travel="$emit('travel', $event)"
  />
</template>

<script setup lang="ts">
import { computed } from 'vue';
import InfoTabsDialog from './InfoTabsDialog.vue';
import type { MapLocationOverview } from '../types/game';
import { buildLocationLabelMap, formatLocationLabel } from '../shared/locationLabel';

type WorldLocation = {
  id: string;
  name: string;
  spiritual_energy: number;
};

type GameStoreView = {
  playerCharacter?: {
    name?: string;
    location?: string;
  } | null;
  currentScene?: {
    location?: string;
  } | null;
  plotState?: {
    segment_count?: number;
    current_chapter?: {
      index?: number;
      title?: string;
    } | null;
    last_generation_diagnostics?: string | null;
  } | null;
  isWaitingForInput: boolean;
  reachableLocationIds: string[];
  mapOverview: MapLocationOverview[];
  isGameInitialized: boolean;
  gameState?: {
    event_history?: unknown[];
  } | null;
  error: string | null;
};

const props = defineProps<{
  isOpen: boolean;
  gameStore: GameStoreView;
  playerRealmLabel: string;
  playerCombatPowerLabel: string;
  chapterProgressLabel: string;
  chapterInteractionLabel: string;
  worldLocationList: WorldLocation[];
  recentCombatReview: string[];
  travelPending: boolean;
  isDevMode: boolean;
  optionSourceLabel: string;
  optionSourceHint?: string;
  consistencyRiskScore: number | null;
}>();

defineEmits<{
  (event: 'close'): void;
  (event: 'clear-error'): void;
  (event: 'travel', locationId: string): void;
}>();

const locationLabelMap = computed(() =>
  buildLocationLabelMap(props.worldLocationList.map((loc) => ({ id: loc.id, name: loc.name }))),
);

const currentLocationId = computed(() => props.gameStore.playerCharacter?.location || '');
const currentLocationLabel = computed(() =>
  formatLocationLabel(currentLocationId.value, locationLabelMap.value),
);
const playerLocationLabel = computed(() =>
  formatLocationLabel(
    props.gameStore.playerCharacter?.location || props.gameStore.currentScene?.location,
    locationLabelMap.value,
  ),
);
</script>
