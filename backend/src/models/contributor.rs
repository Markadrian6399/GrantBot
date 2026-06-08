use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Contributor {
    pub id: Uuid,
    pub github_username: String,
    pub wallet_address: String,
    pub repo_id: Uuid,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateContributorRequest {
    /// GitHub username of the contributor
    pub github_username: String,
    /// Ethereum wallet address that receives USDC payouts
    pub wallet_address: String,
}
