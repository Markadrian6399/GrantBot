use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
#[allow(dead_code)]
pub struct PrEvent {
    pub id: Uuid,
    pub pr_number: i32,
    pub pr_title: String,
    pub contributor: String,
    pub repo_id: Uuid,
    pub merged_at: DateTime<Utc>,
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
}

#[allow(dead_code)]
pub enum PrStatus {
    Pending,
    Approved,
    Rejected,
    Paid,
    Failed,
}

impl PrStatus {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            PrStatus::Pending => "pending",
            PrStatus::Approved => "approved",
            PrStatus::Rejected => "rejected",
            PrStatus::Paid => "paid",
            PrStatus::Failed => "failed",
        }
    }
}
