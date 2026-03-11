use deadpool_sqlite::{Config, Pool, Runtime};
use rusqlite::Connection;

use super::connection::configure_connection;

fn post_create(conn: &Connection) {
    configure_connection(conn).expect("Failed to configure pooled connection");
}

#[derive(Clone)]
pub struct DbPools {
    pub write: Pool,
    pub read: Pool,
}

impl DbPools {
    pub fn new(db_path: &str, read_pool_size: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let write = create_pool(db_path, 1)?;
        let read = create_pool(db_path, read_pool_size)?;
        Ok(DbPools { write, read })
    }
}

fn create_pool(db_path: &str, size: usize) -> Result<Pool, Box<dyn std::error::Error>> {
    let cfg = Config::new(db_path);
    let pool = cfg.builder(Runtime::Tokio1)?
        .max_size(size)
        .post_create(deadpool_sqlite::Hook::async_fn(move |conn, _| {
            Box::pin(async move {
                conn.interact(|conn| post_create(conn))
                    .await
                    .map_err(|e| deadpool_sqlite::HookError::Message(e.to_string().into()))?;
                Ok(())
            })
        }))
        .build()?;
    Ok(pool)
}
