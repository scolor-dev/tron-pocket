<script setup lang="ts">
import { RouterLink } from "vue-router";
import { useWalletStore } from "@/stores/wallet";
import { NETWORK_INFO } from "@/lib/constants";
import { shortenAddress } from "@/lib/format";

const wallet = useWalletStore();

const navItems = [
  { to: "/", label: "ホーム", icon: "M3 12l9-9 9 9M5 10v10h14V10" },
  { to: "/send", label: "送金", icon: "M5 12h14M13 6l6 6-6 6" },
  { to: "/receive", label: "受取", icon: "M19 12H5M11 18l-6-6 6-6" },
  { to: "/history", label: "履歴", icon: "M12 8v4l3 3M21 12a9 9 0 1 1-9-9 9 9 0 0 1 9 9z" },
  { to: "/settings", label: "設定", icon: "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM4 12h2m12 0h2M12 4v2m0 12v2m5.66-12.66-1.42 1.42M7.76 16.24l-1.42 1.42m0-11.32 1.42 1.42m9.48 9.48 1.42 1.42" },
];
</script>

<template>
  <div class="flex h-full">
    <aside class="flex w-56 shrink-0 flex-col border-r border-surface-border bg-surface-raised p-4">
      <div class="mb-6 flex items-center gap-2 px-2">
        <div class="h-8 w-8 rounded-full bg-brand-500"></div>
        <span class="text-lg font-semibold">Tron Pocket</span>
      </div>

      <nav class="flex flex-1 flex-col gap-1">
        <RouterLink
          v-for="item in navItems"
          :key="item.to"
          :to="item.to"
          class="flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium text-gray-300 transition-colors hover:bg-surface hover:text-white"
          active-class="bg-brand-500/15 text-brand-400 hover:bg-brand-500/15 hover:text-brand-400"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="h-5 w-5">
            <path :d="item.icon" />
          </svg>
          {{ item.label }}
        </RouterLink>
      </nav>

      <div class="mt-auto rounded-lg border border-surface-border p-3 text-xs text-gray-400">
        <div class="mb-1 font-medium text-gray-300">{{ NETWORK_INFO[wallet.network].label }}</div>
        <div class="font-mono">{{ wallet.address ? shortenAddress(wallet.address) : "-" }}</div>
      </div>
    </aside>

    <main class="flex-1 overflow-y-auto p-8">
      <slot />
    </main>
  </div>
</template>
