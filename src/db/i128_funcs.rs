use rusqlite::functions::{Aggregate, Context, FunctionFlags};
use rusqlite::types::{ToSqlOutput, Value};
use rusqlite::{Connection, Result};

/// Encode an i128 as a 16-byte big-endian BLOB with MSB XOR 0x80.
/// This encoding makes memcmp produce correct signed ordering.
pub fn encode_i128(val: i128) -> [u8; 16] {
    let mut bytes = val.to_be_bytes();
    bytes[0] ^= 0x80;
    bytes
}

/// Decode a 16-byte BLOB back to i128 (reverse of encode_i128).
pub fn decode_i128(blob: &[u8]) -> i128 {
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(blob);
    bytes[0] ^= 0x80;
    i128::from_be_bytes(bytes)
}

fn extract_i128(ctx: &Context<'_>, idx: usize) -> Result<Option<i128>> {
    let value = ctx.get_raw(idx);
    match value {
        rusqlite::types::ValueRef::Null => Ok(None),
        rusqlite::types::ValueRef::Blob(b) => {
            if b.len() != 16 {
                return Err(rusqlite::Error::UserFunctionError(
                    format!("i128 BLOB must be 16 bytes, got {}", b.len()).into(),
                ));
            }
            Ok(Some(decode_i128(b)))
        }
        _ => Err(rusqlite::Error::UserFunctionError(
            "Expected BLOB or NULL for i128 value".into(),
        )),
    }
}

struct SumI128;

impl Aggregate<i128, Option<Vec<u8>>> for SumI128 {
    fn init(&self, _ctx: &mut Context<'_>) -> Result<i128> {
        Ok(0i128)
    }

    fn step(&self, ctx: &mut Context<'_>, acc: &mut i128) -> Result<()> {
        if let Some(val) = extract_i128(ctx, 0)? {
            *acc = acc.checked_add(val).ok_or_else(|| {
                rusqlite::Error::UserFunctionError("i128 overflow in sum_i128".into())
            })?;
        }
        Ok(())
    }

    fn finalize(&self, _ctx: &mut Context<'_>, acc: Option<i128>) -> Result<Option<Vec<u8>>> {
        Ok(acc.map(|val| encode_i128(val).to_vec()))
    }
}

pub fn register_i128_functions(conn: &Connection) -> Result<()> {
    // i128_add(a, b) -> BLOB
    conn.create_scalar_function(
        "i128_add",
        2,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let a = extract_i128(ctx, 0)?;
            let b = extract_i128(ctx, 1)?;
            match (a, b) {
                (Some(a), Some(b)) => {
                    let sum = a.checked_add(b).ok_or_else(|| {
                        rusqlite::Error::UserFunctionError("i128 overflow in i128_add".into())
                    })?;
                    Ok(ToSqlOutput::Owned(Value::Blob(encode_i128(sum).to_vec())))
                }
                _ => Ok(ToSqlOutput::Owned(Value::Null)),
            }
        },
    )?;

    // sum_i128(column) -> BLOB (aggregate)
    conn.create_aggregate_function(
        "sum_i128",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        SumI128,
    )?;

    // i128_to_text(blob) -> TEXT
    conn.create_scalar_function(
        "i128_to_text",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let val = extract_i128(ctx, 0)?;
            match val {
                Some(v) => Ok(ToSqlOutput::Owned(Value::Text(v.to_string()))),
                None => Ok(ToSqlOutput::Owned(Value::Null)),
            }
        },
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let values = [
            0i128,
            1,
            -1,
            42,
            -42,
            i128::MAX,
            i128::MIN + 1,
            i128::MIN,
            1_000_000_000_000_000_000i128, // 1 ETH in wei
        ];
        for val in values {
            let encoded = encode_i128(val);
            let decoded = decode_i128(&encoded);
            assert_eq!(val, decoded, "Round-trip failed for {val}");
        }
    }

    #[test]
    fn test_memcmp_sort_order() {
        let values = [i128::MIN, -1000, -1, 0, 1, 1000, i128::MAX];
        let encoded: Vec<[u8; 16]> = values.iter().map(|v| encode_i128(*v)).collect();
        for i in 0..encoded.len() - 1 {
            assert!(
                encoded[i] < encoded[i + 1],
                "Sort order broken: {} (encoded {:?}) should be < {} (encoded {:?})",
                values[i],
                encoded[i],
                values[i + 1],
                encoded[i + 1]
            );
        }
    }

    #[test]
    fn test_zero_encoding() {
        let encoded = encode_i128(0);
        assert_eq!(
            encoded,
            [0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn test_i128_add_sql() {
        let conn = Connection::open_in_memory().unwrap();
        register_i128_functions(&conn).unwrap();

        let a = encode_i128(100);
        let b = encode_i128(250);
        let result: Vec<u8> = conn
            .query_row("SELECT i128_add(?1, ?2)", [a.as_slice(), b.as_slice()], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(decode_i128(&result), 350);
    }

    #[test]
    fn test_i128_add_with_negatives() {
        let conn = Connection::open_in_memory().unwrap();
        register_i128_functions(&conn).unwrap();

        let a = encode_i128(100);
        let b = encode_i128(-300);
        let result: Vec<u8> = conn
            .query_row("SELECT i128_add(?1, ?2)", [a.as_slice(), b.as_slice()], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(decode_i128(&result), -200);
    }

    #[test]
    fn test_i128_add_null() {
        let conn = Connection::open_in_memory().unwrap();
        register_i128_functions(&conn).unwrap();

        let a = encode_i128(100);
        let result: Option<Vec<u8>> = conn
            .query_row(
                "SELECT i128_add(?1, NULL)",
                [a.as_slice()],
                |row| row.get(0),
            )
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_sum_i128_sql() {
        let conn = Connection::open_in_memory().unwrap();
        register_i128_functions(&conn).unwrap();

        conn.execute_batch("CREATE TABLE test (val BLOB)").unwrap();
        let values = [100i128, 200, 300, -50];
        for v in values {
            conn.execute("INSERT INTO test VALUES (?1)", [encode_i128(v).as_slice()])
                .unwrap();
        }

        let result: Vec<u8> = conn
            .query_row("SELECT sum_i128(val) FROM test", [], |row| row.get(0))
            .unwrap();
        assert_eq!(decode_i128(&result), 550);
    }

    #[test]
    fn test_sum_i128_empty() {
        let conn = Connection::open_in_memory().unwrap();
        register_i128_functions(&conn).unwrap();

        conn.execute_batch("CREATE TABLE test (val BLOB)").unwrap();

        let result: Option<Vec<u8>> = conn
            .query_row("SELECT sum_i128(val) FROM test", [], |row| row.get(0))
            .unwrap();
        // Empty aggregate returns None/NULL
        assert!(result.is_none());
    }

    #[test]
    fn test_i128_to_text() {
        let conn = Connection::open_in_memory().unwrap();
        register_i128_functions(&conn).unwrap();

        let val = encode_i128(1050);
        let text: String = conn
            .query_row("SELECT i128_to_text(?1)", [val.as_slice()], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(text, "1050");
    }

    #[test]
    fn test_i128_to_text_negative() {
        let conn = Connection::open_in_memory().unwrap();
        register_i128_functions(&conn).unwrap();

        let val = encode_i128(-42);
        let text: String = conn
            .query_row("SELECT i128_to_text(?1)", [val.as_slice()], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(text, "-42");
    }

    #[test]
    fn test_i128_to_text_null() {
        let conn = Connection::open_in_memory().unwrap();
        register_i128_functions(&conn).unwrap();

        let result: Option<String> = conn
            .query_row("SELECT i128_to_text(NULL)", [], |row| row.get(0))
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_large_values() {
        let conn = Connection::open_in_memory().unwrap();
        register_i128_functions(&conn).unwrap();

        // ETH total supply in wei (~120 million * 10^18)
        let eth_supply = 120_000_000i128 * 1_000_000_000_000_000_000;
        let a = encode_i128(eth_supply);
        let b = encode_i128(eth_supply);
        let result: Vec<u8> = conn
            .query_row("SELECT i128_add(?1, ?2)", [a.as_slice(), b.as_slice()], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(decode_i128(&result), eth_supply * 2);
    }
}
