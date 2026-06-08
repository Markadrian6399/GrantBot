use crate::{
    errors::AppError,
    models::repo::{CreateRepoRequest, Repo, RepoWithStats},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use rand::Rng;
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

pub type AppState = Arc<PgPool>;

/// Register a new GitHub repository for automated USDC payouts.
#[utoipa::path(
    post,
    path = "/repos",
    request_body = CreateRepoRequest,
    responses(
        (status = 201, description = "Repo registered. Returns repo object + webhook URL + secret.", body = RepoWithStats),
        (status = 400, description = "Invalid request body"),
        (status = 500, description = "Database error")
    ),
    tag = "Repos"
)]
pub async fn create_repo(
    State(pool): State<AppState>,
    Json(req): Json<CreateRepoRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let webhook_secret: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO repos (owner, repo_name, webhook_secret, payout_amount, daily_cap, owner_address) \
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
    )
    .bind(&req.owner)
    .bind(&req.repo_name)
    .bind(&webhook_secret)
    .bind(req.payout_amount)
    .bind(req.daily_cap)
    .bind(&req.owner_address)
    .fetch_one(pool.as_ref())
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "repo": {
                "id": id,
                "owner": req.owner,
                "repo_name": req.repo_name,
                "payout_amount": req.payout_amount,
                "daily_cap": req.daily_cap,
                "owner_address": req.owner_address,
                "webhook_secret": webhook_secret,
                "delegation_hex": null
            },
            "webhook_url": "/webhook/github",
            "webhook_secret": webhook_secret
        })),
    ))
}

/// Fetch a repo by ID with live payment stats.
#[utoipa::path(
    get,
    path = "/repos/{id}",
    params(
        ("id" = Uuid, Path, description = "Repository UUID")
    ),
    responses(
        (status = 200, description = "Repo with stats", body = RepoWithStats),
        (status = 404, description = "Repo not found")
    ),
    tag = "Repos"
)]
pub async fn get_repo(
    State(pool): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RepoWithStats>, AppError> {
    let repo = sqlx::query_as::<_, Repo>(
        "SELECT id, owner, repo_name, webhook_secret, payout_amount, daily_cap, \
         owner_address, delegation_hex, created_at FROM repos WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool.as_ref())
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Repo {} not found", id)))?;

    let total_paid: f64 = sqlx::query_scalar::<_, f64>(
        "SELECT COALESCE(SUM(p.amount), 0.0) FROM payments p \
         JOIN pr_events pe ON p.pr_event_id = pe.id WHERE pe.repo_id = $1",
    )
    .bind(id)
    .fetch_one(pool.as_ref())
    .await
    .unwrap_or(0.0);

    let today_spend: f64 = sqlx::query_scalar::<_, f64>(
        "SELECT COALESCE(SUM(p.amount), 0.0) FROM payments p \
         JOIN pr_events pe ON p.pr_event_id = pe.id \
         WHERE pe.repo_id = $1 AND p.created_at > NOW() - INTERVAL '1 day'",
    )
    .bind(id)
    .fetch_one(pool.as_ref())
    .await
    .unwrap_or(0.0);

    let contributor_count: i64 =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM contributors WHERE repo_id = $1")
            .bind(id)
            .fetch_one(pool.as_ref())
            .await
            .unwrap_or(0);

    Ok(Json(RepoWithStats {
        repo,
        total_paid,
        today_spend,
        contributor_count,
    }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateDelegationRequest {
    /// ERC-7715 delegation hex string obtained from MetaMask delegation toolkit
    pub delegation_hex: String,
}

/// Store an ERC-7715 spending delegation for a repo.
#[utoipa::path(
    put,
    path = "/repos/{id}/delegation",
    params(
        ("id" = Uuid, Path, description = "Repository UUID")
    ),
    request_body = UpdateDelegationRequest,
    responses(
        (status = 200, description = "Delegation stored"),
        (status = 404, description = "Repo not found")
    ),
    tag = "Repos"
)]
pub async fn update_delegation(
    State(pool): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateDelegationRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query("UPDATE repos SET delegation_hex = $1 WHERE id = $2")
        .bind(&req.delegation_hex)
        .bind(id)
        .execute(pool.as_ref())
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound(format!("Repo {} not found", id)));
    }

    Ok(Json(json!({ "success": true })))
}
