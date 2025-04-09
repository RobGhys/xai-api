use sqlx::PgPool;

pub async fn run_migrations(pool: &PgPool) {
    tracing::info!("Running database migrations...");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            username TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await
    .expect("Failed to create users table");

    tracing::info!("✅ Database setup complete!");
}
