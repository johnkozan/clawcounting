use crate::config::Config;
use crate::db::pool::DbPools;

#[derive(Clone)]
pub struct AppState {
    pub pools: DbPools,
    pub config: Config,
}
