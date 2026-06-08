mod config;
mod db;
mod errors;
mod models;
mod openapi;
mod routes;
mod services;

use axum::{
    routing::{get, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let cfg = config::Config::from_env()?;
    tracing::info!("Starting GrantBot on port {}", cfg.port);

    let pool = db::create_pool(&cfg.database_url).await?;
    tracing::info!("Connected to PostgreSQL");

    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Migrations applied");

    let shared_pool = Arc::new(pool);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // Swagger UI
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-doc/openapi.json", openapi::ApiDoc::openapi()),
        )
        // Webhook
        .route(
            "/webhook/github",
            post(routes::webhook::handle_github_webhook),
        )
        // Repos
        .route("/repos", post(routes::repos::create_repo))
        .route("/repos/:id", get(routes::repos::get_repo))
        .route(
            "/repos/:id/delegation",
            put(routes::repos::update_delegation),
        )
        // Contributors
        .route(
            "/repos/:repo_id/contributors",
            post(routes::contributors::add_contributor)
                .get(routes::contributors::list_contributors),
        )
        // Payments
        .route("/payments", get(routes::payments::list_payments))
        .route("/payments/stats", get(routes::payments::get_stats))
        // Test endpoints
        .route("/test/trigger-pr", post(routes::test_trigger::trigger_pr))
        .route(
            "/test/pr-status/:pr_event_id",
            get(routes::test_trigger::get_pr_status),
        )
        .layer(cors)
        .with_state(shared_pool);

    let addr = format!("0.0.0.0:{}", cfg.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {}", addr);
    tracing::info!(
        "Swagger UI available at http://localhost:{}/swagger-ui/",
        cfg.port
    );

    axum::serve(listener, app).await?;
    Ok(())
}
