use crate::error::{AppError, AppResult};
use crate::state::{AppState, AppStateInner};
use crate::storage::settings;
use serde::Serialize;
use tauri::{AppHandle, State};
use tron_core::client::Network;
use tron_core::{keys, keystore};

#[derive(Serialize)]
pub struct WalletStatus {
    pub has_wallet: bool,
    pub unlocked: bool,
    pub address: Option<String>,
    pub network: Network,
}

#[derive(Serialize)]
pub struct CreateWalletResult {
    pub address: String,
    pub mnemonic: String,
}

#[tauri::command]
pub fn wallet_status(state: State<AppState>) -> AppResult<WalletStatus> {
    let inner = state.0.lock().unwrap();
    Ok(WalletStatus {
        has_wallet: inner.keystore.is_some(),
        unlocked: inner.unlocked.is_some(),
        address: inner.address(),
        network: inner.network,
    })
}

/// Generate a brand new wallet (mnemonic + keypair), encrypt it with
/// `password`, persist it, and unlock it for this session.
///
/// The returned mnemonic is shown to the user exactly once - the backend
/// does not retain it.
#[tauri::command]
pub fn create_wallet(
    app: AppHandle,
    state: State<AppState>,
    password: String,
) -> AppResult<CreateWalletResult> {
    let mut inner = state.0.lock().unwrap();
    if inner.keystore.is_some() {
        return Err(AppError::AlreadyExists);
    }

    let mnemonic = keys::generate_mnemonic();
    let keypair = keys::derive_keypair(&mnemonic, "")?;
    let address = keypair.address_base58();
    let ks = keystore::encrypt(&*keypair.private_key_bytes(), &password, &address)?;

    persist_keystore(&app, &mut inner, ks)?;
    inner.unlocked = Some(keypair);

    Ok(CreateWalletResult {
        address,
        mnemonic: mnemonic.to_string(),
    })
}

/// Restore a wallet from an existing BIP39 mnemonic.
#[tauri::command]
pub fn import_wallet(
    app: AppHandle,
    state: State<AppState>,
    mnemonic: String,
    password: String,
) -> AppResult<String> {
    let mut inner = state.0.lock().unwrap();
    if inner.keystore.is_some() {
        return Err(AppError::AlreadyExists);
    }

    let mnemonic = keys::parse_mnemonic(&mnemonic)?;
    let keypair = keys::derive_keypair(&mnemonic, "")?;
    let address = keypair.address_base58();
    let ks = keystore::encrypt(&*keypair.private_key_bytes(), &password, &address)?;

    persist_keystore(&app, &mut inner, ks)?;
    inner.unlocked = Some(keypair);

    Ok(address)
}

/// Decrypt the stored keystore with `password`, unlocking the wallet for this session.
#[tauri::command]
pub fn unlock_wallet(state: State<AppState>, password: String) -> AppResult<String> {
    let mut inner = state.0.lock().unwrap();

    if let Some(kp) = &inner.unlocked {
        return Ok(kp.address_base58());
    }

    let ks = inner.keystore.clone().ok_or(AppError::NoWallet)?;
    let secret = keystore::decrypt(&ks, &password)?;
    let keypair = keys::KeyPair::from_private_key_bytes(secret.as_slice())?;
    let address = keypair.address_base58();
    inner.unlocked = Some(keypair);
    Ok(address)
}

/// Drop the in-memory private key. The encrypted keystore on disk is untouched.
#[tauri::command]
pub fn lock_wallet(state: State<AppState>) -> AppResult<()> {
    state.0.lock().unwrap().unlocked = None;
    Ok(())
}

fn persist_keystore(
    app: &AppHandle,
    inner: &mut AppStateInner,
    ks: keystore::Keystore,
) -> AppResult<()> {
    inner.keystore = Some(ks.clone());
    let data = settings::AppData {
        keystore: Some(ks),
        network: inner.network,
    };
    settings::save(app, &data)
}
