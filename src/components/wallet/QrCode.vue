<script setup lang="ts">
import { ref, watchEffect } from "vue";
import QRCode from "qrcode";

const props = withDefaults(
  defineProps<{
    value: string;
    size?: number;
  }>(),
  {
    size: 220,
  },
);

const dataUrl = ref("");

watchEffect(async () => {
  dataUrl.value = await QRCode.toDataURL(props.value, {
    width: props.size,
    margin: 1,
    color: { dark: "#0f1115", light: "#ffffff" },
  });
});
</script>

<template>
  <img
    v-if="dataUrl"
    :src="dataUrl"
    :width="size"
    :height="size"
    :alt="`QRコード: ${value}`"
    class="rounded-lg"
  />
</template>
