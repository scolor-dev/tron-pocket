import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "@/lib/tauri-api";
import type { Balances, CreateWalletResult, Network, ResourceInfo } from "@/types";

const EMPTY_RESOURCES: ResourceInfo = {
  bandwidth_available: 0,
  bandwidth_limit: 0,
  energy_available: 0,
  energy_limit: 0,
};

export const useWalletStore = defineStore("wallet", () => {
  const statusLoaded = ref(false);
  const hasWallet = ref(false);
  const unlocked = ref(false);
  const address = ref<string | null>(null);
  const network = ref<Network>("mainnet");

  const balances = ref<Balances>({ trx: "0", usdt: "0" });
  const balancesLoading = ref(false);

  const resources = ref<ResourceInfo>({ ...EMPTY_RESOURCES });
  const resourcesLoading = ref(false);

  async function refreshStatus() {
    const status = await api.walletStatus();
    hasWallet.value = status.has_wallet;
    unlocked.value = status.unlocked;
    address.value = status.address;
    network.value = status.network;
    statusLoaded.value = true;
  }

  async function refreshBalances() {
    if (!unlocked.value) return;
    balancesLoading.value = true;
    try {
      balances.value = await api.getBalances();
    } finally {
      balancesLoading.value = false;
    }
  }

  async function refreshResources() {
    if (!unlocked.value) return;
    resourcesLoading.value = true;
    try {
      resources.value = await api.getResources();
    } finally {
      resourcesLoading.value = false;
    }
  }

  async function createWallet(password: string): Promise<CreateWalletResult> {
    const result = await api.createWallet(password);
    await refreshStatus();
    return result;
  }

  async function importWallet(mnemonic: string, password: string) {
    await api.importWallet(mnemonic, password);
    await refreshStatus();
  }

  async function unlock(password: string) {
    await api.unlockWallet(password);
    await refreshStatus();
  }

  async function lock() {
    await api.lockWallet();
    await refreshStatus();
  }

  async function setNetwork(next: Network) {
    await api.setNetwork(next);
    await refreshStatus();
    await refreshBalances();
    await refreshResources();
  }

  return {
    statusLoaded,
    hasWallet,
    unlocked,
    address,
    network,
    balances,
    balancesLoading,
    resources,
    resourcesLoading,
    refreshStatus,
    refreshBalances,
    refreshResources,
    createWallet,
    importWallet,
    unlock,
    lock,
    setNetwork,
  };
});
