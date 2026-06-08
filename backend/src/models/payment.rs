use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Payment {
    pub id: Uuid,
    pub pr_event_id: Uuid,
    pub amount: f64,
    pub tx_hash: Option<String>,
    pub venice_reason: String,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct PaymentWithPr {
    pub id: Uuid,
    pub pr_event_id: Uuid,
    pub amount: f64,
    pub tx_hash: Option<String>,
    pub venice_reason: String,
    pub created_at: Option<DateTime<Utc>>,
    pub pr_number: i32,
    pub pr_title: String,
    pub contributor: String,
    pub status: String,
    pub merged_at: DateTime<Utc>,
    pub repo_id: Uuid,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaymentStats {
    /// Total USDC paid across all time
    pub total_paid: f64,
    /// Total number of PRs processed
    pub pr_count: i64,
    /// USDC paid in the last 24 hours
    pub today_spend: f64,
    /// Number of registered contributors
    pub contributor_count: i64,
}
