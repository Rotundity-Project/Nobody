<template>
  <div class="panel-surface rounded-2xl p-5 max-h-[70vh] overflow-y-auto">
    <h3 class="text-xl font-display mb-4 text-amber-200">角色信息</h3>

    <div v-if="character" class="space-y-4">
      <div class="pb-4 border-b border-slate-700">
        <p class="text-slate-400 text-sm">姓名</p>
        <p class="text-white font-medium text-lg">{{ character.name }}</p>
      </div>

      <div>
        <p class="text-slate-400 text-sm">修为境界</p>
        <p class="text-white font-medium">{{ realmName }}</p>
        <p class="text-slate-500 text-xs">
          等级 {{ character.stats.cultivation_realm.level }}.{{ character.stats.cultivation_realm.sub_level }}
        </p>
      </div>

      <div>
        <p class="text-slate-400 text-sm">灵根</p>
        <div class="flex items-center gap-2">
          <span class="text-white font-medium">{{ elementLabel }}</span>
          <span
            class="px-2 py-0.5 rounded text-xs font-medium"
            :class="getRootGradeClass(character.stats.spiritual_root.grade)"
          >
            {{ gradeLabel }}
          </span>
        </div>
        <p class="text-slate-500 text-xs">亲和度 {{ affinityLabel }}</p>
        <p class="text-slate-500 text-xs">天赋提示：{{ gradeHint }}</p>
      </div>

      <div>
        <p class="text-slate-400 text-sm">寿元</p>
        <p class="text-white font-medium">
          {{ character.stats.lifespan.current_age }} / {{ character.stats.lifespan.max_age }}
        </p>
        <div class="w-full bg-slate-700 rounded-full h-2 mt-1">
          <div
            class="h-2 rounded-full transition-all duration-300"
            :class="getLifespanBarClass(character.stats.lifespan)"
            :style="{ width: `${getLifespanPercentage(character.stats.lifespan)}%` }"
          />
        </div>
      </div>

      <div>
        <p class="text-slate-400 text-sm">战斗力</p>
        <p class="text-white font-medium">{{ character.stats.combat_power.toLocaleString() }}</p>
      </div>

      <div>
        <p class="text-slate-400 text-sm">位置</p>
        <p class="text-white font-medium">{{ locationLabel }}</p>
      </div>
    </div>

    <div v-else class="text-center text-slate-400">
      <p>暂无角色数据</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { Character, Lifespan } from '../types/game';
import { Element, Grade } from '../types/game';

interface Props {
  character: Character | null;
}

const props = defineProps<Props>();

const elementLabel = computed(() => {
  if (!props.character) return '';
  const mapping: Record<Element, string> = {
    [Element.Fire]: '火灵根',
    [Element.Water]: '水灵根',
    [Element.Wood]: '木灵根',
    [Element.Metal]: '金灵根',
    [Element.Earth]: '土灵根',
  };
  return mapping[props.character.stats.spiritual_root.element] ?? '未知灵根';
});

const gradeLabel = computed(() => {
  if (!props.character) return '';
  const mapping: Record<Grade, string> = {
    [Grade.Heavenly]: '单灵根',
    [Grade.Double]: '双灵根',
    [Grade.Triple]: '三灵根',
    [Grade.Pseudo]: '杂灵根',
  };
  return mapping[props.character.stats.spiritual_root.grade] ?? '未知';
});

const gradeHint = computed(() => {
  if (!props.character) return '';
  switch (props.character.stats.spiritual_root.grade) {
    case Grade.Heavenly:
      return '极其稀有，修行速度快，最受宗门重视';
    case Grade.Double:
      return '较为出众，修行效率高，资源倾斜明显';
    case Grade.Triple:
      return '中等资质，稳扎稳打可成材';
    case Grade.Pseudo:
      return '资质普通，需要更多机缘与努力';
    default:
      return '暂无评价';
  }
});

const affinityLabel = computed(() => {
  if (!props.character) return '';
  const pct = Math.round(props.character.stats.spiritual_root.affinity * 100);
  return `${pct}%`;
});

const realmName = computed(() => {
  if (!props.character) return '';
  const raw = props.character.stats.cultivation_realm.name;
  const mapping: Record<string, string> = {
    'Qi Condensation': '练气',
    'Foundation Establishment': '筑基',
    'Golden Core': '金丹',
    'Nascent Soul': '元婴',
  };
  return mapping[raw] ?? raw;
});

const locationLabel = computed(() => {
  if (!props.character) return '';
  const raw = props.character.location;
  const mapping: Record<string, string> = {
    sect_valley: '宗门外谷',
    stone_forest: '乱石林',
    sect: '宗门驻地',
    city: '凡人城镇',
  };
  return mapping[raw] ?? raw.replaceAll('_', ' ');
});

const getRootGradeClass = (grade: Grade): string => {
  switch (grade) {
    case Grade.Heavenly:
      return 'bg-amber-600 text-white';
    case Grade.Double:
      return 'bg-emerald-600 text-white';
    case Grade.Triple:
      return 'bg-sky-600 text-white';
    case Grade.Pseudo:
      return 'bg-slate-600 text-white';
    default:
      return 'bg-slate-600 text-white';
  }
};

const getLifespanPercentage = (lifespan: Lifespan): number => {
  return Math.min(100, (lifespan.current_age / lifespan.max_age) * 100);
};

const getLifespanBarClass = (lifespan: Lifespan): string => {
  const percentage = getLifespanPercentage(lifespan);
  if (percentage < 30) return 'bg-emerald-500';
  if (percentage < 70) return 'bg-amber-500';
  return 'bg-rose-500';
};
</script>
