<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import { useWalletStore } from "@/stores/wallet";
import { NETWORK_INFO } from "@/lib/constants";
import { formatError } from "@/lib/format";
import type { Network } from "@/types";
import AddressDisplay from "@/components/wallet/AddressDisplay.vue";
import Button from "@/components/ui/Button.vue";

const router = useRouter();
const wallet = useWalletStore();

const error = ref("");
const switching = ref(false);

async function changeNetwork(network: Network) {
  if (network === wallet.network || switching.value) return;
  error.value = "";
  switching.value = true;
  try {
    await wallet.setNetwork(network);
  } catch (e) {
    error.value = formatError(e);
  } finally {
    switching.value = false;
  }
}

async function handleLock() {
  await wallet.lock();
  await router.push("/unlock");
}
</script>

<template>
  <div class="mx-auto max-w-md space-y-6">
    <h1 class="text-2xl font-semibold text-gray-100">設定</h1>

    <div class="space-y-3 rounded-xl border border-surface-border bg-surface-raised p-4">
      <p class="text-sm text-gray-400">ウォレットアドレス</p>
      <AddressDisplay v-if="wallet.address" :address="wallet.address" full />
    </div>

    <div class="space-y-3 rounded-xl border border-surface-border bg-surface-raised p-4">
      <p class="text-sm text-gray-400">ネットワーク</p>
      <div class="flex gap-2">
        <Button
          v-for="net in (['mainnet', 'nile'] as Network[])"
          :key="net"
          :variant="wallet.network === net ? 'primary' : 'secondary'"
          :disabled="switching"
          @click="changeNetwork(net)"
        >
          {{ NETWORK_INFO[net].label }}
        </Button>
      </div>
      <p v-if="error" class="text-sm text-red-400">{{ error }}</p>
    </div>

    <div class="rounded-xl border border-surface-border bg-surface-raised p-4">
      <Button variant="danger" class="w-full" @click="handleLock">ウォレットをロック</Button>
    </div>
  </div>
</template>
