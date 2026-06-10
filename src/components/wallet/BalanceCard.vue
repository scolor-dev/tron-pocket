<script setup lang="ts">
defineProps<{
  symbol: string;
  name: string;
  balance: string;
  loading?: boolean;
}>();
</script>

<template>
  <div class="rounded-xl border border-surface-border bg-surface-raised p-5">
    <div class="flex items-center justify-between">
      <div>
        <p class="text-sm text-gray-400">{{ name }}</p>
        <p class="mt-1 text-2xl font-semibold text-gray-100">
          <span v-if="loading" class="text-gray-500">読込中...</span>
          <span v-else>{{ balance }}</span>
          <span class="ml-1 text-base font-normal text-gray-400">{{ symbol }}</span>
        </p>
      </div>
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-brand-500/15 text-sm font-bold text-brand-400">
        {{ symbol.slice(0, 1) }}
      </div>
    </div>
  </div>
</template>
