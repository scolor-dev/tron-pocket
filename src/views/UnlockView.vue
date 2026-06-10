<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import { useWalletStore } from "@/stores/wallet";
import { formatError } from "@/lib/format";
import Button from "@/components/ui/Button.vue";
import Input from "@/components/ui/Input.vue";

const router = useRouter();
const wallet = useWalletStore();

const password = ref("");
const error = ref("");
const submitting = ref(false);

async function submit() {
  error.value = "";
  submitting.value = true;
  try {
    await wallet.unlock(password.value);
    await router.push("/");
  } catch (e) {
    error.value = formatError(e);
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <div class="w-full max-w-sm">
    <div class="mb-8 text-center">
      <div class="mx-auto mb-3 h-12 w-12 rounded-full bg-brand-500"></div>
      <h1 class="text-xl font-semibold text-gray-100">ロック中</h1>
      <p class="mt-1 text-sm text-gray-400">パスワードを入力してください</p>
    </div>

    <form class="space-y-4" @submit.prevent="submit">
      <Input v-model="password" type="password" label="パスワード" />
      <p v-if="error" class="text-sm text-red-400">{{ error }}</p>
      <Button type="submit" class="w-full" :disabled="submitting">
        {{ submitting ? "確認中..." : "ロック解除" }}
      </Button>
    </form>
  </div>
</template>
