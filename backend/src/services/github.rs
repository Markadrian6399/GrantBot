use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn verify_webhook_signature(secret: &str, payload: &[u8], signature_header: &str) -> bool {
    let hex_sig = match signature_header.strip_prefix("sha256=") {
        Some(s) => s,
        None => return false,
    };

    let sig_bytes = match hex::decode(hex_sig) {
        Ok(b) => b,
        Err(_) => return false,
    };

    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(payload);

    mac.verify_slice(&sig_bytes).is_ok()
}

#[allow(dead_code)]
pub async fn get_pr_author(owner: &str, repo: &str, pr_number: i32) -> anyhow::Result<String> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}",
        owner, repo, pr_number
    );

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("User-Agent", "GrantBot/1.0")
        .send()
        .await?;

    let body: serde_json::Value = resp.json().await?;
    let author = body["user"]["login"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Could not extract PR author from GitHub response"))?
        .to_string();

    Ok(author)
}
