use rusqlite::{Connection, Result};

use super::i128_funcs::register_i128_functions;

/// Open and configure a SQLite connection with all required pragmas and custom functions.
pub fn setup_connection(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    configure_connection(&conn)?;
    Ok(conn)
}

/// Configure an already-open connection with pragmas and custom functions.
/// Used by both direct connection setup and pool post_create hooks.
pub fn configure_connection(conn: &Connection) -> Result<()> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "busy_timeout", 5000)?;
    register_i128_functions(conn)?;
    Ok(())
}
