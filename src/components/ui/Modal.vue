<script setup lang="ts">
import {
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
} from "reka-ui";

defineProps<{
  title: string;
  description?: string;
}>();

const open = defineModel<boolean>("open", { default: false });
</script>

<template>
  <DialogRoot v-model:open="open">
    <DialogPortal>
      <DialogOverlay class="fixed inset-0 z-40 bg-black/60 data-[state=open]:animate-in data-[state=open]:fade-in" />
      <DialogContent
        class="fixed left-1/2 top-1/2 z-50 w-full max-w-md -translate-x-1/2 -translate-y-1/2 rounded-xl border border-surface-border bg-surface-raised p-6 shadow-xl focus:outline-none"
      >
        <DialogTitle class="text-lg font-semibold text-gray-100">{{ title }}</DialogTitle>
        <DialogDescription v-if="description" class="mt-1 text-sm text-gray-400">
          {{ description }}
        </DialogDescription>
        <div class="mt-4">
          <slot />
        </div>
        <div class="mt-6 flex justify-end gap-2">
          <slot name="footer" />
        </div>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>
