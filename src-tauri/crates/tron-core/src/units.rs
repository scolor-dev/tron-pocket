//! Conversions between human-readable decimal amounts (e.g. `"1.5"`) and a
//! token's smallest integer unit (e.g. SUN for TRX, or a TRC20's base unit).

use crate::error::{CoreError, Result};

/// Parse a decimal string into the token's smallest unit.
pub fn to_smallest_unit(amount: &str, decimals: u32) -> Result<u128> {
    let amount = amount.trim();
    let decimals = decimals as usize;

    let (int_part, frac_part) = match amount.split_once('.') {
        Some((i, f)) => (i, f),
        None => (amount, ""),
    };

    if frac_part.len() > decimals {
        return Err(CoreError::Encoding("too many decimal places".into()));
    }
    if int_part.is_empty() && frac_part.is_empty() {
        return Err(CoreError::Encoding("empty amount".into()));
    }

    let int_part = if int_part.is_empty() { "0" } else { int_part };
    let mut combined = format!("{int_part}{frac_part}");
    combined.push_str(&"0".repeat(decimals - frac_part.len()));

    combined
        .parse::<u128>()
        .map_err(|e| CoreError::Encoding(format!("invalid amount: {e}")))
}

/// Format the token's smallest unit as a trimmed decimal string.
pub fn from_smallest_unit(amount: u128, decimals: u32) -> String {
    let decimals = decimals as usize;
    if decimals == 0 {
        return amount.to_string();
    }

    let digits = amount.to_string();
    let padded = format!("{digits:0>width$}", width = decimals + 1);
    let split_at = padded.len() - decimals;
    let (int_part, frac_part) = padded.split_at(split_at);

    let frac_trimmed = frac_part.trim_end_matches('0');
    if frac_trimmed.is_empty() {
        int_part.to_string()
    } else {
        format!("{int_part}.{frac_trimmed}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trx_amounts() {
        assert_eq!(to_smallest_unit("1", 6).unwrap(), 1_000_000);
        assert_eq!(to_smallest_unit("1.5", 6).unwrap(), 1_500_000);
        assert_eq!(to_smallest_unit("0.000001", 6).unwrap(), 1);
        assert_eq!(to_smallest_unit(".5", 6).unwrap(), 500_000);
        assert!(to_smallest_unit("1.0000001", 6).is_err());
    }

    #[test]
    fn formats_back_to_decimal() {
        assert_eq!(from_smallest_unit(1_000_000, 6), "1");
        assert_eq!(from_smallest_unit(1_500_000, 6), "1.5");
        assert_eq!(from_smallest_unit(1, 6), "0.000001");
        assert_eq!(from_smallest_unit(0, 6), "0");
    }

    #[test]
    fn round_trips() {
        for s in ["1", "0.5", "123.456789", "0.000001", "1000"] {
            let units = to_smallest_unit(s, 6).unwrap();
            assert_eq!(from_smallest_unit(units, 6), s);
        }
    }
}
