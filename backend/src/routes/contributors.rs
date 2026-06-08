use crate::{
    errors::AppError,
    models::contributor::{Contributor, CreateContributorRequest},
    routes::repos::AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

/// Register a contributor wallet address for a repository.
#[utoipa::path(
    post,
    path = "/repos/{repo_id}/contributors",
    params(
        ("repo_id" = Uuid, Path, description = "Repository UUID")
    ),
    request_body = CreateContributorRequest,
    responses(
        (status = 201, description = "Contributor added", body = Contributor),
        (status = 404, description = "Repo not found")
    ),
    tag = "Contributors"
)]
pub async fn add_contributor(
    State(pool): State<AppState>,
    Path(repo_id): Path<Uuid>,
    Json(req): Json<CreateContributorRequest>,
) -> Result<(StatusCode, Json<Contributor>), AppError> {
    let exists: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM repos WHERE id = $1")
        .bind(repo_id)
        .fetch_one(pool.as_ref())
        .await
        .unwrap_or(0);

    if exists == 0 {
        return Err(AppError::NotFound(format!("Repo {} not found", repo_id)));
    }

    let contributor = sqlx::query_as::<_, Contributor>(
        "INSERT INTO contributors (github_username, wallet_address, repo_id) \
         VALUES ($1, $2, $3) \
         RETURNING id, github_username, wallet_address, repo_id",
    )
    .bind(&req.github_username)
    .bind(&req.wallet_address)
    .bind(repo_id)
    .fetch_one(pool.as_ref())
    .await?;

    Ok((StatusCode::CREATED, Json(contributor)))
}

/// List all registered contributors for a repository.
#[utoipa::path(
    get,
    path = "/repos/{repo_id}/contributors",
    params(
        ("repo_id" = Uuid, Path, description = "Repository UUID")
    ),
    responses(
        (status = 200, description = "List of contributors", body = Vec<Contributor>)
    ),
    tag = "Contributors"
)]
pub async fn list_contributors(
    State(pool): State<AppState>,
    Path(repo_id): Path<Uuid>,
) -> Result<Json<Vec<Contributor>>, AppError> {
    let contributors = sqlx::query_as::<_, Contributor>(
        "SELECT id, github_username, wallet_address, repo_id \
         FROM contributors WHERE repo_id = $1 ORDER BY github_username",
    )
    .bind(repo_id)
    .fetch_all(pool.as_ref())
    .await?;

    Ok(Json(contributors))
}
