use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use clawcounting::db::connection::setup_connection;
use clawcounting::db::i128_funcs::encode_i128;
use clawcounting::db::migrations::run_migrations;

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_db_path(label: &str) -> String {
    let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!(
        "{}/clawcounting_init_test_{label}_{n}_{}.db",
        std::env::temp_dir().display(),
        std::process::id()
    )
}

fn cleanup(path: &str) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(format!("{path}-wal"));
    let _ = std::fs::remove_file(format!("{path}-shm"));
}

#[test]
fn init_creates_new_database() {
    let db_path = temp_db_path("create");
    assert!(!Path::new(&db_path).exists());

    let mut conn = setup_connection(&db_path).expect("setup_connection should succeed");
    run_migrations(&mut conn).expect("migrations should succeed");

    // DB file should now exist
    assert!(Path::new(&db_path).exists());

    // Schema should be in place — check for a known table
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='accounts'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1, "accounts table should exist after init");

    cleanup(&db_path);
}

#[test]
fn init_on_existing_db_is_idempotent() {
    let db_path = temp_db_path("idempotent");

    // First init
    let mut conn = setup_connection(&db_path).expect("first setup_connection");
    run_migrations(&mut conn).expect("first migrations");
    drop(conn);

    // Second init on same file — should not error
    let mut conn = setup_connection(&db_path).expect("second setup_connection");
    run_migrations(&mut conn).expect("second migrations should be idempotent");

    // Schema still intact
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='currencies'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);

    cleanup(&db_path);
}

#[test]
fn db_existence_check_works() {
    let db_path = temp_db_path("existence");
    assert!(!Path::new(&db_path).exists(), "file should not exist yet");

    // After init, file should exist
    let mut conn = setup_connection(&db_path).expect("setup_connection");
    run_migrations(&mut conn).expect("migrations");
    drop(conn);
    assert!(Path::new(&db_path).exists(), "file should exist after init");

    cleanup(&db_path);
}

#[test]
fn init_creates_all_expected_tables() {
    let db_path = temp_db_path("tables");

    let mut conn = setup_connection(&db_path).expect("setup_connection");
    run_migrations(&mut conn).expect("migrations");

    let expected_tables = [
        "currencies",
        "accounts",
        "account_balances",
        "financial_periods",
        "journal_entries",
        "journal_entry_lines",
        "users",
        "settings",
    ];

    for table in &expected_tables {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                [table],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "table '{table}' should exist after init");
    }

    cleanup(&db_path);
}

#[test]
fn init_registers_custom_functions() {
    let db_path = temp_db_path("functions");

    let mut conn = setup_connection(&db_path).expect("setup_connection");
    run_migrations(&mut conn).expect("migrations");

    // i128_to_text should be available — encode 100 properly (MSB-flipped)
    let encoded = encode_i128(100);
    let result: String = conn
        .query_row("SELECT i128_to_text(?1)", [encoded.to_vec()], |r| r.get(0))
        .expect("i128_to_text should work");
    assert_eq!(result, "100");

    cleanup(&db_path);
}
