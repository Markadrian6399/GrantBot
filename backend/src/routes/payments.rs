use crate::{
    errors::AppError,
    models::payment::{PaymentStats, PaymentWithPr},
    routes::repos::AppState,
};
use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct RepoQuery {
    pub repo_id: Uuid,
}

/// List all payment records for a repository (joined with PR event data).
#[utoipa::path(
    get,
    path = "/payments",
    params(
        ("repo_id" = Uuid, Query, description = "Repository UUID")
    ),
    responses(
        (status = 200, description = "List of payments with PR details", body = Vec<PaymentWithPr>)
    ),
    tag = "Payments"
)]
pub async fn list_payments(
    State(pool): State<AppState>,
    Query(q): Query<RepoQuery>,
) -> Result<Json<Vec<PaymentWithPr>>, AppError> {
    let payments = sqlx::query_as::<_, PaymentWithPr>(
        "SELECT p.id, p.pr_event_id, p.amount, p.tx_hash, p.venice_reason, p.created_at, \
                pe.pr_number, pe.pr_title, pe.contributor, pe.status, pe.merged_at, pe.repo_id \
         FROM payments p \
         JOIN pr_events pe ON p.pr_event_id = pe.id \
         WHERE pe.repo_id = $1 \
         ORDER BY p.created_at DESC",
    )
    .bind(q.repo_id)
    .fetch_all(pool.as_ref())
    .await?;

    Ok(Json(payments))
}

/// Aggregate payment statistics for a repository.
#[utoipa::path(
    get,
    path = "/payments/stats",
    params(
        ("repo_id" = Uuid, Query, description = "Repository UUID")
    ),
    responses(
        (status = 200, description = "Payment statistics", body = PaymentStats)
    ),
    tag = "Payments"
)]
pub async fn get_stats(
    State(pool): State<AppState>,
    Query(q): Query<RepoQuery>,
) -> Result<Json<PaymentStats>, AppError> {
    let total_paid: f64 = sqlx::query_scalar::<_, f64>(
        "SELECT COALESCE(SUM(p.amount), 0.0) FROM payments p \
         JOIN pr_events pe ON p.pr_event_id = pe.id WHERE pe.repo_id = $1",
    )
    .bind(q.repo_id)
    .fetch_one(pool.as_ref())
    .await
    .unwrap_or(0.0);

    let pr_count: i64 =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM pr_events WHERE repo_id = $1")
            .bind(q.repo_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);

    let today_spend: f64 = sqlx::query_scalar::<_, f64>(
        "SELECT COALESCE(SUM(p.amount), 0.0) FROM payments p \
         JOIN pr_events pe ON p.pr_event_id = pe.id \
         WHERE pe.repo_id = $1 AND p.created_at > NOW() - INTERVAL '1 day'",
    )
    .bind(q.repo_id)
    .fetch_one(pool.as_ref())
    .await
    .unwrap_or(0.0);

    let contributor_count: i64 =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM contributors WHERE repo_id = $1")
            .bind(q.repo_id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);

    Ok(Json(PaymentStats {
        total_paid,
        pr_count,
        today_spend,
        contributor_count,
    }))
}
