//! Mnemonic generation and BIP32/44 key derivation for TRON.

use crate::address::{self, AddressBytes};
use crate::error::{CoreError, Result};
use bip32::{DerivationPath, XPrv};
use bip39::Mnemonic;
use k256::ecdsa::{SigningKey, VerifyingKey};
use rand_core::RngCore;
use std::str::FromStr;
use zeroize::Zeroizing;

/// BIP44 path for TRON's first account / first external address.
/// TRON registered coin type 195: m/44'/195'/0'/0/{index}
pub const TRON_DERIVATION_PATH: &str = "m/44'/195'/0'/0/0";

/// A derived (or imported) secp256k1 keypair, with TRON address helpers.
pub struct KeyPair {
    signing_key: SigningKey,
}

impl KeyPair {
    pub fn from_private_key_bytes(bytes: &[u8]) -> Result<Self> {
        let signing_key = SigningKey::from_slice(bytes).map_err(|_| CoreError::InvalidPrivateKey)?;
        Ok(Self { signing_key })
    }

    pub fn signing_key(&self) -> &SigningKey {
        &self.signing_key
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        *self.signing_key.verifying_key()
    }

    pub fn address_bytes(&self) -> AddressBytes {
        address::address_from_pubkey(&self.verifying_key())
    }

    pub fn address_base58(&self) -> String {
        address::to_base58(&self.address_bytes())
    }

    /// Raw 32-byte private key. Wrapped so it is zeroized on drop.
    pub fn private_key_bytes(&self) -> Zeroizing<[u8; 32]> {
        let bytes: [u8; 32] = self.signing_key.to_bytes().into();
        Zeroizing::new(bytes)
    }
}

/// Generate a new random 12-word BIP39 mnemonic.
pub fn generate_mnemonic() -> Mnemonic {
    let mut entropy = [0u8; 16]; // 128 bits -> 12 words
    rand_core::OsRng.fill_bytes(&mut entropy);
    Mnemonic::from_entropy(&entropy).expect("16 bytes is valid entropy for a 12-word mnemonic")
}

/// Parse and validate a BIP39 mnemonic phrase.
pub fn parse_mnemonic(phrase: &str) -> Result<Mnemonic> {
    Mnemonic::parse(phrase.trim()).map_err(|_| CoreError::InvalidMnemonic)
}

/// Derive the TRON keypair at [`TRON_DERIVATION_PATH`] from a mnemonic.
pub fn derive_keypair(mnemonic: &Mnemonic, passphrase: &str) -> Result<KeyPair> {
    let seed = mnemonic.to_seed(passphrase);
    let path =
        DerivationPath::from_str(TRON_DERIVATION_PATH).map_err(|e| CoreError::Encoding(e.to_string()))?;
    let xprv = XPrv::derive_from_path(seed, &path).map_err(|e| CoreError::Encoding(e.to_string()))?;
    let signing_key = xprv.private_key().clone();
    Ok(KeyPair { signing_key })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_valid_mnemonic() {
        let mnemonic = generate_mnemonic();
        assert_eq!(mnemonic.word_count(), 12);
        // Round-trips through string parsing.
        let phrase = mnemonic.to_string();
        assert!(parse_mnemonic(&phrase).is_ok());
    }

    #[test]
    fn rejects_invalid_mnemonic() {
        assert!(parse_mnemonic("not a real mnemonic phrase at all").is_err());
    }

    #[test]
    fn deterministic_derivation() {
        let mnemonic = parse_mnemonic(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        )
        .unwrap();

        let kp1 = derive_keypair(&mnemonic, "").unwrap();
        let kp2 = derive_keypair(&mnemonic, "").unwrap();

        assert_eq!(kp1.address_base58(), kp2.address_base58());
        assert!(kp1.address_base58().starts_with('T'));

        // Different passphrase -> different address.
        let kp3 = derive_keypair(&mnemonic, "extra").unwrap();
        assert_ne!(kp1.address_base58(), kp3.address_base58());
    }

    #[test]
    fn import_from_private_key() {
        let mnemonic = parse_mnemonic(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        )
        .unwrap();
        let kp = derive_keypair(&mnemonic, "").unwrap();
        let bytes = kp.private_key_bytes();

        let imported = KeyPair::from_private_key_bytes(&*bytes).unwrap();
        assert_eq!(kp.address_base58(), imported.address_base58());
    }
}
