use backend::services::venice_agent;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    println!("Testing Venice AI evaluate_pr...\n");

    let decision = venice_agent::evaluate_pr(
        "Fix critical authentication bug in OAuth flow",
        "contributor1",
        5.0,
        50.0,
        12.5,
    )
    .await?;

    println!("VeniceDecision:");
    println!("  approved: {}", decision.approved);
    println!("  amount:   {} USDC", decision.amount);
    println!("  reason:   {}", decision.reason);

    Ok(())
}
