use anyhow::Context;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Config {
    pub github_webhook_secret: String,
    pub venice_ai_api_key: String,
    pub oneshot_api_key: String,
    pub database_url: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Config {
            github_webhook_secret: std::env::var("GITHUB_WEBHOOK_SECRET")
                .context("GITHUB_WEBHOOK_SECRET must be set")?,
            venice_ai_api_key: std::env::var("VENICE_AI_API_KEY")
                .context("VENICE_AI_API_KEY must be set")?,
            oneshot_api_key: std::env::var("ONESHOT_API_KEY")
                .context("ONESHOT_API_KEY must be set")?,
            database_url: std::env::var("DATABASE_URL")
                .context("DATABASE_URL must be set")?,
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3001".to_string())
                .parse()
                .context("PORT must be a valid number")?,
        })
    }
}
