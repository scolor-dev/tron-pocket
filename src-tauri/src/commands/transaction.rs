use crate::error::{AppError, AppResult};
use crate::state::AppState;
use k256::ecdsa::SigningKey;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;
use tron_core::client::{estimate_bandwidth_bytes, Network, TronClient};
use tron_core::{address, sign, trc20, units};

#[tauri::command]
pub async fn get_transaction_history(state: State<'_, AppState>) -> AppResult<Value> {
    let (network, address_b58) = {
        let inner = state.0.lock().unwrap();
        let address = inner.address().ok_or(AppError::NoWallet)?;
        (inner.network, address)
    };

    let address_hex = address::base58_to_hex(&address_b58)?;
    let client = TronClient::new(network);
    Ok(client.get_transactions(&address_hex, 20).await?)
}

/// Send TRX. `amount` is a decimal string in TRX (e.g. `"1.5"`).
#[tauri::command]
pub async fn send_trx(state: State<'_, AppState>, to: String, amount: String) -> AppResult<String> {
    let (network, signing_key, owner_hex) = unlocked_signing_context(&state)?;

    if !address::is_valid(&to) {
        return Err(AppError::InvalidAddress(to));
    }
    let to_hex = address::base58_to_hex(&to)?;
    let amount_sun = units::to_smallest_unit(&amount, 6)?;
    if amount_sun > u64::MAX as u128 {
        return Err(AppError::Other("amount too large".into()));
    }

    let client = TronClient::new(network);
    let created = client
        .create_trx_transfer(&owner_hex, &to_hex, amount_sun as u64)
        .await?;

    let signed = sign_created_transaction(&signing_key, &created)?;
    broadcast(&client, signed, &created.tx_id).await
}

/// Send a TRC20 token (defaults to USDT). `amount` is a decimal string in the
/// token's display unit (e.g. `"10"` USDT), assuming 6 decimals.
#[tauri::command]
pub async fn send_trc20(
    state: State<'_, AppState>,
    to: String,
    amount: String,
    contract: Option<String>,
) -> AppResult<String> {
    let (network, signing_key, owner_hex) = unlocked_signing_context(&state)?;

    let to_bytes = address::from_base58(&to)?;
    let contract_b58 = contract.unwrap_or_else(|| trc20::contracts::USDT_MAINNET.to_string());
    let contract_hex = address::base58_to_hex(&contract_b58)?;
    let amount_units = units::to_smallest_unit(&amount, 6)?;

    let client = TronClient::new(network);
    let created = client
        .create_trc20_transfer(&owner_hex, &contract_hex, &to_bytes, amount_units)
        .await?;

    let signed = sign_created_transaction(&signing_key, &created)?;
    broadcast(&client, signed, &created.tx_id).await
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum FeeToken {
    Trx,
    Usdt,
}

#[derive(Serialize)]
pub struct FeeEstimate {
    pub bandwidth_required: u64,
    pub bandwidth_available: u64,
    pub energy_required: u64,
    pub energy_available: u64,
    /// Estimated network fee paid in TRX (decimal string), for whatever
    /// portion of bandwidth/energy exceeds the account's free/staked allowance.
    pub estimated_fee_trx: String,
}

/// Estimate the network fee for sending `amount` of `token` to `to`, based on
/// the sender's current bandwidth/energy allowance. Does not require the
/// wallet to be unlocked - it only inspects the account's public address.
#[tauri::command]
pub async fn estimate_fee(
    state: State<'_, AppState>,
    token: FeeToken,
    to: String,
    amount: String,
    contract: Option<String>,
) -> AppResult<FeeEstimate> {
    if !address::is_valid(&to) {
        return Err(AppError::InvalidAddress(to));
    }

    let (network, owner_b58) = {
        let inner = state.0.lock().unwrap();
        let address = inner.address().ok_or(AppError::NoWallet)?;
        (inner.network, address)
    };

    let owner_hex = address::base58_to_hex(&owner_b58)?;
    let to_hex = address::base58_to_hex(&to)?;
    let client = TronClient::new(network);

    let resources = client.get_account_resources(&owner_hex).await?;
    let bandwidth_available = (resources.free_net_limit + resources.net_limit)
        .saturating_sub(resources.free_net_used + resources.net_used);
    let energy_available = resources.energy_limit.saturating_sub(resources.energy_used);
    let chain_params = client.get_chain_parameters().await?;

    let (bandwidth_required, energy_required) = match token {
        FeeToken::Trx => {
            let amount_sun = units::to_smallest_unit(&amount, 6)?;
            if amount_sun > u64::MAX as u128 {
                return Err(AppError::Other("amount too large".into()));
            }
            let created = client
                .create_trx_transfer(&owner_hex, &to_hex, amount_sun as u64)
                .await?;
            (estimate_bandwidth_bytes(&created.raw_data_hex), 0)
        }
        FeeToken::Usdt => {
            let to_bytes = address::from_base58(&to)?;
            let contract_b58 = contract.unwrap_or_else(|| trc20::contracts::USDT_MAINNET.to_string());
            let contract_hex = address::base58_to_hex(&contract_b58)?;
            let amount_units = units::to_smallest_unit(&amount, 6)?;

            let energy_required = client
                .estimate_trc20_transfer_energy(&owner_hex, &contract_hex, &to_bytes, amount_units)
                .await?;
            let created = client
                .create_trc20_transfer(&owner_hex, &contract_hex, &to_bytes, amount_units)
                .await?;
            (estimate_bandwidth_bytes(&created.raw_data_hex), energy_required)
        }
    };

    let bandwidth_shortfall = bandwidth_required.saturating_sub(bandwidth_available);
    let energy_shortfall = energy_required.saturating_sub(energy_available);
    let fee_sun = bandwidth_shortfall * chain_params.bandwidth_fee_sun
        + energy_shortfall * chain_params.energy_fee_sun;

    Ok(FeeEstimate {
        bandwidth_required,
        bandwidth_available,
        energy_required,
        energy_available,
        estimated_fee_trx: units::from_smallest_unit(fee_sun as u128, 6),
    })
}

/// Extract (network, signing key, owner address hex) from the unlocked wallet,
/// without holding the state mutex across an `.await`.
fn unlocked_signing_context(state: &State<AppState>) -> AppResult<(Network, SigningKey, String)> {
    let inner = state.0.lock().unwrap();
    let keypair = inner.unlocked.as_ref().ok_or(AppError::Locked)?;
    let owner_hex = address::base58_to_hex(&keypair.address_base58())?;
    Ok((inner.network, keypair.signing_key().clone(), owner_hex))
}

fn sign_created_transaction(
    signing_key: &SigningKey,
    created: &tron_core::client::CreatedTransaction,
) -> AppResult<Value> {
    let tx_id_bytes: [u8; 32] = hex::decode(&created.tx_id)
        .map_err(|e| AppError::Other(e.to_string()))?
        .try_into()
        .map_err(|_| AppError::Other("txID must be 32 bytes".into()))?;

    let signature = sign::sign_digest(signing_key, &tx_id_bytes)?;

    let mut tx_value = serde_json::to_value(created)?;
    tx_value["signature"] = serde_json::json!([hex::encode(signature)]);
    Ok(tx_value)
}

async fn broadcast(client: &TronClient, signed_tx: Value, tx_id: &str) -> AppResult<String> {
    let result = client.broadcast_transaction(signed_tx).await?;
    if result["result"].as_bool() == Some(true) {
        return Ok(tx_id.to_string());
    }

    let code = result["code"].as_str().unwrap_or("UNKNOWN_ERROR");
    let message = result["message"]
        .as_str()
        .and_then(|hex_msg| hex::decode(hex_msg).ok())
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_else(|| code.to_string());
    Err(AppError::Node(message))
}
