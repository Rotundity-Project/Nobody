<template>
  <div class="w-full lg:w-80 bg-slate-900/80 border-b lg:border-b-0 lg:border-r border-slate-700 p-6 overflow-y-auto backdrop-blur">
    <h3 class="text-xl font-display mb-4 text-amber-200">角色信息</h3>

    <div v-if="character" class="space-y-4">
      <div class="pb-4 border-b border-slate-700">
        <p class="text-slate-400 text-sm">姓名</p>
        <p class="text-white font-medium text-lg">{{ character.name }}</p>
      </div>

      <div>
        <p class="text-slate-400 text-sm">修为境界</p>
        <p class="text-white font-medium">{{ character.stats.cultivation_realm.name }}</p>
        <p class="text-slate-500 text-xs">
          等级 {{ character.stats.cultivation_realm.level }}.{{ character.stats.cultivation_realm.sub_level }}
        </p>
      </div>

      <div>
        <p class="text-slate-400 text-sm">灵根</p>
        <div class="flex items-center gap-2">
          <span class="text-white font-medium">{{ character.stats.spiritual_root.element }}</span>
          <span
            class="px-2 py-0.5 rounded text-xs font-medium"
            :class="getRootGradeClass(character.stats.spiritual_root.grade)"
          >
            {{ character.stats.spiritual_root.grade }}
          </span>
        </div>
        <p class="text-slate-500 text-xs">亲和度 {{ character.stats.spiritual_root.affinity }}%</p>
      </div>

      <div>
        <p class="text-slate-400 text-sm">寿元</p>
        <div class="flex items-center gap-2">
          <p class="text-white font-medium">
            {{ character.stats.lifespan.current_age }} / {{ character.stats.lifespan.max_age }}
          </p>
        </div>
        <div class="w-full bg-slate-700 rounded-full h-2 mt-1">
          <div
            class="h-2 rounded-full transition-all duration-300"
            :class="getLifespanBarClass(character.stats.lifespan)"
            :style="{ width: `${getLifespanPercentage(character.stats.lifespan)}%` }"
          ></div>
        </div>
      </div>

      <div>
        <p class="text-slate-400 text-sm">战斗力</p>
        <p class="text-white font-medium">{{ character.stats.combat_power.toLocaleString() }}</p>
      </div>

      <div v-if="character.stats.techniques.length > 0">
        <p class="text-slate-400 text-sm mb-2">功法</p>
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

      <div>
        <p class="text-slate-400 text-sm">位置</p>
        <p class="text-white font-medium">{{ character.location }}</p>
      </div>

      <div v-if="character.inventory.length > 0">
        <p class="text-slate-400 text-sm mb-2">物品</p>
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

    <div v-else class="text-center text-slate-400">
      <p>暂无角色数据</p>
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
    case Grade.Double:
      return 'bg-blue-600 text-white';
    case Grade.Triple:
      return 'bg-green-600 text-white';
    case Grade.Pseudo:
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
