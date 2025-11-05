/// Example: Load Solana NFTs using Metaplex Token Metadata
///
/// This example demonstrates how to load NFTs owned by a wallet using the
/// Metaplex Token Metadata program integration.
///
/// Features demonstrated:
/// - Load all NFTs owned by a wallet
/// - Load individual NFT metadata
/// - Fetch off-chain JSON metadata (IPFS/Arweave)
/// - Display NFT attributes and properties
///
/// To run this example:
/// ```bash
/// cargo run --example solana_nft_loading --features crypto_solana
/// ```

#[cfg(all(feature = "crypto_solana", not(target_arch = "wasm32")))]
#[tokio::main]
async fn main() {
    use idos_game_sdk::{
        crypto_solana::{SolanaCluster, SolanaHandler, SolanaSettings},
        IdosClient, IdosConfig,
    };

    println!("🎨 Solana NFT Loading Example with Metaplex");
    println!("==========================================\n");

    // Initialize client
    let config = IdosConfig {
        api_key: "your_api_key".to_string(),
        game_id: "your_game_id".to_string(),
        ..Default::default()
    };
    let client = IdosClient::new(config);

    // Configure Solana (use devnet for testing)
    let settings = SolanaSettings {
        cluster: SolanaCluster::Devnet,
        rpc_url: "https://api.devnet.solana.com".to_string(),
        ws_url: None,
        program_id: String::new(),
    };

    let solana = SolanaHandler::new(client, settings);

    // Example wallet address (replace with your own)
    let wallet_address = "YourWalletAddressHere";

    println!("📍 Loading NFTs for wallet: {}\n", wallet_address);

    // Load all NFTs owned by the wallet
    match solana.load_nfts(wallet_address).await {
        Ok(result) => {
            println!("✅ Found {} NFTs\n", result.count);

            for (i, nft) in result.nfts.iter().enumerate() {
                println!("═══════════════════════════════════════");
                println!("NFT #{}", i + 1);
                println!("═══════════════════════════════════════");

                // On-chain metadata
                println!("🎨 On-Chain Metadata:");
                println!("  Mint:           {}", nft.metadata.mint);
                println!("  Name:           {}", nft.metadata.name);
                println!("  Symbol:         {}", nft.metadata.symbol);
                println!("  URI:            {}", nft.metadata.uri);
                println!(
                    "  Royalty:        {}%",
                    nft.metadata.seller_fee_basis_points as f64 / 100.0
                );
                println!("  Update Auth:    {}", nft.metadata.update_authority);
                println!("  Mutable:        {}", nft.metadata.is_mutable);
                println!("  Primary Sale:   {}", nft.metadata.primary_sale_happened);

                // Creators
                if let Some(creators) = &nft.metadata.creators {
                    println!("  Creators:");
                    for creator in creators {
                        println!(
                            "    - {} ({}%, verified: {})",
                            creator.address, creator.share, creator.verified
                        );
                    }
                }

                // Collection
                if let Some(collection) = &nft.metadata.collection {
                    println!(
                        "  Collection:     {} (verified: {})",
                        collection.key, collection.verified
                    );
                }

                // Uses
                if let Some(uses) = &nft.metadata.uses {
                    println!(
                        "  Uses:           {} / {} (method: {})",
                        uses.remaining, uses.total, uses.use_method
                    );
                }

                // Off-chain JSON metadata
                if let Some(json) = &nft.json_metadata {
                    println!("\n📝 Off-Chain Metadata:");
                    println!("  Name:           {}", json.name);
                    println!("  Symbol:         {}", json.symbol);

                    if let Some(desc) = &json.description {
                        println!("  Description:    {}", desc);
                    }

                    if let Some(image) = &json.image {
                        println!("  Image:          {}", image);
                    }

                    if let Some(url) = &json.external_url {
                        println!("  External URL:   {}", url);
                    }

                    // Attributes
                    if let Some(attributes) = &json.attributes {
                        println!("  Attributes:");
                        for attr in attributes {
                            println!("    - {}: {}", attr.trait_type, attr.value);
                        }
                    }
                }

                println!();
            }

            // Example: Load a specific NFT by mint
            if let Some(first_nft) = result.nfts.first() {
                println!("\n🔍 Loading specific NFT: {}", first_nft.metadata.mint);

                match solana
                    .load_nft(&first_nft.metadata.mint, wallet_address)
                    .await
                {
                    Ok(nft) => {
                        println!("✅ Successfully loaded NFT: {}", nft.metadata.name);
                    }
                    Err(e) => {
                        println!("❌ Failed to load NFT: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to load NFTs: {}", e);
            println!("\n💡 Tips:");
            println!("  - Make sure you're using a valid wallet address");
            println!("  - Ensure the wallet owns NFTs on the selected network");
            println!("  - Try using devnet if you don't have mainnet NFTs");
            println!("  - Check your RPC endpoint is accessible");
        }
    }

    println!("\n==========================================");
    println!("✨ NFT Loading Example Complete");
}

#[cfg(not(all(feature = "crypto_solana", not(target_arch = "wasm32"))))]
fn main() {
    println!("⚠️  This example requires:");
    println!("  - The 'crypto_solana' feature to be enabled");
    println!("  - Native compilation (not WASM)");
    println!("\nRun with:");
    println!("  cargo run --example solana_nft_loading --features crypto_solana");
}
