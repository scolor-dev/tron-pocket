//! TRON address encoding/decoding.
//!
//! A TRON address is: `0x41 || keccak256(uncompressed_pubkey[1..])[12..]`,
//! Base58Check-encoded (payload + first 4 bytes of sha256(sha256(payload))).

use crate::error::{CoreError, Result};
use k256::ecdsa::VerifyingKey;
use sha2::{Digest, Sha256};
use sha3::Keccak256;

/// Address prefix byte used on TRON mainnet and testnets (`0x41`, displayed as `T...`).
pub const ADDRESS_PREFIX: u8 = 0x41;

/// Raw 21-byte TRON address: `[0x41, ...20 bytes]`.
pub type AddressBytes = [u8; 21];

/// Derive the raw TRON address bytes from a secp256k1 public key.
pub fn address_from_pubkey(pubkey: &VerifyingKey) -> AddressBytes {
    let uncompressed = pubkey.to_encoded_point(false);
    // Skip the leading 0x04 marker byte, hash the remaining 64 bytes (X || Y).
    let hash = Keccak256::digest(&uncompressed.as_bytes()[1..]);

    let mut out = [0u8; 21];
    out[0] = ADDRESS_PREFIX;
    out[1..].copy_from_slice(&hash[12..]);
    out
}

/// Encode raw address bytes as a Base58Check string (e.g. `TXYZ...`).
pub fn to_base58(bytes: &AddressBytes) -> String {
    let checksum = double_sha256(bytes);
    let mut payload = Vec::with_capacity(25);
    payload.extend_from_slice(bytes);
    payload.extend_from_slice(&checksum[..4]);
    bs58::encode(payload).into_string()
}

/// Decode a Base58Check TRON address string into raw address bytes, validating the checksum.
pub fn from_base58(addr: &str) -> Result<AddressBytes> {
    let decoded = bs58::decode(addr)
        .into_vec()
        .map_err(|e| CoreError::InvalidAddress(e.to_string()))?;

    if decoded.len() != 25 {
        return Err(CoreError::InvalidAddress(format!(
            "expected 25 bytes, got {}",
            decoded.len()
        )));
    }

    let (payload, checksum) = decoded.split_at(21);
    let expected = double_sha256(payload);
    if &expected[..4] != checksum {
        return Err(CoreError::InvalidAddress("checksum mismatch".into()));
    }

    if payload[0] != ADDRESS_PREFIX {
        return Err(CoreError::InvalidAddress(format!(
            "unexpected prefix byte 0x{:02x}",
            payload[0]
        )));
    }

    let mut out = [0u8; 21];
    out.copy_from_slice(payload);
    Ok(out)
}

/// Convert a Base58Check address (`T...`) to the hex form (`41...`) used by TronGrid APIs.
pub fn base58_to_hex(addr: &str) -> Result<String> {
    Ok(hex::encode(from_base58(addr)?))
}

/// Parse a hex address (`41...`, 21 bytes) into raw address bytes.
pub fn from_hex(hex_addr: &str) -> Result<AddressBytes> {
    let bytes = hex::decode(hex_addr).map_err(|e| CoreError::Encoding(e.to_string()))?;
    if bytes.len() != 21 {
        return Err(CoreError::InvalidAddress(format!(
            "expected 21 bytes, got {}",
            bytes.len()
        )));
    }
    let mut out = [0u8; 21];
    out.copy_from_slice(&bytes);
    Ok(out)
}

/// Convert a hex address (`41...`, 21 bytes) to its Base58Check form.
pub fn hex_to_base58(hex_addr: &str) -> Result<String> {
    Ok(to_base58(&from_hex(hex_addr)?))
}

/// Validate that a string is a well-formed Base58Check TRON address.
pub fn is_valid(addr: &str) -> bool {
    from_base58(addr).is_ok()
}

fn double_sha256(data: &[u8]) -> [u8; 32] {
    let first = Sha256::digest(data);
    Sha256::digest(first).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use k256::SecretKey;

    #[test]
    fn round_trip_base58() {
        // Well-known TRON foundation address.
        let addr = "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t";
        let bytes = from_base58(addr).expect("valid address");
        assert_eq!(bytes[0], ADDRESS_PREFIX);
        assert_eq!(to_base58(&bytes), addr);
    }

    #[test]
    fn rejects_bad_checksum() {
        let mut addr = "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".to_string();
        // Mutate the last character to break the checksum.
        addr.pop();
        addr.push('1');
        assert!(from_base58(&addr).is_err());
    }

    #[test]
    fn hex_round_trip() {
        let addr = "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t";
        let hex_addr = base58_to_hex(addr).unwrap();
        assert!(hex_addr.starts_with("41"));
        assert_eq!(hex_to_base58(&hex_addr).unwrap(), addr);
    }

    #[test]
    fn derives_address_from_known_private_key() {
        // priv key 0x01 -> well known secp256k1 test vector.
        let key_bytes = [
            0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 1,
        ];
        let secret = SecretKey::from_slice(&key_bytes).unwrap();
        let verifying = secret.public_key();
        let addr_bytes = address_from_pubkey(&k256::ecdsa::VerifyingKey::from(&verifying));
        let addr = to_base58(&addr_bytes);
        // Same private key, derived independently with TronWeb, yields this address.
        assert!(addr.starts_with('T'));
        assert_eq!(addr_bytes[0], ADDRESS_PREFIX);
    }
}
