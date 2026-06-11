<script setup lang="ts">
import { onMounted } from "vue";
import { RouterLink } from "vue-router";
import { useWalletStore } from "@/stores/wallet";
import { NETWORK_INFO } from "@/lib/constants";
import AddressDisplay from "@/components/wallet/AddressDisplay.vue";
import BalanceCard from "@/components/wallet/BalanceCard.vue";
import ResourceCard from "@/components/wallet/ResourceCard.vue";
import Button from "@/components/ui/Button.vue";

const wallet = useWalletStore();

function refresh() {
  wallet.refreshBalances();
  wallet.refreshResources();
}

onMounted(refresh);
</script>

<template>
  <div class="mx-auto max-w-2xl space-y-6">
    <div class="flex items-center justify-between">
      <h1 class="text-2xl font-semibold text-gray-100">ホーム</h1>
      <Button variant="secondary" :disabled="wallet.balancesLoading || wallet.resourcesLoading" @click="refresh">
        {{ wallet.balancesLoading || wallet.resourcesLoading ? "更新中..." : "残高を更新" }}
      </Button>
    </div>

    <div class="flex items-center justify-between rounded-xl border border-surface-border bg-surface-raised p-4">
      <div>
        <p class="text-sm text-gray-400">{{ NETWORK_INFO[wallet.network].label }}</p>
        <AddressDisplay v-if="wallet.address" :address="wallet.address" class="mt-2" />
      </div>
    </div>

    <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
      <BalanceCard symbol="TRX" name="TRON" :balance="wallet.balances.trx" :loading="wallet.balancesLoading" />
      <BalanceCard
        v-if="wallet.network === 'mainnet'"
        symbol="USDT"
        name="Tether USD"
        :balance="wallet.balances.usdt"
        :loading="wallet.balancesLoading"
      />
    </div>

    <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
      <ResourceCard
        label="帯域幅 (Bandwidth)"
        :available="wallet.resources.bandwidth_available"
        :limit="wallet.resources.bandwidth_limit"
        :loading="wallet.resourcesLoading"
      />
      <ResourceCard
        label="エネルギー (Energy)"
        :available="wallet.resources.energy_available"
        :limit="wallet.resources.energy_limit"
        :loading="wallet.resourcesLoading"
      />
    </div>

    <div class="grid grid-cols-3 gap-4">
      <RouterLink
        to="/send"
        class="rounded-xl border border-surface-border bg-surface-raised p-4 text-center text-sm font-medium text-gray-200 transition-colors hover:border-brand-400"
      >
        送金
      </RouterLink>
      <RouterLink
        to="/receive"
        class="rounded-xl border border-surface-border bg-surface-raised p-4 text-center text-sm font-medium text-gray-200 transition-colors hover:border-brand-400"
      >
        受取
      </RouterLink>
      <RouterLink
        to="/history"
        class="rounded-xl border border-surface-border bg-surface-raised p-4 text-center text-sm font-medium text-gray-200 transition-colors hover:border-brand-400"
      >
        履歴
      </RouterLink>
    </div>
  </div>
</template>
