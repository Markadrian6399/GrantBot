use backend::services::oneshot_relayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    println!("Testing 1Shot relayer execute_usdc_transfer...\n");

    let test_address = "0x742d35Cc6634C0532925a3b844Bc454e4438f44e";
    let test_delegation = "0xdeadbeef"; // mock delegation hex

    match oneshot_relayer::execute_usdc_transfer(test_address, 0.01, test_delegation).await {
        Ok(tx_hash) => {
            println!("Success! tx_hash: {}", tx_hash);
        }
        Err(e) => {
            println!("Error (expected if no real credentials): {:?}", e);
        }
    }

    Ok(())
}
