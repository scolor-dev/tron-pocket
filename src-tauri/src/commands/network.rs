use crate::error::AppResult;
use crate::state::AppState;
use crate::storage::settings;
use tauri::{AppHandle, State};
use tron_core::client::Network;

#[tauri::command]
pub fn get_network(state: State<AppState>) -> AppResult<Network> {
    Ok(state.0.lock().unwrap().network)
}

#[tauri::command]
pub fn set_network(app: AppHandle, state: State<AppState>, network: Network) -> AppResult<()> {
    let mut inner = state.0.lock().unwrap();
    inner.network = network;
    let data = settings::AppData {
        keystore: inner.keystore.clone(),
        network,
    };
    settings::save(&app, &data)
}
