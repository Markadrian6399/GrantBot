use crate::{
    routes::repos::AppState,
    services::{github, payment_processor},
};
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use chrono::Utc;
use std::sync::Arc;

/// GitHub webhook receiver. Handles `pull_request` events and triggers
/// autonomous USDC payment processing for merged PRs.
///
/// Set this as the Payload URL in your GitHub repo webhook settings.
/// Content-Type must be `application/json` and the secret must match
/// the `webhook_secret` returned when the repo was registered.
#[utoipa::path(
    post,
    path = "/webhook/github",
    request_body(
        content = String,
        description = "Raw GitHub webhook JSON payload",
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Event received and processed"),
        (status = 400, description = "Malformed payload"),
        (status = 401, description = "Invalid HMAC signature"),
        (status = 500, description = "Internal error inserting PR event")
    ),
    tag = "Webhooks"
)]
pub async fn handle_github_webhook(
    State(pool): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    let signature = match headers
        .get("X-Hub-Signature-256")
        .and_then(|v| v.to_str().ok())
    {
        Some(s) => s.to_string(),
        None => {
            tracing::warn!("Missing X-Hub-Signature-256 header");
            return StatusCode::UNAUTHORIZED;
        }
    };

    let payload: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to parse webhook payload: {:?}", e);
            return StatusCode::BAD_REQUEST;
        }
    };

    let owner = payload["repository"]["owner"]["login"].as_str().unwrap_or("");
    let repo_name = payload["repository"]["name"].as_str().unwrap_or("");

    if owner.is_empty() || repo_name.is_empty() {
        tracing::warn!("Missing repository info in webhook payload");
        return StatusCode::BAD_REQUEST;
    }

    let repo = match sqlx::query_as::<_, (uuid::Uuid, String)>(
        "SELECT id, webhook_secret FROM repos WHERE owner = $1 AND repo_name = $2",
    )
    .bind(owner)
    .bind(repo_name)
    .fetch_optional(pool.as_ref())
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => {
            tracing::warn!("No repo found for {}/{}", owner, repo_name);
            return StatusCode::OK;
        }
        Err(e) => {
            tracing::error!("DB error looking up repo: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    if !github::verify_webhook_signature(&repo.1, &body, &signature) {
        tracing::warn!("Invalid webhook signature for {}/{}", owner, repo_name);
        return StatusCode::UNAUTHORIZED;
    }

    let action = payload["action"].as_str().unwrap_or("");
    let merged = payload["pull_request"]["merged"].as_bool().unwrap_or(false);

    if action == "closed" && merged {
        let pr_number = payload["pull_request"]["number"].as_i64().unwrap_or(0) as i32;
        let pr_title = payload["pull_request"]["title"]
            .as_str()
            .unwrap_or("Untitled PR")
            .to_string();
        let contributor = payload["pull_request"]["user"]["login"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        let merged_at = payload["pull_request"]["merged_at"]
            .as_str()
            .and_then(|s| s.parse::<chrono::DateTime<Utc>>().ok())
            .unwrap_or_else(Utc::now);

        let repo_id = repo.0;

        let pr_event_id: uuid::Uuid = match sqlx::query_scalar(
            "INSERT INTO pr_events (pr_number, pr_title, contributor, repo_id, merged_at, status) \
             VALUES ($1, $2, $3, $4, $5, 'pending') RETURNING id",
        )
        .bind(pr_number)
        .bind(&pr_title)
        .bind(&contributor)
        .bind(repo_id)
        .bind(merged_at)
        .fetch_one(pool.as_ref())
        .await
        {
            Ok(id) => id,
            Err(e) => {
                tracing::error!("Failed to insert pr_event: {:?}", e);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        };

        let pool_clone = Arc::clone(&pool);
        tokio::spawn(async move {
            if let Err(e) =
                payment_processor::process_pr_event(pool_clone, pr_event_id).await
            {
                tracing::error!(
                    "payment_processor error for pr_event {}: {:?}",
                    pr_event_id,
                    e
                );
            }
        });

        tracing::info!(
            "Webhook: merged PR #{} '{}' by {} in {}/{}",
            pr_number,
            pr_title,
            contributor,
            owner,
            repo_name
        );
    }

    StatusCode::OK
}
