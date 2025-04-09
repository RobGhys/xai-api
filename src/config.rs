#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub server_addr: String,
    pub log_level: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set."),
            server_addr: std::env::var("SERVER_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:3000".to_string()),
            log_level: std::env::var("LOG_LEVEL").unwrap_or_else(|_| {
                "xai_api=info,sqlx=debug,tower_http=debug,axum=info".to_string()
            }),
        }
    }
}
