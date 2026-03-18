use rusqlite::Connection;

use crate::config::Config;
use crate::db::pool::DbPools;
use crate::error::AppError;

#[derive(Clone)]
pub struct AppState {
    pub pools: DbPools,
    pub config: Config,
}

impl AppState {
    pub async fn with_write<F, T>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&Connection) -> Result<T, AppError> + Send + 'static,
        T: Send + 'static,
    {
        let conn = self
            .pools
            .write
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        conn.interact(move |conn| f(conn))
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
    }

    pub async fn with_write_mut<F, T>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&mut Connection) -> Result<T, AppError> + Send + 'static,
        T: Send + 'static,
    {
        let conn = self
            .pools
            .write
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        conn.interact(move |conn| f(conn))
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
    }

    pub async fn with_read<F, T>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&Connection) -> Result<T, AppError> + Send + 'static,
        T: Send + 'static,
    {
        let conn = self
            .pools
            .read
            .get()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        conn.interact(move |conn| f(conn))
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
    }
}
