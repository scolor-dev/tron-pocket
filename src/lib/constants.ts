import type { Network } from "@/types";

export const NETWORK_INFO: Record<Network, { label: string; explorerTxUrl: string; explorerAddressUrl: string }> = {
  mainnet: {
    label: "Mainnet",
    explorerTxUrl: "https://tronscan.org/#/transaction/",
    explorerAddressUrl: "https://tronscan.org/#/address/",
  },
  nile: {
    label: "Nile Testnet",
    explorerTxUrl: "https://nile.tronscan.org/#/transaction/",
    explorerAddressUrl: "https://nile.tronscan.org/#/address/",
  },
};

export const USDT_CONTRACT_MAINNET = "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t";

export const TRX_DECIMALS = 6;
export const USDT_DECIMALS = 6;
