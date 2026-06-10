<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import { useWalletStore } from "@/stores/wallet";
import { api } from "@/lib/tauri-api";
import { formatError } from "@/lib/format";
import Button from "@/components/ui/Button.vue";
import Input from "@/components/ui/Input.vue";

type Step = "choice" | "create-password" | "create-mnemonic" | "create-confirm" | "import";

const router = useRouter();
const wallet = useWalletStore();

const step = ref<Step>("choice");
const error = ref("");
const submitting = ref(false);

// Create flow state
const createPassword = ref("");
const createPasswordConfirm = ref("");
const generatedAddress = ref("");
const generatedMnemonic = ref("");
const mnemonicConfirmed = ref(false);

const mnemonicWords = computed(() => generatedMnemonic.value.split(" "));

// Import flow state
const importMnemonic = ref("");
const importPassword = ref("");
const importPasswordConfirm = ref("");

function startCreate() {
  error.value = "";
  step.value = "create-password";
}

function startImport() {
  error.value = "";
  step.value = "import";
}

function backToChoice() {
  error.value = "";
  step.value = "choice";
}

function validatePassword(password: string, confirm: string): string | null {
  if (password.length < 8) return "パスワードは8文字以上で入力してください。";
  if (password !== confirm) return "パスワードが一致しません。";
  return null;
}

async function submitCreatePassword() {
  const validationError = validatePassword(createPassword.value, createPasswordConfirm.value);
  if (validationError) {
    error.value = validationError;
    return;
  }

  error.value = "";
  submitting.value = true;
  try {
    const result = await api.createWallet(createPassword.value);
    generatedAddress.value = result.address;
    generatedMnemonic.value = result.mnemonic;
    step.value = "create-mnemonic";
  } catch (e) {
    error.value = formatError(e);
  } finally {
    submitting.value = false;
  }
}

function proceedToConfirm() {
  step.value = "create-confirm";
}

async function finishCreate() {
  submitting.value = true;
  try {
    await wallet.refreshStatus();
    await router.push("/");
  } catch (e) {
    error.value = formatError(e);
  } finally {
    submitting.value = false;
  }
}

async function submitImport() {
  const words = importMnemonic.value.trim().split(/\s+/);
  if (words.length !== 12 && words.length !== 24) {
    error.value = "リカバリーフレーズは12または24単語で入力してください。";
    return;
  }
  const validationError = validatePassword(importPassword.value, importPasswordConfirm.value);
  if (validationError) {
    error.value = validationError;
    return;
  }

  error.value = "";
  submitting.value = true;
  try {
    await wallet.importWallet(words.join(" "), importPassword.value);
    await router.push("/");
  } catch (e) {
    error.value = formatError(e);
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <div class="w-full max-w-md">
    <div class="mb-8 text-center">
      <div class="mx-auto mb-3 h-12 w-12 rounded-full bg-brand-500"></div>
      <h1 class="text-xl font-semibold text-gray-100">Tron Pocket へようこそ</h1>
    </div>

    <!-- Step: choice -->
    <div v-if="step === 'choice'" class="space-y-3">
      <Button class="w-full" @click="startCreate">新しいウォレットを作成</Button>
      <Button class="w-full" variant="secondary" @click="startImport">リカバリーフレーズから復元</Button>
    </div>

    <!-- Step: create-password -->
    <form v-else-if="step === 'create-password'" class="space-y-4" @submit.prevent="submitCreatePassword">
      <p class="text-sm text-gray-400">ウォレットを保護するパスワードを設定してください。</p>
      <Input v-model="createPassword" type="password" label="パスワード" placeholder="8文字以上" />
      <Input v-model="createPasswordConfirm" type="password" label="パスワード(確認)" />
      <p v-if="error" class="text-sm text-red-400">{{ error }}</p>
      <div class="flex gap-2">
        <Button type="button" variant="ghost" @click="backToChoice">戻る</Button>
        <Button type="submit" class="flex-1" :disabled="submitting">
          {{ submitting ? "作成中..." : "ウォレットを作成" }}
        </Button>
      </div>
    </form>

    <!-- Step: create-mnemonic -->
    <div v-else-if="step === 'create-mnemonic'" class="space-y-4">
      <div class="rounded-lg border border-brand-500/40 bg-brand-500/10 p-3 text-sm text-brand-100">
        このリカバリーフレーズは、ウォレットの復元に必要な唯一の情報です。
        紙に書き写すなどして安全な場所に保管し、誰にも共有しないでください。
      </div>
      <div class="grid grid-cols-3 gap-2 rounded-lg border border-surface-border bg-surface-raised p-4">
        <div
          v-for="(word, index) in mnemonicWords"
          :key="index"
          class="rounded bg-surface px-2 py-1.5 font-mono text-sm text-gray-200"
        >
          <span class="mr-1.5 text-gray-500">{{ index + 1 }}.</span>{{ word }}
        </div>
      </div>
      <p class="font-mono text-xs text-gray-500">アドレス: {{ generatedAddress }}</p>
      <Button class="w-full" @click="proceedToConfirm">次へ</Button>
    </div>

    <!-- Step: create-confirm -->
    <div v-else-if="step === 'create-confirm'" class="space-y-4">
      <label class="flex items-start gap-3 rounded-lg border border-surface-border bg-surface-raised p-4 text-sm text-gray-200">
        <input v-model="mnemonicConfirmed" type="checkbox" class="mt-0.5 h-4 w-4 accent-brand-500" />
        <span>リカバリーフレーズを安全な場所に控えました。これを失うとウォレットを復元できなくなります。</span>
      </label>
      <p v-if="error" class="text-sm text-red-400">{{ error }}</p>
      <Button class="w-full" :disabled="!mnemonicConfirmed || submitting" @click="finishCreate">
        {{ submitting ? "処理中..." : "完了" }}
      </Button>
    </div>

    <!-- Step: import -->
    <form v-else-if="step === 'import'" class="space-y-4" @submit.prevent="submitImport">
      <label class="block">
        <span class="mb-1.5 block text-sm font-medium text-gray-300">リカバリーフレーズ</span>
        <textarea
          v-model="importMnemonic"
          rows="3"
          placeholder="12個または24個の単語をスペース区切りで入力"
          class="w-full rounded-lg border border-surface-border bg-surface px-3 py-2 text-sm text-gray-100 placeholder:text-gray-500 focus:border-brand-400 focus:outline-none focus:ring-1 focus:ring-brand-400"
        />
      </label>
      <Input v-model="importPassword" type="password" label="パスワード" placeholder="8文字以上" />
      <Input v-model="importPasswordConfirm" type="password" label="パスワード(確認)" />
      <p v-if="error" class="text-sm text-red-400">{{ error }}</p>
      <div class="flex gap-2">
        <Button type="button" variant="ghost" @click="backToChoice">戻る</Button>
        <Button type="submit" class="flex-1" :disabled="submitting">
          {{ submitting ? "復元中..." : "ウォレットを復元" }}
        </Button>
      </div>
    </form>
  </div>
</template>
