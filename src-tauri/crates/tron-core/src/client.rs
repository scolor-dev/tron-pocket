//! Minimal TronGrid HTTP client: account balances, history, and the
//! create-sign-broadcast transaction flow.
//!
//! Building/broadcasting transactions is delegated to TronGrid's REST API
//! (`/wallet/createtransaction`, `/wallet/broadcasttransaction`) so the
//! client doesn't need to implement TRON's protobuf transaction format
//! itself - it only needs to sign the returned `txID`.

use crate::error::{CoreError, Result};
use crate::trc20;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Network {
    #[default]
    Mainnet,
    Nile,
}

impl Network {
    pub fn base_url(&self) -> &'static str {
        match self {
            Network::Mainnet => "https://api.trongrid.io",
            Network::Nile => "https://nile.trongrid.io",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountBalance {
    /// Balance in SUN (1 TRX = 1_000_000 SUN).
    pub trx_sun: u64,
}

/// Bandwidth and Energy usage/limits for an account, from `/wallet/getaccountresources`.
/// Fields are absent from the API response (and default to 0) when an account
/// has no free or staked allowance of that kind.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountResources {
    #[serde(rename = "freeNetUsed", default)]
    pub free_net_used: u64,
    #[serde(rename = "freeNetLimit", default)]
    pub free_net_limit: u64,
    #[serde(rename = "NetUsed", default)]
    pub net_used: u64,
    #[serde(rename = "NetLimit", default)]
    pub net_limit: u64,
    #[serde(rename = "EnergyUsed", default)]
    pub energy_used: u64,
    #[serde(rename = "EnergyLimit", default)]
    pub energy_limit: u64,
}

/// Network fee parameters relevant to estimating transaction costs, from
/// `/wallet/getchainparameters`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChainParameters {
    /// Cost in SUN per byte of bandwidth consumed beyond an account's free/staked allowance.
    pub bandwidth_fee_sun: u64,
    /// Cost in SUN per unit of energy consumed beyond an account's staked allowance.
    pub energy_fee_sun: u64,
}

impl Default for ChainParameters {
    fn default() -> Self {
        // TRON mainnet defaults as of 2024; used as a fallback if the chain
        // parameter is missing from the response for any reason.
        Self {
            bandwidth_fee_sun: 1000,
            energy_fee_sun: 100,
        }
    }
}

/// Parse the `chainParameter: [{key, value}]` array returned by
/// `/wallet/getchainparameters`, falling back to [`ChainParameters::default`]
/// values for any parameter that isn't present.
fn parse_chain_parameters(value: &Value) -> ChainParameters {
    let defaults = ChainParameters::default();
    let find = |key: &str, default: u64| -> u64 {
        value["chainParameter"]
            .as_array()
            .and_then(|params| params.iter().find(|p| p["key"] == key))
            .and_then(|p| p["value"].as_u64())
            .unwrap_or(default)
    };

    ChainParameters {
        bandwidth_fee_sun: find("getTransactionFee", defaults.bandwidth_fee_sun),
        energy_fee_sun: find("getEnergyFee", defaults.energy_fee_sun),
    }
}

/// Estimate the bandwidth (in bytes) a signed transaction will consume,
/// given the hex-encoded `raw_data` of the unsigned transaction.
///
/// This is `len(raw_data) + len(signature)`, plus a small constant for the
/// protobuf field overhead of the appended signature.
pub fn estimate_bandwidth_bytes(raw_data_hex: &str) -> u64 {
    const SIGNATURE_AND_OVERHEAD_BYTES: u64 = 67;
    (raw_data_hex.len() / 2) as u64 + SIGNATURE_AND_OVERHEAD_BYTES
}

/// A transaction built by TronGrid, ready to be signed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedTransaction {
    #[serde(rename = "txID")]
    pub tx_id: String,
    pub raw_data: Value,
    pub raw_data_hex: String,
    #[serde(default)]
    pub visible: bool,
}

pub struct TronClient {
    base_url: String,
    http: reqwest::Client,
}

impl TronClient {
    pub fn new(network: Network) -> Self {
        Self {
            base_url: network.base_url().to_string(),
            http: reqwest::Client::new(),
        }
    }

    /// Fetch the TRX balance (in SUN) for an address. Returns 0 for unactivated accounts.
    pub async fn get_trx_balance(&self, address_hex: &str) -> Result<AccountBalance> {
        let url = format!("{}/v1/accounts/{}", self.base_url, address_hex);
        let resp: Value = self.http.get(&url).send().await?.json().await?;
        let balance = resp["data"]
            .get(0)
            .and_then(|d| d["balance"].as_u64())
            .unwrap_or(0);
        Ok(AccountBalance { trx_sun: balance })
    }

    /// Fetch the TRC20 token balance (in the token's smallest unit) via `balanceOf`.
    pub async fn get_trc20_balance(
        &self,
        owner_address_hex: &str,
        contract_address_hex: &str,
    ) -> Result<u128> {
        let owner_bytes = crate::address::from_hex(owner_address_hex)?;
        let parameter = trc20::encode_balance_of(&owner_bytes);

        let resp = self
            .trigger_constant_contract(
                owner_address_hex,
                contract_address_hex,
                trc20::BALANCE_OF_SELECTOR,
                &parameter,
            )
            .await?;

        let result_hex = resp["constant_result"]
            .get(0)
            .and_then(|v| v.as_str())
            .unwrap_or("0");
        if result_hex.is_empty() {
            return Ok(0);
        }
        u128::from_str_radix(result_hex, 16).map_err(|e| CoreError::Encoding(e.to_string()))
    }

    /// Fetch the bandwidth/energy usage and limits for an account.
    pub async fn get_account_resources(&self, address_hex: &str) -> Result<AccountResources> {
        let url = format!("{}/wallet/getaccountresources", self.base_url);
        let body = json!({ "address": address_hex, "visible": false });
        let resp: Value = self.http.post(&url).json(&body).send().await?.json().await?;
        serde_json::from_value(resp).map_err(CoreError::from)
    }

    /// Fetch network-wide fee parameters (bandwidth/energy price in SUN).
    pub async fn get_chain_parameters(&self) -> Result<ChainParameters> {
        let url = format!("{}/wallet/getchainparameters", self.base_url);
        let resp: Value = self.http.get(&url).send().await?.json().await?;
        Ok(parse_chain_parameters(&resp))
    }

    /// Estimate the energy a TRC20 `transfer(address,uint256)` call will consume,
    /// via a constant (read-only) contract call.
    pub async fn estimate_trc20_transfer_energy(
        &self,
        owner_address_hex: &str,
        contract_address_hex: &str,
        to_address: &[u8; 21],
        amount: u128,
    ) -> Result<u64> {
        let parameter = trc20::encode_transfer(to_address, amount);
        let resp = self
            .trigger_constant_contract(
                owner_address_hex,
                contract_address_hex,
                trc20::TRANSFER_SELECTOR,
                &parameter,
            )
            .await?;
        Ok(resp["energy_used"].as_u64().unwrap_or(0))
    }

    /// Fetch recent transactions for an address (newest first).
    pub async fn get_transactions(&self, address_hex: &str, limit: u32) -> Result<Value> {
        let url = format!(
            "{}/v1/accounts/{address_hex}/transactions?limit={limit}",
            self.base_url
        );
        Ok(self.http.get(&url).send().await?.json().await?)
    }

    /// Build an unsigned TRX transfer transaction.
    pub async fn create_trx_transfer(
        &self,
        owner_address_hex: &str,
        to_address_hex: &str,
        amount_sun: u64,
    ) -> Result<CreatedTransaction> {
        let url = format!("{}/wallet/createtransaction", self.base_url);
        let body = json!({
            "owner_address": owner_address_hex,
            "to_address": to_address_hex,
            "amount": amount_sun,
            "visible": false,
        });
        self.post_for_transaction(&url, &body).await
    }

    /// Build an unsigned TRC20 `transfer` transaction.
    pub async fn create_trc20_transfer(
        &self,
        owner_address_hex: &str,
        contract_address_hex: &str,
        to_address: &[u8; 21],
        amount: u128,
    ) -> Result<CreatedTransaction> {
        let url = format!("{}/wallet/triggersmartcontract", self.base_url);
        let parameter = trc20::encode_transfer(to_address, amount);
        let body = json!({
            "owner_address": owner_address_hex,
            "contract_address": contract_address_hex,
            "function_selector": "transfer(address,uint256)",
            "parameter": parameter,
            "fee_limit": 100_000_000,
            "call_value": 0,
            "visible": false,
        });
        let resp: Value = self.http.post(&url).json(&body).send().await?.json().await?;
        let tx = resp
            .get("transaction")
            .cloned()
            .ok_or_else(|| CoreError::Node(format!("unexpected response: {resp}")))?;
        serde_json::from_value(tx).map_err(CoreError::from)
    }

    /// Broadcast a signed transaction (the `CreatedTransaction` JSON plus a `signature` array).
    pub async fn broadcast_transaction(&self, signed_tx: Value) -> Result<Value> {
        let url = format!("{}/wallet/broadcasttransaction", self.base_url);
        Ok(self.http.post(&url).json(&signed_tx).send().await?.json().await?)
    }

    async fn trigger_constant_contract(
        &self,
        owner_address_hex: &str,
        contract_address_hex: &str,
        function_selector: &str,
        parameter: &str,
    ) -> Result<Value> {
        let url = format!("{}/wallet/triggerconstantcontract", self.base_url);
        let body = json!({
            "owner_address": owner_address_hex,
            "contract_address": contract_address_hex,
            "function_selector": function_selector,
            "parameter": parameter,
            "visible": false,
        });
        Ok(self.http.post(&url).json(&body).send().await?.json().await?)
    }

    async fn post_for_transaction(&self, url: &str, body: &Value) -> Result<CreatedTransaction> {
        let resp: Value = self.http.post(url).json(body).send().await?.json().await?;
        if let Some(err) = resp.get("Error").and_then(|e| e.as_str()) {
            return Err(CoreError::Node(err.to_string()));
        }
        serde_json::from_value(resp).map_err(CoreError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_resources_full_response() {
        let value = json!({
            "freeNetUsed": 557,
            "freeNetLimit": 5000,
            "NetUsed": 261,
            "NetLimit": 5239057,
            "TotalNetLimit": 43200000000_u64,
            "TotalNetWeight": 41884116364_u64,
            "EnergyUsed": 30150,
            "EnergyLimit": 32360,
        });
        let resources: AccountResources = serde_json::from_value(value).unwrap();
        assert_eq!(resources.free_net_used, 557);
        assert_eq!(resources.free_net_limit, 5000);
        assert_eq!(resources.net_used, 261);
        assert_eq!(resources.net_limit, 5239057);
        assert_eq!(resources.energy_used, 30150);
        assert_eq!(resources.energy_limit, 32360);
    }

    #[test]
    fn account_resources_missing_fields_default_to_zero() {
        // An account with only free bandwidth (no staked NET/Energy) omits
        // the NetUsed/NetLimit/EnergyUsed/EnergyLimit fields entirely.
        let value = json!({ "freeNetUsed": 100, "freeNetLimit": 600 });
        let resources: AccountResources = serde_json::from_value(value).unwrap();
        assert_eq!(resources.free_net_used, 100);
        assert_eq!(resources.free_net_limit, 600);
        assert_eq!(resources.net_used, 0);
        assert_eq!(resources.net_limit, 0);
        assert_eq!(resources.energy_used, 0);
        assert_eq!(resources.energy_limit, 0);
    }

    #[test]
    fn parses_chain_parameters() {
        let value = json!({
            "chainParameter": [
                { "key": "getMaintenanceTimeInterval", "value": 21_600_000 },
                { "key": "getTransactionFee", "value": 1000 },
                { "key": "getEnergyFee", "value": 210 },
            ]
        });
        let params = parse_chain_parameters(&value);
        assert_eq!(params.bandwidth_fee_sun, 1000);
        assert_eq!(params.energy_fee_sun, 210);
    }

    #[test]
    fn chain_parameters_fall_back_to_defaults_when_missing() {
        let value = json!({ "chainParameter": [] });
        let params = parse_chain_parameters(&value);
        assert_eq!(params, ChainParameters::default());
    }

    #[test]
    fn estimates_bandwidth_from_raw_data_hex() {
        // 100 hex chars = 50 bytes of raw_data.
        let raw_data_hex = "00".repeat(50);
        assert_eq!(estimate_bandwidth_bytes(&raw_data_hex), 50 + 67);
    }
}
