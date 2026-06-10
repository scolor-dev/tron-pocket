<script setup lang="ts">
import { ref } from "vue";
import { shortenAddress } from "@/lib/format";

const props = withDefaults(
  defineProps<{
    address: string;
    full?: boolean;
  }>(),
  {
    full: false,
  },
);

const copied = ref(false);

async function copy() {
  await navigator.clipboard.writeText(props.address);
  copied.value = true;
  setTimeout(() => (copied.value = false), 1500);
}
</script>

<template>
  <button
    type="button"
    class="inline-flex items-center gap-2 rounded-lg border border-surface-border bg-surface px-3 py-1.5 font-mono text-sm text-gray-200 transition-colors hover:border-brand-400"
    @click="copy"
  >
    <span>{{ full ? address : shortenAddress(address) }}</span>
    <span class="text-xs text-gray-500">{{ copied ? "コピーしました" : "コピー" }}</span>
  </button>
</template>
