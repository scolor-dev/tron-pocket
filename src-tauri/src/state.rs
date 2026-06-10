use std::sync::Mutex;
use tron_core::client::Network;
use tron_core::keys::KeyPair;
use tron_core::keystore::Keystore;

/// Shared application state, managed by Tauri and accessed from commands.
///
/// `unlocked` holds the decrypted keypair only in memory for the duration of
/// the session - it is never written to disk. `keystore` is the encrypted
/// blob that *is* persisted (via `storage::settings`).
#[derive(Default)]
pub struct AppState(pub Mutex<AppStateInner>);

#[derive(Default)]
pub struct AppStateInner {
    pub network: Network,
    pub keystore: Option<Keystore>,
    pub unlocked: Option<KeyPair>,
}

impl AppStateInner {
    pub fn address(&self) -> Option<String> {
        self.keystore.as_ref().map(|k| k.address.clone())
    }
}
