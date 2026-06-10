//! Persisted app data: the encrypted keystore and the selected network.
//!
//! Stored as a single JSON file in the OS app-data directory. The keystore
//! itself is encrypted (see `tron_core::keystore`), so this file is safe to
//! read/write without holding any secrets in plaintext.

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;
use tron_core::client::Network;
use tron_core::keystore::Keystore;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppData {
    pub keystore: Option<Keystore>,
    #[serde(default)]
    pub network: Network,
}

fn data_file_path(app: &tauri::AppHandle) -> AppResult<PathBuf> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Storage(e.to_string()))?;
    std::fs::create_dir_all(&dir).map_err(|e| AppError::Storage(e.to_string()))?;
    Ok(dir.join("wallet.json"))
}

pub fn load(app: &tauri::AppHandle) -> AppResult<AppData> {
    let path = data_file_path(app)?;
    if !path.exists() {
        return Ok(AppData::default());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| AppError::Storage(e.to_string()))?;
    serde_json::from_str(&content).map_err(|e| AppError::Storage(e.to_string()))
}

pub fn save(app: &tauri::AppHandle, data: &AppData) -> AppResult<()> {
    let path = data_file_path(app)?;
    let content =
        serde_json::to_string_pretty(data).map_err(|e| AppError::Storage(e.to_string()))?;
    std::fs::write(&path, content).map_err(|e| AppError::Storage(e.to_string()))
}
