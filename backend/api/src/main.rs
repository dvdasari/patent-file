mod error;
mod middleware;
mod routes;

use axum::{middleware as axum_mw, routing::{delete, get, patch, post, put}, Router};
use clap::{Parser, Subcommand};
use shared::config::AppConfig;
use shared::db::create_pool;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;

use routes::auth::AuthState;
use routes::export::ExportState;
use routes::fer::FerState;
use routes::figures::FiguresState;
use routes::generate::GenerateState;
use routes::oauth::OAuthState;
use routes::search::SearchState;

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
        /// User role: inventor (default), patent_agent, admin
        #[arg(long, default_value = "inventor")]
        role: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env from repo root (try ../.env since we run from backend/)
    for path in &[".env", "../.env"] {
        if std::path::Path::new(path).exists() {
            dotenvy::from_path(path).ok();
            break;
        }
    }

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
        role,
    }) = cli.command
    {
        return seed_user(&pool, &email, &password, &name, with_subscription, &role).await;
    }

    // Initialize AI provider
    let ai_provider: std::sync::Arc<dyn ai::LlmProvider> = std::sync::Arc::from(
        ai::create_provider(
            &config.ai_provider,
            config.anthropic_api_key.as_deref(),
        )?
    );

    // Initialize storage client
    let storage = storage::create_storage_client(
        &config.storage_backend,
        config.storage_local_path.as_deref(),
        config.port,
        config.r2_account_id.as_deref(),
        config.r2_access_key_id.as_deref(),
        config.r2_secret_access_key.as_deref(),
        config.r2_bucket_name.as_deref(),
        config.r2_public_url.as_deref(),
    )?;

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

    // OAuth routes (public — provider handles auth)
    let oauth_state = OAuthState {
        pool: pool.clone(),
        jwt_secret: config.jwt_secret.clone(),
        google_client_id: config.google_client_id.clone(),
        google_client_secret: config.google_client_secret.clone(),
        linkedin_client_id: config.linkedin_client_id.clone(),
        linkedin_client_secret: config.linkedin_client_secret.clone(),
        redirect_base_url: config.oauth_redirect_base_url.clone(),
        frontend_url: config.allowed_origin.clone(),
    };
    let oauth_routes = Router::new()
        .route(
            "/api/auth/oauth/{provider}",
            get(routes::oauth::oauth_redirect),
        )
        .route(
            "/api/auth/oauth/{provider}/callback",
            get(routes::oauth::oauth_callback),
        )
        .with_state(oauth_state);

    // Routes that need auth but NOT subscription (for pre-payment flow)
    let auth_only_routes = Router::new()
        .route("/api/me", get(routes::me::get_me))
        .route("/api/subscriptions/create", post(routes::subscriptions::create_subscription))
        .route("/api/subscriptions/status", get(routes::subscriptions::subscription_status))
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

    // Figures routes (need storage client)
    let figures_state = FiguresState {
        pool: pool.clone(),
        storage: storage.clone(),
    };
    let figures_routes = Router::new()
        .route("/api/projects/{id}/figures", post(routes::figures::upload_figure))
        .route("/api/projects/{id}/figures", get(routes::figures::list_figures))
        .route("/api/projects/{id}/figures/{figure_id}", delete(routes::figures::delete_figure))
        .layer(axum_mw::from_fn_with_state(
            pool.clone(),
            middleware::subscription::subscription_middleware,
        ))
        .layer(axum_mw::from_fn_with_state(
            jwt_secret.clone(),
            middleware::auth::auth_middleware,
        ))
        .with_state(figures_state);

    // Generate + sections routes (need AI provider)
    let generate_state = GenerateState {
        pool: pool.clone(),
        provider: ai_provider,
    };
    let generate_routes = Router::new()
        .route("/api/projects/{id}/generate", post(routes::generate::generate))
        .layer(axum_mw::from_fn_with_state(
            pool.clone(),
            middleware::rate_limit::rate_limit_generate,
        ))
        .layer(axum_mw::from_fn_with_state(
            pool.clone(),
            middleware::subscription::subscription_middleware,
        ))
        .layer(axum_mw::from_fn_with_state(
            jwt_secret.clone(),
            middleware::auth::auth_middleware,
        ))
        .with_state(generate_state);

    let sections_routes = Router::new()
        .route("/api/projects/{id}/sections/{section_type}", put(routes::sections::update_section))
        .route("/api/projects/{id}/sections/{section_type}/versions", get(routes::sections::list_versions))
        .route("/api/projects/{id}/sections/{section_type}/versions/{version_number}/restore", post(routes::sections::restore_version))
        .layer(axum_mw::from_fn_with_state(
            pool.clone(),
            middleware::subscription::subscription_middleware,
        ))
        .layer(axum_mw::from_fn_with_state(
            jwt_secret.clone(),
            middleware::auth::auth_middleware,
        ))
        .with_state(pool.clone());

    // Export routes
    let export_state = ExportState {
        pool: pool.clone(),
        storage: storage.clone(),
    };
    let export_routes = Router::new()
        .route("/api/projects/{id}/export", post(routes::export::create_export))
        .route("/api/projects/{id}/exports", get(routes::export::list_exports))
        .route("/api/exports/{id}/download", get(routes::export::get_download_url))
        .layer(axum_mw::from_fn_with_state(
            pool.clone(),
            middleware::subscription::subscription_middleware,
        ))
        .layer(axum_mw::from_fn_with_state(
            jwt_secret.clone(),
            middleware::auth::auth_middleware,
        ))
        .with_state(export_state);

    // Deadline routes
    let deadline_routes = Router::new()
        .route("/api/projects/{id}/deadlines", post(routes::deadlines::create_deadline))
        .route("/api/projects/{id}/deadlines", get(routes::deadlines::list_deadlines))
        .route("/api/projects/{id}/deadlines/{deadline_id}", patch(routes::deadlines::update_deadline))
        .route("/api/projects/{id}/deadlines/{deadline_id}", delete(routes::deadlines::delete_deadline))
        .layer(axum_mw::from_fn_with_state(
            pool.clone(),
            middleware::subscription::subscription_middleware,
        ))
        .layer(axum_mw::from_fn_with_state(
            jwt_secret.clone(),
            middleware::auth::auth_middleware,
        ))
        .with_state(pool.clone());

    // Compliance routes (auth + subscription required)
    let compliance_routes = Router::new()
        .route("/api/projects/{id}/compliance-check", post(routes::compliance::check_compliance))
        .route("/api/projects/{id}/compliance-checks", get(routes::compliance::list_compliance_checks))
        .layer(axum_mw::from_fn_with_state(
            pool.clone(),
            middleware::subscription::subscription_middleware,
        ))
        .layer(axum_mw::from_fn_with_state(
            jwt_secret.clone(),
            middleware::auth::auth_middleware,
        ))
        .with_state(pool.clone());

    // Case law search (auth required, no subscription needed)
    let case_law_routes = Router::new()
        .route("/api/case-law/search", get(routes::compliance::search_case_law))
        .layer(axum_mw::from_fn_with_state(
            jwt_secret.clone(),
            middleware::auth::auth_middleware,
        ));

    // Search routes (need AI provider + storage)
    let search_engine = std::sync::Arc::new(search::SearchEngine::new(ai_provider.clone()));
    let search_state = SearchState {
        pool: pool.clone(),
        engine: search_engine,
        storage: storage.clone(),
    };
    let search_routes = Router::new()
        .route("/api/search", post(routes::search::create_search))
        .route("/api/searches", get(routes::search::list_searches))
        .route("/api/searches/{id}", get(routes::search::get_search))
        .route("/api/searches/{id}/report", post(routes::search::generate_report))
        .route("/api/search-reports/{id}/download", get(routes::search::download_report))
        .layer(axum_mw::from_fn_with_state(
            pool.clone(),
            middleware::subscription::subscription_middleware,
        ))
        .layer(axum_mw::from_fn_with_state(
            jwt_secret.clone(),
            middleware::auth::auth_middleware,
        ))
        .with_state(search_state);

    // FER routes (need AI provider)
    let fer_state = FerState {
        pool: pool.clone(),
        provider: ai_provider.clone(),
    };
    let fer_routes = Router::new()
        .route("/api/fer", post(routes::fer::create_fer))
        .route("/api/fer", get(routes::fer::list_fer))
        .route("/api/fer/{id}", get(routes::fer::get_fer))
        .route("/api/fer/{id}/generate", post(routes::fer::generate_responses))
        .route("/api/fer/responses/{id}", patch(routes::fer::update_response))
        .route("/api/fer/responses/{id}/accept", post(routes::fer::accept_response))
        .route("/api/fer/responses/{id}/regenerate", post(routes::fer::regenerate_response))
        .layer(axum_mw::from_fn_with_state(
            pool.clone(),
            middleware::subscription::subscription_middleware,
        ))
        .layer(axum_mw::from_fn_with_state(
            jwt_secret.clone(),
            middleware::auth::auth_middleware,
        ))
        .with_state(fer_state);

    // Admin routes (auth + role + admin required)
    let admin_routes = Router::new()
        .route("/api/admin/users", get(routes::admin::list_users))
        .route("/api/admin/users/role", patch(routes::admin::update_user_role))
        .layer(axum_mw::from_fn(middleware::role::require_admin))
        .layer(axum_mw::from_fn_with_state(
            pool.clone(),
            middleware::role::role_middleware,
        ))
        .layer(axum_mw::from_fn_with_state(
            jwt_secret.clone(),
            middleware::auth::auth_middleware,
        ))
        .with_state(pool.clone());

    // Webhook routes (no auth — signature verified internally)
    let webhook_routes = Router::new()
        .route("/api/webhooks/razorpay", post(routes::webhooks::razorpay_webhook))
        .with_state(pool.clone());

    // Static file serving for local storage (dev only)
    let static_routes = if config.storage_backend == "local" {
        let path = config.storage_local_path.as_deref().unwrap_or("./storage");
        Some(Router::new().nest_service("/files", ServeDir::new(path)))
    } else {
        None
    };

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

    let mut app = Router::new()
        .merge(public_routes)
        .merge(oauth_routes)
        .merge(auth_only_routes)
        .merge(protected_routes)
        .merge(figures_routes)
        .merge(generate_routes)
        .merge(sections_routes)
        .merge(export_routes)
        .merge(deadline_routes)
        .merge(compliance_routes)
        .merge(case_law_routes)
        .merge(search_routes)
        .merge(fer_routes)
        .merge(admin_routes)
        .merge(webhook_routes);

    if let Some(static_rt) = static_routes {
        app = app.merge(static_rt);
    }

    let app = app.layer(cors);

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
    role: &str,
) -> anyhow::Result<()> {
    let valid_roles = ["inventor", "patent_agent", "admin"];
    if !valid_roles.contains(&role) {
        anyhow::bail!("Invalid role '{}'. Must be one of: {}", role, valid_roles.join(", "));
    }
    use argon2::password_hash::{rand_core::OsRng, SaltString};
    use argon2::{Argon2, PasswordHasher};

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
        .to_string();

    // Safe: role is validated against allowlist above
    let role_cast = format!("'{}'::user_role", role);
    let query = format!(
        "INSERT INTO users (email, password_hash, full_name, role) VALUES ($1, $2, $3, {})
         ON CONFLICT (email) DO UPDATE SET password_hash = EXCLUDED.password_hash, full_name = EXCLUDED.full_name, role = {}
         RETURNING id",
        role_cast, role_cast
    );
    let user_id: uuid::Uuid = sqlx::query_scalar(&query)
        .bind(email)
        .bind(&hash)
        .bind(name)
        .fetch_one(pool)
        .await?;

    println!("Upserted user: {} ({})", email, user_id);

    if with_subscription {
        let period_end = chrono::Utc::now() + chrono::Duration::days(365);
        let sub_id = format!("sub_test_seed_{}", user_id);
        sqlx::query(
            "INSERT INTO subscriptions (user_id, razorpay_customer_id, razorpay_subscription_id, plan_id, status, current_period_start, current_period_end)
             VALUES ($1, $2, $3, $4, 'active', now(), $5)
             ON CONFLICT (razorpay_subscription_id) DO UPDATE SET
                status = 'active', current_period_end = EXCLUDED.current_period_end"
        )
        .bind(user_id)
        .bind("cust_test_seed")
        .bind(&sub_id)
        .bind("plan_test_seed")
        .bind(period_end)
        .execute(pool)
        .await?;

        println!("Upserted active subscription (expires: {})", period_end.format("%Y-%m-%d"));
    }

    Ok(())
}
