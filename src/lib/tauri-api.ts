import { invoke } from "@tauri-apps/api/core";
import type { Balances, CreateWalletResult, Network, TronTransactionPage, WalletStatus } from "@/types";

/** Typed wrappers around `invoke()` for every Tauri command exposed by src-tauri. */
export const api = {
  walletStatus: () => invoke<WalletStatus>("wallet_status"),
  createWallet: (password: string) => invoke<CreateWalletResult>("create_wallet", { password }),
  importWallet: (mnemonic: string, password: string) =>
    invoke<string>("import_wallet", { mnemonic, password }),
  unlockWallet: (password: string) => invoke<string>("unlock_wallet", { password }),
  lockWallet: () => invoke<void>("lock_wallet"),

  getNetwork: () => invoke<Network>("get_network"),
  setNetwork: (network: Network) => invoke<void>("set_network", { network }),

  getBalances: () => invoke<Balances>("get_balances"),
  getTransactionHistory: () => invoke<TronTransactionPage>("get_transaction_history"),

  sendTrx: (to: string, amount: string) => invoke<string>("send_trx", { to, amount }),
  sendTrc20: (to: string, amount: string, contract?: string) =>
    invoke<string>("send_trc20", { to, amount, contract }),
};
