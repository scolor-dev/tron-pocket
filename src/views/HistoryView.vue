<script setup lang="ts">
import { onMounted, ref } from "vue";
import type { TronTransaction } from "@/types";
import { api } from "@/lib/tauri-api";
import { formatError } from "@/lib/format";
import TxListItem from "@/components/wallet/TxListItem.vue";
import Button from "@/components/ui/Button.vue";

const transactions = ref<TronTransaction[]>([]);
const loading = ref(false);
const error = ref("");

async function load() {
  loading.value = true;
  error.value = "";
  try {
    const page = await api.getTransactionHistory();
    transactions.value = page.data ?? [];
  } catch (e) {
    error.value = formatError(e);
  } finally {
    loading.value = false;
  }
}

onMounted(load);
</script>

<template>
  <div class="mx-auto max-w-2xl space-y-6">
    <div class="flex items-center justify-between">
      <h1 class="text-2xl font-semibold text-gray-100">取引履歴</h1>
      <Button variant="secondary" :disabled="loading" @click="load">
        {{ loading ? "更新中..." : "更新" }}
      </Button>
    </div>

    <p v-if="error" class="text-sm text-red-400">{{ error }}</p>

    <div v-if="loading && transactions.length === 0" class="text-sm text-gray-400">読み込み中...</div>
    <div v-else-if="transactions.length === 0" class="text-sm text-gray-400">取引履歴はありません。</div>

    <div v-else class="space-y-2">
      <TxListItem v-for="tx in transactions" :key="tx.txID" :tx="tx" />
    </div>
  </div>
</template>
