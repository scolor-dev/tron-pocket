//! Transaction signing.
//!
//! TRON transaction IDs (`txID`) are the SHA256 hash of the transaction's
//! `raw_data` protobuf bytes. The wallet signs that 32-byte digest directly
//! (no extra hashing) and appends a 65-byte `r || s || v` signature, where
//! `v` is the secp256k1 recovery id (0 or 1).

use crate::error::{CoreError, Result};
use k256::ecdsa::SigningKey;

/// Sign a 32-byte digest (typically a transaction's `txID`), returning a
/// 65-byte `r || s || v` signature suitable for TRON's `signature` field.
pub fn sign_digest(signing_key: &SigningKey, digest: &[u8; 32]) -> Result<[u8; 65]> {
    let (signature, recovery_id) = signing_key
        .sign_prehash_recoverable(digest)
        .map_err(|e| CoreError::Encoding(format!("signing failed: {e}")))?;

    let mut out = [0u8; 65];
    out[..64].copy_from_slice(&signature.to_bytes());
    out[64] = recovery_id.to_byte();
    Ok(out)
}

/// Sign a transaction `txID` given as a hex string, returning the hex-encoded signature.
pub fn sign_tx_id_hex(signing_key: &SigningKey, tx_id_hex: &str) -> Result<String> {
    let digest_vec = hex::decode(tx_id_hex).map_err(|e| CoreError::Encoding(e.to_string()))?;
    let digest: [u8; 32] = digest_vec
        .try_into()
        .map_err(|_| CoreError::Encoding("txID must be 32 bytes".into()))?;
    Ok(hex::encode(sign_digest(signing_key, &digest)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys;

    #[test]
    fn sign_and_recover() {
        let mnemonic = keys::parse_mnemonic(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        )
        .unwrap();
        let kp = keys::derive_keypair(&mnemonic, "").unwrap();

        let digest = [7u8; 32];
        let sig = sign_digest(kp.signing_key(), &digest).unwrap();
        assert_eq!(sig.len(), 65);
        assert!(sig[64] == 0 || sig[64] == 1);

        // The recovery id + (r,s) must recover the original public key.
        let signature = k256::ecdsa::Signature::from_slice(&sig[..64]).unwrap();
        let recid = k256::ecdsa::RecoveryId::from_byte(sig[64]).unwrap();
        let recovered =
            k256::ecdsa::VerifyingKey::recover_from_prehash(&digest, &signature, recid).unwrap();
        assert_eq!(recovered, kp.verifying_key());
    }
}
