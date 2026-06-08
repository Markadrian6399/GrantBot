use serde_json::json;

const USDC_SEPOLIA: &str = "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238";

/// Encodes ERC-20 transfer(address, uint256) calldata.
fn encode_transfer_calldata(to_address: &str, amount_micro: u64) -> String {
    // Function selector: keccak256("transfer(address,uint256)") first 4 bytes = 0xa9059cbb
    let selector = "a9059cbb";

    // Strip 0x prefix and pad address to 32 bytes (64 hex chars)
    let addr = to_address.trim_start_matches("0x");
    let addr_padded = format!("{:0>64}", addr);

    // Encode uint256 as 32 bytes (64 hex chars)
    let amount_padded = format!("{:064x}", amount_micro);

    format!("0x{}{}{}", selector, addr_padded, amount_padded)
}

pub async fn execute_usdc_transfer(
    to_address: &str,
    amount_usdc: f64,
    delegation_hex: &str,
) -> anyhow::Result<String> {
    let api_key = std::env::var("ONESHOT_API_KEY")
        .unwrap_or_else(|_| "missing".to_string());

    let oneshot_url = std::env::var("ONESHOT_API_URL")
        .unwrap_or_else(|_| "https://relay.1shot.io/api/v1/relay".to_string());

    // Convert USDC amount to 6-decimal micro units
    let amount_micro = (amount_usdc * 1_000_000.0) as u64;
    let calldata = encode_transfer_calldata(to_address, amount_micro);

    let body = json!({
        "delegation": delegation_hex,
        "target": USDC_SEPOLIA,
        "calldata": calldata,
        "value": "0"
    });

    tracing::info!(
        "1Shot relay request: to={}, amount_usdc={}, amount_micro={}, calldata={}",
        to_address, amount_usdc, amount_micro, calldata
    );

    let client = reqwest::Client::new();
    let resp = client
        .post(&oneshot_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let status = resp.status();
    let body_text = resp.text().await?;

    tracing::info!("1Shot relay response status={}, body={}", status, body_text);

    if !status.is_success() {
        return Err(anyhow::anyhow!(
            "1Shot relay error {}: {}",
            status,
            body_text
        ));
    }

    let resp_json: serde_json::Value = serde_json::from_str(&body_text)?;
    let tx_hash = resp_json["txHash"]
        .as_str()
        .or_else(|| resp_json["tx_hash"].as_str())
        .or_else(|| resp_json["hash"].as_str())
        .ok_or_else(|| anyhow::anyhow!("No tx_hash in 1Shot response: {}", body_text))?
        .to_string();

    Ok(tx_hash)
}
