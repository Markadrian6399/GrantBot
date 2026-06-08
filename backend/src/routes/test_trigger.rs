use crate::{
    errors::AppError,
    models::payment::Payment,
    routes::repos::AppState,
    services::payment_processor,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub struct TestTriggerRequest {
    /// Repository UUID (must already exist in the database)
    pub repo_id: Uuid,
    /// PR number shown in the activity feed
    pub pr_number: i32,
    /// PR title sent to Venice AI for evaluation
    pub pr_title: String,
    /// GitHub username — must be a registered contributor to receive payment
    pub contributor: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PrStatusResponse {
    /// Current status: pending | approved | rejected | paid | failed
    pub status: String,
    /// Payment record if one was created (approved or rejected by Venice)
    pub payment: Option<Payment>,
}

/// Simulate a merged PR without a real GitHub webhook. Used for testing the full payment flow.
#[utoipa::path(
    post,
    path = "/test/trigger-pr",
    request_body = TestTriggerRequest,
    responses(
        (status = 200, description = "PR event created and processor spawned", body = inline(PrEventIdResponse)),
        (status = 500, description = "Database error")
    ),
    tag = "Test"
)]
pub async fn trigger_pr(
    State(pool): State<AppState>,
    Json(req): Json<TestTriggerRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let pr_event_id: Uuid = sqlx::query_scalar(
        "INSERT INTO pr_events (pr_number, pr_title, contributor, repo_id, merged_at, status) \
         VALUES ($1, $2, $3, $4, $5, 'pending') \
         RETURNING id",
    )
    .bind(req.pr_number)
    .bind(&req.pr_title)
    .bind(&req.contributor)
    .bind(req.repo_id)
    .bind(Utc::now())
    .fetch_one(pool.as_ref())
    .await?;

    let pool_clone = Arc::clone(&pool);
    tokio::spawn(async move {
        if let Err(e) = payment_processor::process_pr_event(pool_clone, pr_event_id).await {
            tracing::error!("payment_processor error: {:?}", e);
        }
    });

    Ok((StatusCode::OK, Json(json!({ "pr_event_id": pr_event_id }))))
}

/// Poll the status of a PR event triggered via /test/trigger-pr.
#[utoipa::path(
    get,
    path = "/test/pr-status/{pr_event_id}",
    params(
        ("pr_event_id" = Uuid, Path, description = "PR event UUID returned by /test/trigger-pr")
    ),
    responses(
        (status = 200, description = "PR status and optional payment record", body = PrStatusResponse),
        (status = 404, description = "PR event not found")
    ),
    tag = "Test"
)]
pub async fn get_pr_status(
    State(pool): State<AppState>,
    Path(pr_event_id): Path<Uuid>,
) -> Result<Json<PrStatusResponse>, AppError> {
    let status: Option<String> =
        sqlx::query_scalar("SELECT status FROM pr_events WHERE id = $1")
            .bind(pr_event_id)
            .fetch_optional(pool.as_ref())
            .await?;

    let status = status
        .ok_or_else(|| AppError::NotFound(format!("PR event {} not found", pr_event_id)))?;

    let payment = sqlx::query_as::<_, Payment>(
        "SELECT id, pr_event_id, amount, tx_hash, venice_reason, created_at \
         FROM payments WHERE pr_event_id = $1",
    )
    .bind(pr_event_id)
    .fetch_optional(pool.as_ref())
    .await?;

    Ok(Json(PrStatusResponse { status, payment }))
}

// Inline schema used only in the utoipa doc — not a real type.
#[derive(ToSchema)]
#[allow(dead_code)]
struct PrEventIdResponse {
    pr_event_id: Uuid,
}
