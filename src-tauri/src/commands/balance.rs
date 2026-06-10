use crate::error::{AppError, AppResult};
use crate::state::AppState;
use serde::Serialize;
use tauri::State;
use tron_core::client::{Network, TronClient};
use tron_core::{address, trc20, units};

#[derive(Serialize)]
pub struct Balances {
    /// TRX balance, as a decimal string (e.g. "12.5").
    pub trx: String,
    /// USDT (TRC20) balance, as a decimal string. "0" on networks without a known USDT contract.
    pub usdt: String,
}

#[tauri::command]
pub async fn get_balances(state: State<'_, AppState>) -> AppResult<Balances> {
    let (network, address_b58) = {
        let inner = state.0.lock().unwrap();
        let address = inner.address().ok_or(AppError::NoWallet)?;
        (inner.network, address)
    };

    let address_hex = address::base58_to_hex(&address_b58)?;
    let client = TronClient::new(network);

    let trx_balance = client.get_trx_balance(&address_hex).await?;
    let trx = units::from_smallest_unit(trx_balance.trx_sun as u128, 6);

    let usdt = match network {
        Network::Mainnet => {
            let contract_hex = address::base58_to_hex(trc20::contracts::USDT_MAINNET)?;
            let raw = client
                .get_trc20_balance(&address_hex, &contract_hex)
                .await
                .unwrap_or(0);
            units::from_smallest_unit(raw, 6)
        }
        Network::Nile => "0".to_string(),
    };

    Ok(Balances { trx, usdt })
}
