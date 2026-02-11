<template>
  <div class="w-80 bg-slate-800 border-r border-slate-700 p-6 overflow-y-auto">
    <h3 class="text-xl font-bold mb-4 text-purple-400">Character Info</h3>
    
    <div v-if="character" class="space-y-4">
      <!-- Basic Info -->
      <div class="pb-4 border-b border-slate-700">
        <p class="text-gray-400 text-sm">Name</p>
        <p class="text-white font-medium text-lg">{{ character.name }}</p>
      </div>

      <!-- Cultivation Realm -->
      <div>
        <p class="text-gray-400 text-sm">Cultivation Realm</p>
        <p class="text-white font-medium">{{ character.stats.cultivation_realm.name }}</p>
        <p class="text-gray-500 text-xs">Level {{ character.stats.cultivation_realm.level }}.{{ character.stats.cultivation_realm.sub_level }}</p>
      </div>

      <!-- Spiritual Root -->
      <div>
        <p class="text-gray-400 text-sm">Spiritual Root</p>
        <div class="flex items-center gap-2">
          <span class="text-white font-medium">{{ character.stats.spiritual_root.element }}</span>
          <span 
            class="px-2 py-0.5 rounded text-xs font-medium"
            :class="getRootGradeClass(character.stats.spiritual_root.grade)"
          >
            {{ character.stats.spiritual_root.grade }}
          </span>
        </div>
        <p class="text-gray-500 text-xs">Affinity: {{ character.stats.spiritual_root.affinity }}%</p>
      </div>

      <!-- Lifespan -->
      <div>
        <p class="text-gray-400 text-sm">Lifespan</p>
        <div class="flex items-center gap-2">
          <p class="text-white font-medium">{{ character.stats.lifespan.current_age }} / {{ character.stats.lifespan.max_age }}</p>
        </div>
        <div class="w-full bg-slate-700 rounded-full h-2 mt-1">
          <div 
            class="h-2 rounded-full transition-all duration-300"
            :class="getLifespanBarClass(character.stats.lifespan)"
            :style="{ width: `${getLifespanPercentage(character.stats.lifespan)}%` }"
          ></div>
        </div>
      </div>

      <!-- Combat Power -->
      <div>
        <p class="text-gray-400 text-sm">Combat Power</p>
        <p class="text-white font-medium">{{ character.stats.combat_power.toLocaleString() }}</p>
      </div>

      <!-- Techniques -->
      <div v-if="character.stats.techniques.length > 0">
        <p class="text-gray-400 text-sm mb-2">Techniques</p>
        <div class="space-y-1">
          <div 
            v-for="(technique, index) in character.stats.techniques" 
            :key="index"
            class="text-sm text-white bg-slate-700 px-2 py-1 rounded"
          >
            {{ technique }}
          </div>
        </div>
      </div>

      <!-- Location -->
      <div>
        <p class="text-gray-400 text-sm">Location</p>
        <p class="text-white font-medium">{{ character.location }}</p>
      </div>

      <!-- Inventory -->
      <div v-if="character.inventory.length > 0">
        <p class="text-gray-400 text-sm mb-2">Inventory</p>
        <div class="space-y-1">
          <div 
            v-for="(item, index) in character.inventory" 
            :key="index"
            class="text-sm text-white bg-slate-700 px-2 py-1 rounded"
          >
            {{ item }}
          </div>
        </div>
      </div>
    </div>

    <div v-else class="text-center text-gray-400">
      <p>No character data available</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Character, Lifespan } from '../types/game';
import { Grade } from '../types/game';

interface Props {
  character: Character | null;
}

defineProps<Props>();

const getRootGradeClass = (grade: Grade): string => {
  switch (grade) {
    case Grade.Heavenly:
      return 'bg-purple-600 text-white';
    case Grade.Earthly:
      return 'bg-blue-600 text-white';
    case Grade.Mortal:
      return 'bg-gray-600 text-white';
    default:
      return 'bg-gray-600 text-white';
  }
};

const getLifespanPercentage = (lifespan: Lifespan): number => {
  return (lifespan.current_age / lifespan.max_age) * 100;
};

const getLifespanBarClass = (lifespan: Lifespan): string => {
  const percentage = getLifespanPercentage(lifespan);
  if (percentage < 30) {
    return 'bg-red-500';
  } else if (percentage < 70) {
    return 'bg-yellow-500';
  } else {
    return 'bg-green-500';
  }
};
</script>
