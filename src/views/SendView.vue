<script setup lang="ts">
import { computed, ref } from "vue";
import { useWalletStore } from "@/stores/wallet";
import { NETWORK_INFO, USDT_CONTRACT_MAINNET } from "@/lib/constants";
import { formatError, shortenTxId } from "@/lib/format";
import { api } from "@/lib/tauri-api";
import Button from "@/components/ui/Button.vue";
import Input from "@/components/ui/Input.vue";
import Modal from "@/components/ui/Modal.vue";

const ADDRESS_PATTERN = /^T[1-9A-HJ-NP-Za-km-z]{33}$/;
const AMOUNT_PATTERN = /^\d+(\.\d{1,6})?$/;

const wallet = useWalletStore();

const token = ref<"TRX" | "USDT">("TRX");
const toAddress = ref("");
const amount = ref("");

const error = ref("");
const fieldErrors = ref<{ to?: string; amount?: string }>({});
const showConfirm = ref(false);
const sending = ref(false);
const txId = ref<string | null>(null);

const explorerTxUrl = computed(() => (txId.value ? `${NETWORK_INFO[wallet.network].explorerTxUrl}${txId.value}` : ""));

function validate(): boolean {
  const errors: { to?: string; amount?: string } = {};
  if (!ADDRESS_PATTERN.test(toAddress.value)) {
    errors.to = "アドレスの形式が正しくありません。";
  }
  if (!AMOUNT_PATTERN.test(amount.value) || Number(amount.value) <= 0) {
    errors.amount = "正しい金額を入力してください。";
  }
  fieldErrors.value = errors;
  return Object.keys(errors).length === 0;
}

function openConfirm() {
  error.value = "";
  txId.value = null;
  if (!validate()) return;
  showConfirm.value = true;
}

async function confirmSend() {
  sending.value = true;
  error.value = "";
  try {
    if (token.value === "TRX") {
      txId.value = await api.sendTrx(toAddress.value, amount.value);
    } else {
      txId.value = await api.sendTrc20(toAddress.value, amount.value, USDT_CONTRACT_MAINNET);
    }
    showConfirm.value = false;
    toAddress.value = "";
    amount.value = "";
    await wallet.refreshBalances();
  } catch (e) {
    error.value = formatError(e);
  } finally {
    sending.value = false;
  }
}
</script>

<template>
  <div class="mx-auto max-w-md space-y-6">
    <h1 class="text-2xl font-semibold text-gray-100">送金</h1>

    <div v-if="txId" class="space-y-3 rounded-lg border border-green-500/40 bg-green-500/10 p-4 text-sm text-green-100">
      <p>送金リクエストを送信しました。</p>
      <a :href="explorerTxUrl" target="_blank" rel="noreferrer" class="font-mono text-xs underline">
        {{ shortenTxId(txId) }}
      </a>
    </div>

    <form class="space-y-4" @submit.prevent="openConfirm">
      <label class="block">
        <span class="mb-1.5 block text-sm font-medium text-gray-300">トークン</span>
        <select
          v-model="token"
          class="w-full rounded-lg border border-surface-border bg-surface px-3 py-2 text-sm text-gray-100 focus:border-brand-400 focus:outline-none focus:ring-1 focus:ring-brand-400"
        >
          <option value="TRX">TRX</option>
          <option v-if="wallet.network === 'mainnet'" value="USDT">USDT</option>
        </select>
      </label>

      <Input v-model="toAddress" label="宛先アドレス" placeholder="T..." :error="fieldErrors.to" />
      <Input v-model="amount" label="金額" placeholder="0.0" :error="fieldErrors.amount" :hint="`単位: ${token}`" />

      <p v-if="error" class="text-sm text-red-400">{{ error }}</p>

      <Button type="submit" class="w-full">確認画面へ</Button>
    </form>

    <Modal v-model:open="showConfirm" title="送金内容の確認" description="内容をご確認のうえ、送金を実行してください。">
      <div class="space-y-2 text-sm">
        <div class="flex justify-between">
          <span class="text-gray-400">ネットワーク</span>
          <span class="text-gray-100">{{ NETWORK_INFO[wallet.network].label }}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-gray-400">トークン</span>
          <span class="text-gray-100">{{ token }}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-gray-400">金額</span>
          <span class="text-gray-100">{{ amount }} {{ token }}</span>
        </div>
        <div class="break-all text-right">
          <span class="block text-gray-400">宛先</span>
          <span class="font-mono text-xs text-gray-100">{{ toAddress }}</span>
        </div>
      </div>
      <p v-if="error" class="mt-3 text-sm text-red-400">{{ error }}</p>

      <template #footer>
        <Button variant="secondary" :disabled="sending" @click="showConfirm = false">キャンセル</Button>
        <Button :disabled="sending" @click="confirmSend">{{ sending ? "送信中..." : "送金する" }}</Button>
      </template>
    </Modal>
  </div>
</template>
