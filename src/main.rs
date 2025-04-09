use axum::{
    routing::{get, post},
    http::StatusCode,
    extract::State,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use utoipa::{ToSchema, OpenApi};
use utoipa_swagger_ui::SwaggerUi;
use dotenv::dotenv;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tower_http::trace::TraceLayer;
use serde_json::to_string_pretty;

#[derive(OpenApi)]
#[openapi(
    paths(root, create_user, get_users),
    components(
        schemas(CreateUser, User)
    ),
    tags(
        (name = "xai-api", description = "API for the Lungs-xAI Research Project")
    )
)]
struct ApiDoc;

type AppState = PgPool;

#[tokio::main]
async fn main() {
    // .env
    dotenv().ok();
    // initialize tracing
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "xai_api=info,sqlx=debug,tower_http=debug,axum=info".into())
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set.");
    println!("Connecting to the database ...");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            username TEXT NOT NULL
        )"
    )
        .execute(&pool)
        .await
        .expect("Failed to create users table");

    println!("✅ Database setup complete!");

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user).get(get_users))
        // Add Swagger UI
        .merge(SwaggerUi::new("/swagger-ui")
            .url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
        // Add DB pool
        .with_state(pool);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://localhost:3000");
    println!("Swagger UI available at http://localhost:3000/swagger-ui/");
    axum::serve(listener, app).await.unwrap();
}

/// Get a hello world message
#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Hello message returned successfully", body = String)
    ),
    tag = "hello-world"
)]
async fn root() -> &'static str {
    "Hello, World!"
}

/// Create a new user
#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUser,
    responses(
        (status = 201, description = "User created successfully", body = User),
        (status = 500, description = "Internal server error")
    ),
    tag = "xai"
)]
async fn create_user(
    State(pool): State<AppState>,
    Json(payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), StatusCode> {
    tracing::debug!("Trying to create user with username: {}", payload.username);

    let table_check = sqlx::query("SELECT to_regclass('public.users')")
        .fetch_one(&pool)
        .await;

    tracing::debug!("Table check result: {:?}", table_check);

    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (username) VALUES ($1) RETURNING id, username",
        payload.username
    )
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            eprintln!("Database errror: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((StatusCode::CREATED, Json(user)))
}

/// Get all users
#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = 200, description = "List all users", body = Vec<User>),
        (status = 500, description = "Internal server error")
    ),
    tag = "xai"
)]
async fn get_users(
    State(pool): State<AppState>,
) -> Result<Json<Vec<User>>, StatusCode> {
    let users = sqlx::query_as!(
        User,
        "SELECT id as \"id: i32\", username FROM users"
    )
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::info!("Users found: {}", to_string_pretty(&users).unwrap());

    Ok(Json(users))
}

// the input to our `create_user` handler
#[derive(Deserialize, ToSchema)]
struct CreateUser {
    /// The username of the user
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize, sqlx::FromRow, ToSchema)]
struct User {
    /// The user's unique identifier
    id: i32,
    /// The username of the user
    username: String,
}