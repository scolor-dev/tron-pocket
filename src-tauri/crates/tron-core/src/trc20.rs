//! Minimal ABI encoding for TRC20 token calls (subset of the Solidity ABI).

/// 4-byte selector for `transfer(address,uint256)` (`keccak256("transfer(address,uint256)")[..4]`).
pub const TRANSFER_SELECTOR: &str = "a9059cbb";

/// 4-byte selector for `balanceOf(address)` (`keccak256("balanceOf(address)")[..4]`).
pub const BALANCE_OF_SELECTOR: &str = "70a08231";

/// Well-known TRC20 contract addresses (Base58 form).
pub mod contracts {
    /// USDT on TRON mainnet.
    pub const USDT_MAINNET: &str = "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t";
}

/// Encode a TRON address as a 32-byte (64 hex char) ABI parameter:
/// the rightmost 20 bytes of the address, left-padded with zeros.
pub fn encode_address_param(address: &[u8; 21]) -> String {
    format!("{:0>64}", hex::encode(&address[1..]))
}

/// Encode the parameter data for `transfer(address,uint256)`.
///
/// `to_address` is the raw 21-byte TRON address (`0x41 || ...20 bytes`);
/// `amount` is the token amount in its smallest unit (respecting decimals).
pub fn encode_transfer(to_address: &[u8; 21], amount: u128) -> String {
    let amount_param = format!("{:064x}", amount);
    format!("{TRANSFER_SELECTOR}{}{amount_param}", encode_address_param(to_address))
}

/// Encode the parameter data for `balanceOf(address)`.
pub fn encode_balance_of(owner_address: &[u8; 21]) -> String {
    format!("{BALANCE_OF_SELECTOR}{}", encode_address_param(owner_address))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address;

    #[test]
    fn encodes_transfer_call() {
        let to = address::from_base58("TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t").unwrap();
        let data = encode_transfer(&to, 1_000_000); // 1 USDT (6 decimals)

        assert!(data.starts_with(TRANSFER_SELECTOR));
        assert_eq!(data.len(), 8 + 64 + 64);
        // Amount is right-aligned, zero-padded to 32 bytes.
        assert!(data.ends_with(&format!("{:064x}", 1_000_000u128)));
    }
}
