/// Example demonstrating Ethereum gas estimation
/// Shows how to estimate gas for different transaction types
///
/// Run with: cargo run --example ethereum_gas_estimation --features crypto_ethereum

#[cfg(feature = "crypto_ethereum")]
use idos_game_sdk::crypto_ethereum::transactions::{
    estimate_gas, estimate_gas_erc20_approval, estimate_gas_erc20_transfer,
    estimate_gas_nft_transfer,
};

#[cfg(feature = "crypto_ethereum")]
use idos_game_sdk::IdosResult;

#[cfg(feature = "crypto_ethereum")]
#[tokio::main]
async fn main() -> IdosResult<()> {
    println!("⛽ Ethereum Gas Estimation Example\n");

    // Configuration
    let rpc_url = "https://eth-sepolia.g.alchemy.com/v2/demo"; // Sepolia testnet
    let from_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb";
    let to_address = "0x5FbDB2315678afecb367f032d93F642f64180aa3";
    let token_address = "0x0000000000000000000000000000000000000000"; // Example token
    let nft_address = "0x1111111111111111111111111111111111111111"; // Example NFT

    // Example 1: Generic gas estimation
    println!("📊 Example 1: Generic Transaction Gas Estimation");
    println!("─────────────────────────────────────────────────");

    match estimate_gas(
        rpc_url,
        from_address,
        to_address,
        None,                        // No data
        Some("1000000000000000000"), // 1 ETH in wei
    )
    .await
    {
        Ok(gas) => {
            println!("✅ Estimated gas: {} units", gas);
            println!(
                "   Cost at 20 gwei: {} ETH\n",
                gas as f64 * 20.0 / 1_000_000_000.0
            );
        }
        Err(e) => println!("❌ Estimation failed: {}\n", e),
    }

    // Example 2: ERC20 Transfer
    println!("💰 Example 2: ERC20 Transfer Gas Estimation");
    println!("────────────────────────────────────────────");

    match estimate_gas_erc20_transfer(
        rpc_url,
        token_address,
        from_address,
        to_address,
        "1000000000000000000", // 1 token (18 decimals)
    )
    .await
    {
        Ok(gas) => {
            println!("✅ Estimated gas: {} units", gas);
            println!("   Typical ERC20 transfer: ~60,000-80,000 gas");
            println!(
                "   Cost at 20 gwei: {} ETH\n",
                gas as f64 * 20.0 / 1_000_000_000.0
            );
        }
        Err(e) => println!("❌ Estimation failed: {}\n", e),
    }

    // Example 3: NFT Transfer (ERC1155)
    println!("🎨 Example 3: NFT Transfer Gas Estimation");
    println!("──────────────────────────────────────────");
    println!("This matches Unity's EstimateGasNFTAsync!");

    match estimate_gas_nft_transfer(
        rpc_url,
        nft_address,
        from_address,
        to_address,
        1,    // Token ID
        1,    // Amount
        None, // No additional data
    )
    .await
    {
        Ok(gas) => {
            println!("✅ Estimated gas: {} units", gas);
            println!("   Typical NFT transfer: ~100,000-150,000 gas");
            println!(
                "   Cost at 20 gwei: {} ETH\n",
                gas as f64 * 20.0 / 1_000_000_000.0
            );
        }
        Err(e) => println!("❌ Estimation failed: {}\n", e),
    }

    // Example 4: ERC20 Approval
    println!("✔️  Example 4: ERC20 Approval Gas Estimation");
    println!("────────────────────────────────────────────");

    match estimate_gas_erc20_approval(
        rpc_url,
        token_address,
        from_address,
        to_address,               // Spender
        "1000000000000000000000", // 1000 tokens
    )
    .await
    {
        Ok(gas) => {
            println!("✅ Estimated gas: {} units", gas);
            println!("   Typical approval: ~45,000-50,000 gas");
            println!(
                "   Cost at 20 gwei: {} ETH\n",
                gas as f64 * 20.0 / 1_000_000_000.0
            );
        }
        Err(e) => println!("❌ Estimation failed: {}\n", e),
    }

    // Example 5: Checking if user has sufficient gas
    println!("💡 Example 5: Gas Sufficiency Check");
    println!("────────────────────────────────────");

    let estimated_gas = 150_000u64;
    let gas_price_gwei = 20.0;
    let required_eth = (estimated_gas as f64 * gas_price_gwei) / 1_000_000_000.0;

    println!("Estimated gas needed: {} units", estimated_gas);
    println!("Gas price: {} gwei", gas_price_gwei);
    println!("Total ETH required: {} ETH", required_eth);
    println!("\nUser balance check would go here...");
    println!("if balance >= required_eth {{ proceed_with_transaction(); }}");

    println!("\n✨ Gas estimation examples complete!");
    println!("\n📚 Key Features:");
    println!("   • Generic gas estimation for any transaction");
    println!("   • Specialized estimation for ERC20 transfers");
    println!("   • NFT transfer estimation (ERC1155)");
    println!("   • Approval transaction estimation");
    println!("   • All using ethers.rs native estimation!");

    Ok(())
}

// Fallback main for non-Ethereum builds
#[cfg(not(feature = "crypto_ethereum"))]
fn main() {
    println!("❌ This example requires the 'crypto_ethereum' feature.");
    println!("Run with: cargo run --example ethereum_gas_estimation --features crypto_ethereum");
}
