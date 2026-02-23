<template>
  <div v-if="visible" class="space-y-2">
    <button
      v-for="(option, index) in options"
      :key="index"
      :disabled="disabled"
      class="option-btn w-full rounded-[8px] border px-4 py-3 text-left transition-all duration-200"
      :class="[
        disabled
          ? 'cursor-not-allowed opacity-50'
          : 'cursor-pointer border-[#d9c0b0] bg-[#ffffff] hover:border-[#b78c4a] hover:bg-[#faf7f2]',
      ]"
      @click="$emit('select', option)"
    >
        <div class="flex items-start justify-between gap-2">
          <div class="min-w-0">
            <div class="mb-2 flex items-center gap-2">
              <span class="text-sm text-[#5e5a54]">选项{{ toCnNumber(index + 1) }}</span>
              <span
                v-if="optionTag(option) === 'risk'"
                class="inline-flex items-center rounded-full border border-[#b23e3e]/40 bg-[#b23e3e]/10 px-2 py-0.5 text-[11px] text-[#b23e3e]"
              >
                风险
              </span>
              <span
                v-else-if="optionTag(option) === 'probe'"
                class="inline-flex items-center rounded-full border border-[#3b7a6b]/40 bg-[#3b7a6b]/10 px-2 py-0.5 text-[11px] text-[#3b7a6b]"
              >
                探查
              </span>
            </div>
          <p class="text-[15px] leading-[1.65] text-[#2d2a24]">
            {{ `选项${toCnNumber(index + 1)} · ${normalizeOptionDescription(option.description)}` }}
          </p>
          <p
            v-if="option.requirements && option.requirements.length > 0"
            class="mt-1 text-sm text-[#5e5a54]"
          >
            条件：{{ option.requirements.join('，') }}
          </p>
        </div>
        <span class="mt-0.5 text-[#b78c4a]" aria-hidden="true">
          <svg
            v-if="optionIcon(option, index) === 'eye'"
            viewBox="0 0 24 24"
            class="h-4 w-4"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
          >
            <path stroke-linecap="round" stroke-linejoin="round" d="M2 12s3.5-6 10-6 10 6 10 6-3.5 6-10 6S2 12 2 12Z" />
            <circle cx="12" cy="12" r="2.5" />
          </svg>
          <svg
            v-else-if="optionIcon(option, index) === 'leaf'"
            viewBox="0 0 24 24"
            class="h-4 w-4"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
          >
            <path stroke-linecap="round" stroke-linejoin="round" d="M12 3c5 0 8 4 8 8 0 5-4 10-8 10S4 16 4 11c0-4 3-8 8-8Z" />
            <path stroke-linecap="round" stroke-linejoin="round" d="M9 12c2 0 4-2 6-6" />
          </svg>
          <svg
            v-else
            viewBox="0 0 24 24"
            class="h-4 w-4"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
          >
            <path stroke-linecap="round" stroke-linejoin="round" d="M5 12h14M13 6l6 6-6 6" />
          </svg>
        </span>
      </div>
    </button>
  </div>
</template>

<script setup lang="ts">
import type { PlayerOption } from '../types/game';

defineProps<{
  visible: boolean;
  options: PlayerOption[];
  disabled: boolean;
}>();

defineEmits<{
  select: [option: PlayerOption];
}>();

type OptionTag = 'risk' | 'probe' | null;
type OptionIcon = 'arrow' | 'eye' | 'leaf';

const backendFieldKeys = [
  'tag',
  'tags',
  'category',
  'intent',
  'risk',
  'risk_tier',
  'option_type',
  'type',
];

const riskWords = [
  'risk',
  'danger',
  'hazard',
  'high',
  'critical',
  'combat',
  'battle',
  'breakthrough',
  '冲突',
  '危险',
  '风险',
  '凶',
  '硬闯',
  '强攻',
  '搏',
  '突破',
  '激进',
];

const probeWords = [
  'probe',
  'investigate',
  'scout',
  'observe',
  'explore',
  'recon',
  'check',
  '探查',
  '探路',
  '观察',
  '侦查',
  '打探',
  '试探',
  '勘察',
  '查探',
];

const normalize = (text: unknown): string => String(text ?? '').toLowerCase();

const extractBackendTone = (option: PlayerOption): OptionTag => {
  const raw = option as unknown as Record<string, unknown>;
  const bucket: string[] = [];
  for (const key of backendFieldKeys) {
    const value = raw[key];
    if (typeof value === 'string') {
      bucket.push(value);
      continue;
    }
    if (Array.isArray(value)) {
      for (const item of value) {
        if (typeof item === 'string') bucket.push(item);
      }
    }
  }
  const merged = normalize(bucket.join(' '));
  if (riskWords.some((word) => merged.includes(word))) return 'risk';
  if (probeWords.some((word) => merged.includes(word))) return 'probe';
  return null;
};

const optionTag = (option: PlayerOption): OptionTag => {
  const backendTag = extractBackendTone(option);
  if (backendTag) return backendTag;

  const corpus = normalize(`${option.description} ${option.requirements.join(' ')} ${JSON.stringify(option.action ?? {})}`);
  if (riskWords.some((word) => corpus.includes(word))) return 'risk';
  if (probeWords.some((word) => corpus.includes(word))) return 'probe';
  return null;
};

const optionIcon = (option: PlayerOption, index: number): OptionIcon => {
  const tag = optionTag(option);
  if (tag === 'probe') return 'eye';
  if (tag === 'risk') return 'arrow';
  return index % 2 === 0 ? 'leaf' : 'arrow';
};

const toCnNumber = (num: number): string => {
  const labels = ['一', '二', '三', '四', '五'];
  return labels[num - 1] ?? String(num);
};

const normalizeOptionDescription = (raw: string): string => {
  const text = String(raw ?? '').trim();
  if (!text) return '';
  const stripped = text
    .replace(/^选项\s*[一二三四五六七八九十百千万\d]+\s*[：:、.\-]\s*/u, '')
    .replace(/^选项\s*[一二三四五六七八九十百千万\d]+\s*/u, '')
    .trim();
  return stripped || text;
};
</script>

<style scoped>
.option-btn {
  box-shadow: 0 2px 10px rgba(45, 42, 36, 0.04);
}

.option-btn:hover {
  box-shadow: 0 4px 14px rgba(45, 42, 36, 0.08);
  transform: translateY(-1px);
}
</style>
