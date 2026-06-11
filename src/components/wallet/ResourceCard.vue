<script setup lang="ts">
import { computed } from "vue";
import { formatNumber } from "@/lib/format";

const props = defineProps<{
  label: string;
  available: number;
  limit: number;
  loading?: boolean;
}>();

const usedRatio = computed(() => {
  if (props.limit <= 0) return 0;
  const used = props.limit - props.available;
  return Math.min(100, Math.max(0, (used / props.limit) * 100));
});
</script>

<template>
  <div class="rounded-xl border border-surface-border bg-surface-raised p-5">
    <p class="text-sm text-gray-400">{{ label }}</p>
    <p class="mt-1 text-lg font-semibold text-gray-100">
      <span v-if="loading" class="text-gray-500">読込中...</span>
      <span v-else>{{ formatNumber(available) }} <span class="text-sm font-normal text-gray-400">/ {{ formatNumber(limit) }}</span></span>
    </p>
    <div class="mt-3 h-1.5 w-full overflow-hidden rounded-full bg-surface">
      <div class="h-full rounded-full bg-brand-500" :style="{ width: `${usedRatio}%` }"></div>
    </div>
  </div>
</template>
