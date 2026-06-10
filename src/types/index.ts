export type Network = "mainnet" | "nile";

export interface WalletStatus {
  has_wallet: boolean;
  unlocked: boolean;
  address: string | null;
  network: Network;
}

export interface CreateWalletResult {
  address: string;
  mnemonic: string;
}

export interface Balances {
  trx: string;
  usdt: string;
}

export type AppErrorKind =
  | "InvalidMnemonic"
  | "InvalidPrivateKey"
  | "InvalidAddress"
  | "IncorrectPassword"
  | "NoWallet"
  | "AlreadyExists"
  | "Locked"
  | "Network"
  | "Node"
  | "Storage"
  | "Other";

/** Mirrors `AppError` from src-tauri/src/error.rs (`#[serde(tag = "kind", content = "message")]`). */
export interface AppError {
  kind: AppErrorKind;
  message?: string;
}

export function isAppError(value: unknown): value is AppError {
  return (
    typeof value === "object" &&
    value !== null &&
    "kind" in value &&
    typeof (value as { kind: unknown }).kind === "string"
  );
}

/** TRON transaction shape returned by TronGrid's `/v1/accounts/{address}/transactions`. */
export interface TronTransaction {
  txID: string;
  raw_data: {
    timestamp?: number;
    contract: Array<{
      type: string;
      parameter: {
        value: {
          owner_address?: string;
          to_address?: string;
          amount?: number;
          [key: string]: unknown;
        };
      };
    }>;
  };
  ret?: Array<{ contractRet: string }>;
}

export interface TronTransactionPage {
  data: TronTransaction[];
}
