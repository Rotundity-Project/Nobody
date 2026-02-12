<template>
  <div ref="containerRef" class="relative w-full">
    <div v-if="!useVirtualization" class="space-y-4">
      <p
        v-for="(text, idx) in paragraphs"
        :key="idx"
        class="font-story text-slate-200 leading-relaxed whitespace-pre-wrap"
      >
        {{ text }}
      </p>
    </div>
    <div v-else :style="{ height: `${totalHeight}px` }">
      <div :style="{ transform: `translateY(${topPadding}px)` }">
        <p
          v-for="(text, idx) in visibleItems"
          :key="visibleStart + idx"
          class="font-story text-slate-200 leading-relaxed whitespace-pre-wrap"
        >
          {{ text }}
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';

interface Props {
  paragraphs: string[];
  scrollElement?: HTMLElement | null;
}

const props = defineProps<Props>();
const containerRef = ref<HTMLElement | null>(null);
const viewportHeight = ref(0);
const scrollTop = ref(0);
const containerWidth = ref(0);

const lineHeightPx = 28;
const paragraphGap = 16;
const overscan = 6;
const MIN_VIRTUAL_PARAGRAPHS = 120;
const MIN_VIRTUAL_WIDTH = 900;

let resizeObserver: ResizeObserver | null = null;
let rafId = 0;

const updateViewport = () => {
  const el = props.scrollElement ?? containerRef.value;
  if (!el) {
    return;
  }
  viewportHeight.value = el.clientHeight || 0;
  containerWidth.value = el.clientWidth || 0;
};

const onScroll = () => {
  if (rafId) {
    cancelAnimationFrame(rafId);
  }
  rafId = requestAnimationFrame(() => {
    const el = props.scrollElement ?? containerRef.value;
    if (!el) {
      return;
    }
    scrollTop.value = el.scrollTop;
  });
};

const estimateHeight = (text: string) => {
  const width = containerWidth.value || 640;
  const charsPerLine = Math.max(12, Math.floor(width / 14));
  const lines = Math.max(1, Math.ceil(text.length / charsPerLine));
  return lines * lineHeightPx + paragraphGap;
};

const heights = computed(() => props.paragraphs.map(estimateHeight));
const useVirtualization = computed(
  () =>
    props.paragraphs.length >= MIN_VIRTUAL_PARAGRAPHS &&
    containerWidth.value >= MIN_VIRTUAL_WIDTH,
);
const offsets = computed(() => {
  if (!useVirtualization.value) {
    return [];
  }
  const result = new Array(props.paragraphs.length).fill(0);
  let acc = 0;
  for (let i = 0; i < props.paragraphs.length; i += 1) {
    result[i] = acc;
    acc += heights.value[i] ?? 0;
  }
  return result;
});
const totalHeight = computed(() => {
  if (!useVirtualization.value) {
    return 0;
  }
  const last = offsets.value.length - 1;
  if (last < 0) {
    return 0;
  }
  return (offsets.value[last] ?? 0) + (heights.value[last] ?? 0);
});

const findStartIndex = (scroll: number) => {
  const offs = offsets.value;
  let lo = 0;
  let hi = offs.length - 1;
  let ans = 0;
  while (lo <= hi) {
    const mid = Math.floor((lo + hi) / 2);
    const val = offs[mid] ?? 0;
    if (val <= scroll) {
      ans = mid;
      lo = mid + 1;
    } else {
      hi = mid - 1;
    }
  }
  return ans;
};

const visibleRange = computed(() => {
  if (!useVirtualization.value) {
    return { start: 0, end: props.paragraphs.length };
  }
  const start = Math.max(0, findStartIndex(scrollTop.value) - overscan);
  const maxScroll = scrollTop.value + viewportHeight.value;
  let end = start;
  while (
    end < offsets.value.length &&
    (offsets.value[end] ?? 0) < maxScroll + overscan * lineHeightPx
  ) {
    end += 1;
  }
  return {
    start,
    end: Math.min(offsets.value.length, end + overscan),
  };
});

const visibleStart = computed(() => visibleRange.value.start);
const visibleItems = computed(() =>
  props.paragraphs.slice(visibleRange.value.start, visibleRange.value.end),
);
const topPadding = computed(() => offsets.value[visibleRange.value.start] ?? 0);

onMounted(() => {
  const el = props.scrollElement ?? containerRef.value;
  if (!el) {
    return;
  }
  updateViewport();
  el.addEventListener('scroll', onScroll, { passive: true });
  resizeObserver = new ResizeObserver(updateViewport);
  resizeObserver.observe(el);
});

onBeforeUnmount(() => {
  const el = props.scrollElement ?? containerRef.value;
  if (el) {
    el.removeEventListener('scroll', onScroll);
  }
  if (resizeObserver) {
    resizeObserver.disconnect();
  }
  if (rafId) {
    cancelAnimationFrame(rafId);
  }
});

watch(
  () => props.paragraphs.length,
  () => {
    const el = props.scrollElement ?? containerRef.value;
    if (el) {
      scrollTop.value = el.scrollTop;
    }
  },
);
</script>
