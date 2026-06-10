<script setup lang="ts">
import { computed } from "vue";
import { useWalletStore } from "@/stores/wallet";
import AppShell from "@/components/layout/AppShell.vue";

const wallet = useWalletStore();
const showShell = computed(() => wallet.statusLoaded && wallet.unlocked);
</script>

<template>
  <AppShell v-if="showShell">
    <RouterView />
  </AppShell>
  <div v-else class="flex h-full items-center justify-center p-6">
    <RouterView />
  </div>
</template>
