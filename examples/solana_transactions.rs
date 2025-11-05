/// Example demonstrating Solana transaction building, signing, and sending
/// Shows how to use the new transaction serialization capabilities
///
/// Run with: cargo run --example solana_transactions --features crypto_solana

#[cfg(all(feature = "crypto_solana", not(target_arch = "wasm32")))]
use idos_game_sdk::crypto_solana::{
    handler::SolanaHandler,
    service::SolanaPlatformPoolService,
    transactions::{
        estimate_transaction_fee, get_recent_blockhash, send_transaction, TransactionBuilder,
    },
};

#[cfg(all(feature = "crypto_solana", not(target_arch = "wasm32")))]
use idos_game_sdk::IdosResult;

#[cfg(all(feature = "crypto_solana", not(target_arch = "wasm32")))]
#[tokio::main]
async fn main() -> IdosResult<()> {
    println!("🚀 Solana Transaction Example\n");

    // Configuration
    let rpc_url = "https://api.devnet.solana.com";
    let program_id = "YourProgramID...";
    let private_key_base58 = "your_private_key_base58"; // 64-byte keypair in base58

    // Initialize client and handler
    use idos_game_sdk::crypto_solana::dto::{SolanaCluster, SolanaSettings};
    use idos_game_sdk::{IdosClient, IdosConfig};

    let config = IdosConfig {
        api_key: "test_api_key".to_string(),
        game_id: "test_game".to_string(),
        ..Default::default()
    };

    let client = IdosClient::new(config);

    let settings = SolanaSettings {
        cluster: SolanaCluster::Devnet,
        rpc_url: rpc_url.to_string(),
        ws_url: Some("wss://api.devnet.solana.com".to_string()),
        program_id: program_id.to_string(),
    };

    let handler = SolanaHandler::new(client, settings);

    let mut service = SolanaPlatformPoolService::new(handler);
    service.set_private_key(private_key_base58)?;

    println!("✅ Service initialized\n");

    // Example 1: Estimate transaction fees
    println!("📊 Example 1: Fee Estimation");
    println!("─────────────────────────────");

    let num_signatures = 1; // Single signer
    let estimated_fee = estimate_transaction_fee(num_signatures);
    println!(
        "Estimated fee for {} signature(s): {} lamports ({} SOL)",
        num_signatures,
        estimated_fee,
        estimated_fee as f64 / 1_000_000_000.0
    );

    // Multi-signature transaction
    let num_signatures = 3;
    let estimated_fee = estimate_transaction_fee(num_signatures);
    println!(
        "Estimated fee for {} signature(s): {} lamports ({} SOL)\n",
        num_signatures,
        estimated_fee,
        estimated_fee as f64 / 1_000_000_000.0
    );

    // Example 2: Get recent blockhash
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("🔗 Example 2: Get Recent Blockhash");
        println!("─────────────────────────────────");

        let blockhash = get_recent_blockhash(rpc_url).await?;
        println!("Recent blockhash: {}\n", blockhash);
    }

    // Example 3: Deposit SPL tokens (full flow)
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("💰 Example 3: Deposit SPL Tokens");
        println!("─────────────────────────────────");

        let mint_address = "So11111111111111111111111111111111111111112"; // Wrapped SOL
        let amount = 1_000_000; // 0.001 SOL
        let user_id = "user123";

        match service.deposit_spl(mint_address, amount, user_id).await {
            Ok(signature) => {
                println!("✅ Transaction sent successfully!");
                println!("   Signature: {}", signature);
                println!(
                    "   Explorer: https://explorer.solana.com/tx/{}?cluster=devnet\n",
                    signature
                );
            }
            Err(e) => {
                println!("❌ Transaction failed: {}\n", e);
            }
        }
    }

    // Example 4: Simulate a transaction before sending
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("🔮 Example 4: Transaction Simulation");
        println!("────────────────────────────────────");

        // This example shows how to simulate before sending
        // In practice, you'd build your transaction first

        println!("Note: Simulation happens automatically with skipPreflight=false");
        println!("The send_transaction function validates transactions before sending.\n");
    }

    // Example 5: Low-level transaction building
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("🔧 Example 5: Manual Transaction Building");
        println!("─────────────────────────────────────────");

        // Get user's public key from private key
        let key_bytes = bs58::decode(private_key_base58)
            .into_vec()
            .expect("Invalid private key");
        let mut user_pubkey = [0u8; 32];
        user_pubkey.copy_from_slice(&key_bytes[32..]); // Public key is second half

        // Get recent blockhash
        let blockhash = get_recent_blockhash(rpc_url).await?;

        // Create transaction builder
        let mut tx_builder = TransactionBuilder::new(user_pubkey);

        // Add instructions (simplified example - you'd add real instructions)
        // tx_builder.add_instruction(your_instruction);

        // Set blockhash
        tx_builder.set_recent_blockhash(&blockhash);

        // Get transaction size estimate
        let tx_size = tx_builder.estimate_size();
        println!("Estimated transaction size: {} bytes", tx_size);

        // Sign and serialize (commented out - would need real instruction)
        // let signed_tx = tx_builder.sign_and_serialize(&key_bytes)?;
        // println!("Signed transaction (base64): {}", signed_tx);

        println!("✅ Transaction builder ready (no instructions added in example)\n");
    }

    // Example 6: Best practices
    println!("📚 Best Practices");
    println!("─────────────────");
    println!("1. Always estimate fees before sending transactions");
    println!("2. Use simulation to validate transactions before sending");
    println!("3. Handle transaction confirmation properly");
    println!("4. For WASM builds, use Phantom/Solflare wallet adapter");
    println!("5. For native builds, use the TransactionBuilder API");
    println!("6. Store private keys securely (never hardcode!)");
    println!("\n✨ Transaction examples complete!");

    Ok(())
}

// Helper function to demonstrate transaction confirmation
#[cfg(all(feature = "crypto_solana", not(target_arch = "wasm32")))]
async fn wait_for_confirmation(
    rpc_url: &str,
    signature: &str,
    max_retries: u32,
) -> IdosResult<bool> {
    use tokio::time::{sleep, Duration};

    println!("⏳ Waiting for transaction confirmation...");

    for attempt in 1..=max_retries {
        // Check transaction status
        let status = check_transaction_status(rpc_url, signature).await?;

        if status {
            println!("✅ Transaction confirmed after {} attempt(s)!", attempt);
            return Ok(true);
        }

        println!(
            "   Attempt {}/{}: Not confirmed yet...",
            attempt, max_retries
        );
        sleep(Duration::from_secs(2)).await;
    }

    println!(
        "⚠️  Transaction not confirmed after {} attempts",
        max_retries
    );
    Ok(false)
}

#[cfg(all(feature = "crypto_solana", not(target_arch = "wasm32")))]
async fn check_transaction_status(rpc_url: &str, signature: &str) -> IdosResult<bool> {
    use idos_game_sdk::crypto_solana::dto::{TransactionStatusRequest, TransactionStatusResponse};

    let client = reqwest::Client::new();
    let request = TransactionStatusRequest {
        jsonrpc: "2.0".to_string(),
        id: 1,
        method: "getSignatureStatuses".to_string(),
        params: vec![signature.to_string()],
    };

    let response = client
        .post(rpc_url)
        .json(&request)
        .send()
        .await
        .map_err(|e| idos_game_sdk::IdosError::NetworkError(format!("Status check failed: {}", e)))?
        .json::<TransactionStatusResponse>()
        .await
        .map_err(|e| {
            idos_game_sdk::IdosError::NetworkError(format!("Failed to parse status: {}", e))
        })?;

    Ok(response.result.is_some())
}

// Fallback main for non-Solana builds
#[cfg(not(all(feature = "crypto_solana", not(target_arch = "wasm32"))))]
fn main() {
    println!("❌ This example requires the 'crypto_solana' feature and native build.");
    println!("Run with: cargo run --example solana_transactions --features crypto_solana");
}
