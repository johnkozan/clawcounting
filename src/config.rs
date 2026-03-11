use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub db_path: String,
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

        let port = env::var("CLAWCOUNTING_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3000);

        let jwt_secret = env::var("CLAWCOUNTING_JWT_SECRET").ok();

        Config {
            db_path,
            port,
            jwt_secret,
        }
    }

    pub fn require_jwt_secret(&self) -> &str {
        self.jwt_secret
            .as_deref()
            .expect("CLAWCOUNTING_JWT_SECRET must be set for server mode. Set it in .env or as an environment variable.")
    }
}
