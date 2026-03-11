/// Convert a decimal string (e.g. "10.50") to i128 using the given asset_scale.
/// With scale 2: "10.50" → 1050, "10" → 1000, "0.01" → 1.
pub fn decimal_str_to_i128(s: &str, asset_scale: u32) -> Result<i128, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("Amount cannot be empty".to_string());
    }

    let negative = s.starts_with('-');
    let s = if negative { &s[1..] } else { s };

    // Handle trailing dot (e.g. "10.")
    let s = s.strip_suffix('.').unwrap_or(s);

    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() > 2 {
        return Err("Invalid decimal format: multiple decimal points".to_string());
    }
    if parts[0].is_empty() {
        return Err("Invalid decimal format: missing integer part".to_string());
    }

    let integer_part: i128 = parts[0]
        .parse()
        .map_err(|_| format!("Invalid integer part: {}", parts[0]))?;

    let fractional_str = if parts.len() == 2 { parts[1] } else { "" };

    if fractional_str.len() > asset_scale as usize {
        return Err(format!(
            "Too many decimal places: got {} but asset_scale is {}",
            fractional_str.len(),
            asset_scale
        ));
    }

    let fractional: i128 = if fractional_str.is_empty() {
        0
    } else {
        fractional_str
            .parse()
            .map_err(|_| format!("Invalid fractional part: {}", fractional_str))?
    };

    let padding = asset_scale as usize - fractional_str.len();
    let padded_fractional = fractional * 10i128.pow(padding as u32);

    let scale_factor = 10i128.pow(asset_scale);
    let result = integer_part * scale_factor + padded_fractional;

    Ok(if negative { -result } else { result })
}

/// Convert an i128 amount to a decimal display string using asset_scale.
/// With scale 2: 1050 → "10.50", 0 → "0.00", -1050 → "-10.50".
pub fn i128_to_decimal_str(val: i128, asset_scale: u32) -> String {
    if asset_scale == 0 {
        return val.to_string();
    }

    let negative = val < 0;
    let abs_val = val.unsigned_abs();
    let scale = 10u128.pow(asset_scale);
    let integer_part = abs_val / scale;
    let fractional_part = abs_val % scale;

    let sign = if negative { "-" } else { "" };
    format!(
        "{sign}{integer_part}.{fractional_part:0>width$}",
        width = asset_scale as usize
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decimal_str_to_i128_basic() {
        assert_eq!(decimal_str_to_i128("10.50", 2).unwrap(), 1050);
        assert_eq!(decimal_str_to_i128("10", 2).unwrap(), 1000);
        assert_eq!(decimal_str_to_i128("0.01", 2).unwrap(), 1);
        assert_eq!(decimal_str_to_i128("10.5", 2).unwrap(), 1050);
        assert_eq!(decimal_str_to_i128("100", 0).unwrap(), 100);
    }

    #[test]
    fn test_decimal_str_to_i128_negative() {
        assert_eq!(decimal_str_to_i128("-10.50", 2).unwrap(), -1050);
        assert_eq!(decimal_str_to_i128("-1", 0).unwrap(), -1);
    }

    #[test]
    fn test_decimal_str_to_i128_high_precision() {
        assert_eq!(
            decimal_str_to_i128("1.5", 18).unwrap(),
            1_500_000_000_000_000_000
        );
        assert_eq!(
            decimal_str_to_i128("0.000000000000000001", 18).unwrap(),
            1
        );
    }

    #[test]
    fn test_decimal_str_to_i128_too_many_decimals() {
        assert!(decimal_str_to_i128("0.001", 2).is_err());
    }

    #[test]
    fn test_decimal_str_to_i128_trailing_dot() {
        assert_eq!(decimal_str_to_i128("10.", 2).unwrap(), 1000);
    }

    #[test]
    fn test_i128_to_decimal_str_basic() {
        assert_eq!(i128_to_decimal_str(1050, 2), "10.50");
        assert_eq!(i128_to_decimal_str(0, 2), "0.00");
        assert_eq!(i128_to_decimal_str(1, 2), "0.01");
        assert_eq!(i128_to_decimal_str(100, 0), "100");
    }

    #[test]
    fn test_i128_to_decimal_str_negative() {
        assert_eq!(i128_to_decimal_str(-1050, 2), "-10.50");
    }

    #[test]
    fn test_i128_to_decimal_str_high_precision() {
        assert_eq!(
            i128_to_decimal_str(1_500_000_000_000_000_000, 18),
            "1.500000000000000000"
        );
    }

    #[test]
    fn test_roundtrip() {
        for (s, scale) in [("10.50", 2), ("0.01", 2), ("100", 0), ("1.5", 18)] {
            let val = decimal_str_to_i128(s, scale).unwrap();
            let back = i128_to_decimal_str(val, scale);
            let val2 = decimal_str_to_i128(&back, scale).unwrap();
            assert_eq!(val, val2, "Round-trip failed for {s} scale {scale}");
        }
    }
}
