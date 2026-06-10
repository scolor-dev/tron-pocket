import { createRouter, createWebHashHistory } from "vue-router";
import { useWalletStore } from "@/stores/wallet";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/onboarding", name: "onboarding", component: () => import("@/views/OnboardingView.vue") },
    { path: "/unlock", name: "unlock", component: () => import("@/views/UnlockView.vue") },
    { path: "/", name: "dashboard", component: () => import("@/views/DashboardView.vue") },
    { path: "/send", name: "send", component: () => import("@/views/SendView.vue") },
    { path: "/receive", name: "receive", component: () => import("@/views/ReceiveView.vue") },
    { path: "/history", name: "history", component: () => import("@/views/HistoryView.vue") },
    { path: "/settings", name: "settings", component: () => import("@/views/SettingsView.vue") },
  ],
});

router.beforeEach(async (to) => {
  const wallet = useWalletStore();
  if (!wallet.statusLoaded) {
    await wallet.refreshStatus();
  }

  if (!wallet.hasWallet) {
    return to.name === "onboarding" ? true : { name: "onboarding" };
  }

  if (!wallet.unlocked) {
    return to.name === "unlock" ? true : { name: "unlock" };
  }

  if (to.name === "onboarding" || to.name === "unlock") {
    return { name: "dashboard" };
  }

  return true;
});
