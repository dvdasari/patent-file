mod error;
mod middleware;
mod routes;

use axum::{middleware as axum_mw, routing::{delete, get, patch, post, put}, Router};
use clap::{Parser, Subcommand};
use shared::config::AppConfig;
use shared::db::create_pool;
use tower_http::cors::CorsLayer;
use tracing_subscriber::EnvFilter;

use routes::auth::AuthState;

#[derive(Parser)]
#[command(name = "patent-draft-pro-api")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a test user for local development
    SeedUser {
        #[arg(long)]
        email: String,
        #[arg(long)]
        password: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        with_subscription: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = AppConfig::from_env()?;
    let pool = create_pool(&config.database_url).await?;

    let cli = Cli::parse();

    if let Some(Commands::SeedUser {
        email,
        password,
        name,
        with_subscription,
    }) = cli.command
    {
        return seed_user(&pool, &email, &password, &name, with_subscription).await;
    }

    let jwt_secret = config.jwt_secret.clone();

    let auth_state = AuthState {
        pool: pool.clone(),
        jwt_secret: config.jwt_secret.clone(),
    };

    // Routes that don't need auth
    let public_routes = Router::new()
        .route("/api/health", get(routes::health::health))
        .route("/api/auth/login", post(routes::auth::login))
        .route("/api/auth/refresh", post(routes::auth::refresh))
        .route("/api/auth/logout", post(routes::auth::logout))
        .with_state(auth_state);

    // Routes that need auth but NOT subscription (for pre-payment flow)
    let auth_only_routes = Router::new()
        .route("/api/me", get(routes::me::get_me))
        .layer(axum_mw::from_fn_with_state(
            jwt_secret.clone(),
            middleware::auth::auth_middleware,
        ))
        .with_state(pool.clone());

    // Routes that need auth + active subscription
    let protected_routes = Router::new()
        .route("/api/projects", get(routes::projects::list_projects))
        .route("/api/projects", post(routes::projects::create_project))
        .route("/api/projects/{id}", get(routes::projects::get_project))
        .route("/api/projects/{id}", patch(routes::projects::update_project))
        .route("/api/projects/{id}", delete(routes::projects::delete_project))
        .route("/api/projects/{id}/applicant", put(routes::applicants::upsert_applicant))
        .route("/api/projects/{id}/applicant", get(routes::applicants::get_applicant))
        .route("/api/projects/{id}/interview", put(routes::interview::save_interview))
        .route("/api/projects/{id}/interview", get(routes::interview::get_interview))
        .layer(axum_mw::from_fn_with_state(
            pool.clone(),
            middleware::subscription::subscription_middleware,
        ))
        .layer(axum_mw::from_fn_with_state(
            jwt_secret.clone(),
            middleware::auth::auth_middleware,
        ))
        .with_state(pool.clone());

    let cors = CorsLayer::new()
        .allow_origin(
            config
                .allowed_origin
                .parse::<http::HeaderValue>()
                .expect("valid ALLOWED_ORIGIN"),
        )
        .allow_methods([
            http::Method::GET,
            http::Method::POST,
            http::Method::PUT,
            http::Method::PATCH,
            http::Method::DELETE,
        ])
        .allow_headers([http::header::CONTENT_TYPE, http::header::AUTHORIZATION])
        .allow_credentials(true);

    let app = Router::new()
        .merge(public_routes)
        .merge(auth_only_routes)
        .merge(protected_routes)
        .layer(cors);

    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("Starting server on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn seed_user(
    pool: &sqlx::PgPool,
    email: &str,
    password: &str,
    name: &str,
    with_subscription: bool,
) -> anyhow::Result<()> {
    use argon2::password_hash::{rand_core::OsRng, SaltString};
    use argon2::{Argon2, PasswordHasher};

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
        .to_string();

    let user_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO users (email, password_hash, full_name) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(email)
    .bind(&hash)
    .bind(name)
    .fetch_one(pool)
    .await?;

    println!("Created user: {} ({})", email, user_id);

    if with_subscription {
        let period_end = chrono::Utc::now() + chrono::Duration::days(365);
        sqlx::query(
            "INSERT INTO subscriptions (user_id, razorpay_customer_id, razorpay_subscription_id, plan_id, status, current_period_start, current_period_end)
             VALUES ($1, $2, $3, $4, 'active', now(), $5)"
        )
        .bind(user_id)
        .bind("cust_test_seed")
        .bind(format!("sub_test_seed_{}", user_id))
        .bind("plan_test_seed")
        .bind(period_end)
        .execute(pool)
        .await?;

        println!("Created active subscription (expires: {})", period_end.format("%Y-%m-%d"));
    }

    Ok(())
}
