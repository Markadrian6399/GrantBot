use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
pub struct VeniceDecision {
    pub approved: bool,
    pub amount: f64,
    pub reason: String,
}

pub async fn evaluate_pr(
    pr_title: &str,
    contributor: &str,
    payout_amount: f64,
    daily_cap: f64,
    daily_spent: f64,
) -> anyhow::Result<VeniceDecision> {
    let api_key = std::env::var("VENICE_AI_API_KEY")
        .unwrap_or_else(|_| "missing".to_string());

    let system_prompt = r#"You are GrantBot, an autonomous USDC payment agent. Given a merged PR and rules, decide if contributor should be paid. Respond ONLY in valid JSON with no markdown: { "approved": bool, "amount": number, "reason": string }"#;

    let user_content = format!(
        "PR Title: {}\nContributor: {}\nDefault payout: {} USDC\nDaily cap: {} USDC\nAlready spent today: {} USDC\nShould this contributor be paid?",
        pr_title, contributor, payout_amount, daily_cap, daily_spent
    );

    let body = json!({
        "model": "llama-3.3-70b",
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_content }
        ]
    });

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.venice.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let resp_json: serde_json::Value = resp.json().await?;
    tracing::debug!("Venice AI raw response: {}", resp_json);

    let raw = resp_json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("");

    // Venice (and most LLMs) sometimes wraps JSON in markdown code fences.
    // Strip them so serde_json can parse the bare object.
    let content = raw
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    match serde_json::from_str::<VeniceDecision>(content) {
        Ok(decision) => Ok(decision),
        Err(e) => {
            tracing::error!("Venice AI parse error: {} | Raw content: {}", e, content);
            Ok(VeniceDecision {
                approved: false,
                amount: 0.0,
                reason: "AI parse error".to_string(),
            })
        }
    }
}
