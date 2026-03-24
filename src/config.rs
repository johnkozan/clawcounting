use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub db_path: String,
    pub host: String,
    pub port: u16,
    pub jwt_secret: Option<String>,
}

impl Config {
    pub fn from_env(db_override: Option<&str>) -> Self {
        let _ = dotenvy::dotenv();

        let db_path = db_override
            .map(String::from)
            .or_else(|| env::var("CLAWCOUNTING_DB").ok())
            .unwrap_or_else(|| "./clawcounting.db".to_string());

        let host = env::var("CLAWCOUNTING_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        let port = env::var("CLAWCOUNTING_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3000);

        let jwt_secret = env::var("CLAWCOUNTING_JWT_SECRET").ok();

        Config {
            db_path,
            host,
            port,
            jwt_secret,
        }
    }
}
