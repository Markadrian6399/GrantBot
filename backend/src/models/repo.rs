use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Repo {
    pub id: Uuid,
    pub owner: String,
    pub repo_name: String,
    pub webhook_secret: String,
    pub payout_amount: f64,
    pub daily_cap: f64,
    pub owner_address: String,
    pub delegation_hex: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateRepoRequest {
    /// GitHub organisation or user name
    pub owner: String,
    /// Repository name (without owner prefix)
    pub repo_name: String,
    /// USDC paid per merged PR
    pub payout_amount: f64,
    /// Maximum USDC that can be paid in a 24-hour window
    pub daily_cap: f64,
    /// Ethereum address of the repo owner (funds source)
    pub owner_address: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RepoWithStats {
    #[serde(flatten)]
    pub repo: Repo,
    pub total_paid: f64,
    pub today_spend: f64,
    pub contributor_count: i64,
}
