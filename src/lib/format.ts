import type { AppError } from "@/types";
import { isAppError } from "@/types";

export function shortenAddress(address: string, chars = 6): string {
  if (address.length <= chars * 2 + 3) return address;
  return `${address.slice(0, chars)}...${address.slice(-chars)}`;
}

export function shortenTxId(txId: string, chars = 8): string {
  return shortenAddress(txId, chars);
}

const ERROR_MESSAGES: Record<AppError["kind"], string> = {
  InvalidMnemonic: "リカバリーフレーズが正しくありません。",
  InvalidPrivateKey: "秘密鍵が正しくありません。",
  InvalidAddress: "アドレスの形式が正しくありません。",
  IncorrectPassword: "パスワードが正しくありません。",
  NoWallet: "ウォレットが見つかりません。",
  AlreadyExists: "ウォレットは既に作成されています。",
  Locked: "ウォレットがロックされています。",
  Network: "ネットワークへの接続に失敗しました。",
  Node: "ノードでの処理に失敗しました。",
  Storage: "データの保存に失敗しました。",
  Other: "エラーが発生しました。",
};

/** Convert a Tauri command rejection into a user-facing Japanese message. */
export function formatError(error: unknown): string {
  if (isAppError(error)) {
    const base = ERROR_MESSAGES[error.kind] ?? ERROR_MESSAGES.Other;
    return error.message ? `${base} (${error.message})` : base;
  }
  if (error instanceof Error) return error.message;
  return String(error);
}
