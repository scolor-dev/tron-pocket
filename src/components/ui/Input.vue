<script setup lang="ts">
withDefaults(
  defineProps<{
    label?: string;
    type?: string;
    placeholder?: string;
    error?: string;
    hint?: string;
  }>(),
  {
    type: "text",
  },
);

const model = defineModel<string>({ default: "" });
</script>

<template>
  <label class="block">
    <span v-if="label" class="mb-1.5 block text-sm font-medium text-gray-300">{{ label }}</span>
    <input
      v-model="model"
      :type="type"
      :placeholder="placeholder"
      class="w-full rounded-lg border border-surface-border bg-surface px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-brand-400 focus:outline-none focus:ring-1 focus:ring-brand-400"
      :class="{ 'border-red-500 focus:border-red-500 focus:ring-red-500': error }"
    />
    <p v-if="error" class="mt-1 text-xs text-red-400">{{ error }}</p>
    <p v-else-if="hint" class="mt-1 text-xs text-gray-500">{{ hint }}</p>
  </label>
</template>
