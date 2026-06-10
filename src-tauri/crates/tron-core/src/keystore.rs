//! Password-encrypted storage for a wallet's private key material.
//!
//! Format: Argon2id derives a 256-bit key from the user's password and a
//! random salt; that key encrypts the secret (private key bytes) with
//! AES-256-GCM under a random nonce. Everything needed to reverse this
//! (except the password itself) is stored alongside the ciphertext.

use crate::error::{CoreError, Result};
use aes_gcm::aead::{Aead, KeyInit, OsRng as AesOsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::Argon2;
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keystore {
    pub version: u8,
    /// Base58 address, kept for display without decrypting the secret.
    pub address: String,
    pub kdf: KdfParams,
    pub cipher: CipherParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdfParams {
    pub algorithm: String,
    pub salt: String,
    pub mem_cost_kib: u32,
    pub time_cost: u32,
    pub parallelism: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CipherParams {
    pub algorithm: String,
    pub nonce: String,
    pub ciphertext: String,
}

/// Encrypt `secret` (e.g. a 32-byte private key) under `password`.
pub fn encrypt(secret: &[u8], password: &str, address: &str) -> Result<Keystore> {
    let mut salt = [0u8; SALT_LEN];
    AesOsRng.fill_bytes(&mut salt);

    let mem_cost_kib = 19_456; // ~19 MiB, OWASP recommended minimum for Argon2id
    let time_cost = 2;
    let parallelism = 1;

    let key = derive_key(password, &salt, mem_cost_kib, time_cost, parallelism)?;

    let mut nonce_bytes = [0u8; NONCE_LEN];
    AesOsRng.fill_bytes(&mut nonce_bytes);

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key.as_slice()));
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, secret)
        .map_err(|e| CoreError::Encoding(format!("encryption failed: {e}")))?;

    Ok(Keystore {
        version: 1,
        address: address.to_string(),
        kdf: KdfParams {
            algorithm: "argon2id".into(),
            salt: hex::encode(salt),
            mem_cost_kib,
            time_cost,
            parallelism,
        },
        cipher: CipherParams {
            algorithm: "aes-256-gcm".into(),
            nonce: hex::encode(nonce_bytes),
            ciphertext: hex::encode(ciphertext),
        },
    })
}

/// Decrypt a keystore with `password`, returning the original secret bytes.
/// Returns [`CoreError::IncorrectPassword`] if the password is wrong (or data is corrupt).
pub fn decrypt(keystore: &Keystore, password: &str) -> Result<Zeroizing<Vec<u8>>> {
    let salt = hex::decode(&keystore.kdf.salt).map_err(|e| CoreError::Encoding(e.to_string()))?;
    let nonce_bytes =
        hex::decode(&keystore.cipher.nonce).map_err(|e| CoreError::Encoding(e.to_string()))?;
    let ciphertext =
        hex::decode(&keystore.cipher.ciphertext).map_err(|e| CoreError::Encoding(e.to_string()))?;

    let key = derive_key(
        password,
        &salt,
        keystore.kdf.mem_cost_kib,
        keystore.kdf.time_cost,
        keystore.kdf.parallelism,
    )?;

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key.as_slice()));
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| CoreError::IncorrectPassword)?;

    Ok(Zeroizing::new(plaintext))
}

fn derive_key(
    password: &str,
    salt: &[u8],
    mem_cost_kib: u32,
    time_cost: u32,
    parallelism: u32,
) -> Result<Zeroizing<[u8; KEY_LEN]>> {
    let params = argon2::Params::new(mem_cost_kib, time_cost, parallelism, Some(KEY_LEN))
        .map_err(|e| CoreError::Encoding(e.to_string()))?;
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    let mut key = Zeroizing::new([0u8; KEY_LEN]);
    argon2
        .hash_password_into(password.as_bytes(), salt, key.as_mut_slice())
        .map_err(|e| CoreError::Encoding(e.to_string()))?;
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_round_trip() {
        let secret = b"super-secret-32-byte-private-ky";
        let ks = encrypt(secret, "correct horse battery staple", "Tdummyaddress").unwrap();

        let decrypted = decrypt(&ks, "correct horse battery staple").unwrap();
        assert_eq!(&decrypted[..], secret);
    }

    #[test]
    fn wrong_password_fails() {
        let secret = b"super-secret-32-byte-private-ky";
        let ks = encrypt(secret, "correct horse battery staple", "Tdummyaddress").unwrap();

        let result = decrypt(&ks, "wrong password");
        assert!(matches!(result, Err(CoreError::IncorrectPassword)));
    }

    #[test]
    fn serializes_to_json() {
        let secret = b"super-secret-32-byte-private-ky";
        let ks = encrypt(secret, "pw", "Tdummyaddress").unwrap();
        let json = serde_json::to_string(&ks).unwrap();
        let parsed: Keystore = serde_json::from_str(&json).unwrap();
        let decrypted = decrypt(&parsed, "pw").unwrap();
        assert_eq!(&decrypted[..], secret);
    }
}
