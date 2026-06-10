<script setup lang="ts">
import { computed } from "vue";
import type { TronTransaction } from "@/types";
import { NETWORK_INFO } from "@/lib/constants";
import { shortenAddress, shortenTxId } from "@/lib/format";
import { useWalletStore } from "@/stores/wallet";

const props = defineProps<{
  tx: TronTransaction;
}>();

const wallet = useWalletStore();

const contract = computed(() => props.tx.raw_data.contract[0]);

const typeLabel = computed(() => {
  switch (contract.value?.type) {
    case "TransferContract":
      return "TRX送金";
    case "TriggerSmartContract":
      return "トークン送金";
    default:
      return contract.value?.type ?? "不明";
  }
});

const amountLabel = computed(() => {
  if (contract.value?.type === "TransferContract") {
    const sun = contract.value.parameter.value.amount ?? 0;
    return `${(sun / 1_000_000).toLocaleString("ja-JP", { maximumFractionDigits: 6 })} TRX`;
  }
  return "-";
});

const status = computed(() => props.tx.ret?.[0]?.contractRet ?? "UNKNOWN");

const statusClass = computed(() => {
  switch (status.value) {
    case "SUCCESS":
      return "bg-green-500/15 text-green-400";
    case "UNKNOWN":
      return "bg-gray-500/15 text-gray-400";
    default:
      return "bg-red-500/15 text-red-400";
  }
});

const timestamp = computed(() => {
  const ts = props.tx.raw_data.timestamp;
  if (!ts) return "-";
  return new Date(ts).toLocaleString("ja-JP");
});

const explorerUrl = computed(() => `${NETWORK_INFO[wallet.network].explorerTxUrl}${props.tx.txID}`);
</script>

<template>
  <a
    :href="explorerUrl"
    target="_blank"
    rel="noreferrer"
    class="flex items-center justify-between rounded-lg border border-surface-border bg-surface-raised px-4 py-3 transition-colors hover:border-brand-400"
  >
    <div>
      <p class="text-sm font-medium text-gray-100">{{ typeLabel }}</p>
      <p class="mt-0.5 font-mono text-xs text-gray-500">{{ shortenTxId(tx.txID) }}</p>
      <p v-if="contract?.parameter.value.to_address" class="mt-0.5 text-xs text-gray-500">
        宛先: {{ shortenAddress(contract.parameter.value.to_address) }}
      </p>
    </div>
    <div class="text-right">
      <p class="text-sm font-semibold text-gray-100">{{ amountLabel }}</p>
      <p class="mt-0.5 text-xs text-gray-500">{{ timestamp }}</p>
      <span class="mt-1 inline-block rounded px-2 py-0.5 text-xs font-medium" :class="statusClass">
        {{ status }}
      </span>
    </div>
  </a>
</template>
