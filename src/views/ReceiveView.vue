<script setup lang="ts">
import { useWalletStore } from "@/stores/wallet";
import { NETWORK_INFO } from "@/lib/constants";
import AddressDisplay from "@/components/wallet/AddressDisplay.vue";
import QrCode from "@/components/wallet/QrCode.vue";

const wallet = useWalletStore();
</script>

<template>
  <div class="mx-auto max-w-md space-y-6">
    <h1 class="text-2xl font-semibold text-gray-100">受取</h1>

    <div class="flex flex-col items-center gap-4 rounded-xl border border-surface-border bg-surface-raised p-6">
      <QrCode v-if="wallet.address" :value="wallet.address" />
      <AddressDisplay v-if="wallet.address" :address="wallet.address" full />
      <p class="text-center text-xs text-gray-500">
        このアドレスは {{ NETWORK_INFO[wallet.network].label }} 上の TRX および TRC20トークンの受取に使用できます。
      </p>
    </div>

    <div class="rounded-lg border border-brand-500/40 bg-brand-500/10 p-3 text-sm text-brand-100">
      異なるネットワークやチェーン宛に送金されたトークンは失われる可能性があります。送金元のネットワークを必ず確認してください。
    </div>
  </div>
</template>
