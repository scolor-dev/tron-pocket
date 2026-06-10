mod commands;
mod error;
mod state;
mod storage;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .setup(|app| {
            let handle = app.handle().clone();
            let data = storage::settings::load(&handle)?;

            let state = app.state::<AppState>();
            let mut inner = state.0.lock().unwrap();
            inner.keystore = data.keystore;
            inner.network = data.network;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::wallet::wallet_status,
            commands::wallet::create_wallet,
            commands::wallet::import_wallet,
            commands::wallet::unlock_wallet,
            commands::wallet::lock_wallet,
            commands::network::get_network,
            commands::network::set_network,
            commands::balance::get_balances,
            commands::transaction::get_transaction_history,
            commands::transaction::send_trx,
            commands::transaction::send_trc20,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
