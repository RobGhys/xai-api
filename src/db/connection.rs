use crate::config::Config;
use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn establish_connection(config: &Config) -> PgPool {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to PostgreSQL")
}
