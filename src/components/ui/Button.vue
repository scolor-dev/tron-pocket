<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    variant?: "primary" | "secondary" | "ghost" | "danger";
    type?: "button" | "submit";
    disabled?: boolean;
  }>(),
  {
    variant: "primary",
    type: "button",
    disabled: false,
  },
);

const variantClass = computed(() => {
  switch (props.variant) {
    case "primary":
      return "bg-brand-500 text-white hover:bg-brand-600 focus-visible:outline-brand-400";
    case "secondary":
      return "bg-surface-raised text-gray-100 border border-surface-border hover:bg-surface-border";
    case "ghost":
      return "bg-transparent text-gray-300 hover:bg-surface-raised";
    case "danger":
      return "bg-red-600 text-white hover:bg-red-700 focus-visible:outline-red-500";
  }
  return "";
});
</script>

<template>
  <button
    :type="type"
    :disabled="disabled"
    class="inline-flex items-center justify-center gap-2 rounded-lg px-4 py-2 text-sm font-semibold transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
    :class="variantClass"
  >
    <slot />
  </button>
</template>
