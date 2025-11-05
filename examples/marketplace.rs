/// Example demonstrating Marketplace functionality
/// Player-to-player trading system for items and NFTs
///
/// Run with: cargo run --example marketplace --features marketplace

#[cfg(feature = "marketplace")]
use idos_game_sdk::marketplace::{
    dto::{MarketplaceAction, MarketplaceOrderBy, MarketplacePanel, MarketplaceSortOrder},
    handler::MarketplaceHandler,
};

#[cfg(feature = "marketplace")]
use idos_game_sdk::{IdosClient, IdosConfig, IdosResult};

#[cfg(feature = "marketplace")]
#[tokio::main]
async fn main() -> IdosResult<()> {
    println!("🏪 Marketplace Example\n");

    // Initialize client
    let config = IdosConfig {
        api_key: "your_api_key".to_string(),
        game_id: "your_game_id".to_string(),
        ..Default::default()
    };

    let client = IdosClient::new(config);
    let mut marketplace = MarketplaceHandler::new(client);

    // Set authentication (normally from login response)
    marketplace.set_auth("user123".to_string(), "session_ticket_xyz".to_string());

    println!("✅ Marketplace handler initialized\n");

    // Example 1: Get grouped offers (all items with available offers)
    println!("📊 Example 1: Get Grouped Offers");
    println!("─────────────────────────────────");

    match marketplace.get_grouped_offers(20, None).await {
        Ok(response) => {
            println!("✅ Grouped offers retrieved");
            println!("   Response: {}\n", response);
        }
        Err(e) => println!("❌ Failed: {}\n", e),
    }

    // Example 2: Get offers for a specific item
    println!("🔍 Example 2: Get Offers for Specific Item");
    println!("───────────────────────────────────────────");

    let item_id = "item_sword_001";
    match marketplace
        .get_offers_by_item(
            item_id,
            20,
            None,
            Some("GOLD".to_string()),        // Filter by currency
            Some(MarketplaceSortOrder::Asc), // Ascending order
            Some(MarketplaceOrderBy::Price), // Sort by price
        )
        .await
    {
        Ok(response) => {
            println!("✅ Offers for item '{}' retrieved", item_id);
            println!("   Sorted by: Price (ascending)");
            println!("   Response: {}\n", response);
        }
        Err(e) => println!("❌ Failed: {}\n", e),
    }

    // Example 3: Get player's active offers
    println!("📦 Example 3: Get Player's Active Offers");
    println!("────────────────────────────────────────");

    match marketplace.get_player_active_offers(20, None).await {
        Ok(response) => {
            println!("✅ Player active offers retrieved");
            println!("   Response: {}\n", response);
        }
        Err(e) => println!("❌ Failed: {}\n", e),
    }

    // Example 4: Create a marketplace offer
    println!("✨ Example 4: Create Marketplace Offer");
    println!("──────────────────────────────────────");

    match marketplace
        .create_offer(
            "item_sword_legendary_001", // Item ID
            "GOLD",                     // Currency
            100,                        // Price
        )
        .await
    {
        Ok(response) => {
            println!("✅ Offer created successfully");
            println!("   Item: item_sword_legendary_001");
            println!("   Price: 100 GOLD");
            println!("   Response: {}\n", response);
        }
        Err(e) => println!("❌ Failed: {}\n", e),
    }

    // Example 5: Update an offer
    println!("✏️  Example 5: Update Marketplace Offer");
    println!("───────────────────────────────────────");

    let offer_id = "offer_xyz123";
    match marketplace
        .update_offer(
            offer_id, // Offer ID to update
            "GOLD",   // New currency
            150,      // New price
        )
        .await
    {
        Ok(response) => {
            println!("✅ Offer updated successfully");
            println!("   Offer ID: {}", offer_id);
            println!("   New price: 150 GOLD");
            println!("   Response: {}\n", response);
        }
        Err(e) => println!("❌ Failed: {}\n", e),
    }

    // Example 6: Buy an offer
    println!("💰 Example 6: Buy Marketplace Offer");
    println!("───────────────────────────────────");

    match marketplace.buy_offer(offer_id).await {
        Ok(response) => {
            println!("✅ Offer purchased successfully");
            println!("   Offer ID: {}", offer_id);
            println!("   Response: {}\n", response);
        }
        Err(e) => println!("❌ Failed: {}\n", e),
    }

    // Example 7: Delete an offer
    println!("🗑️  Example 7: Delete Marketplace Offer");
    println!("───────────────────────────────────────");

    match marketplace.delete_offer(offer_id).await {
        Ok(response) => {
            println!("✅ Offer deleted successfully");
            println!("   Offer ID: {}", offer_id);
            println!("   Response: {}\n", response);
        }
        Err(e) => println!("❌ Failed: {}\n", e),
    }

    // Example 8: Get player's marketplace history
    println!("📜 Example 8: Get Player's Trading History");
    println!("──────────────────────────────────────────");

    match marketplace.get_player_history(20, None).await {
        Ok(response) => {
            println!("✅ Trading history retrieved");
            println!("   Response: {}\n", response);
        }
        Err(e) => println!("❌ Failed: {}\n", e),
    }

    // Example 9: Pagination example
    println!("📄 Example 9: Pagination with Continuation Token");
    println!("─────────────────────────────────────────────────");

    // First page
    match marketplace.get_grouped_offers(10, None).await {
        Ok(response) => {
            println!("✅ Page 1 retrieved (10 items)");

            // In practice, you'd parse the response to get the continuation token
            // let parsed: MarketplaceDataResponse = serde_json::from_str(&response)?;
            // let continuation_token = parsed.continuation_token;

            println!("   To get next page, pass continuation_token to next request\n");
        }
        Err(e) => println!("❌ Failed: {}\n", e),
    }

    println!("✨ Marketplace examples complete!");
    println!("\n📚 Available Operations:");
    println!("   • get_grouped_offers() - Browse all items");
    println!("   • get_offers_by_item() - Filter by specific item");
    println!("   • get_player_active_offers() - Your active listings");
    println!("   • get_player_history() - Your trading history");
    println!("   • create_offer() - List item for sale");
    println!("   • update_offer() - Change price/currency");
    println!("   • delete_offer() - Remove listing");
    println!("   • buy_offer() - Purchase an item");
    println!("   • create_demand() - Create buy request");

    Ok(())
}

// Fallback main for non-marketplace builds
#[cfg(not(feature = "marketplace"))]
fn main() {
    println!("❌ This example requires the 'marketplace' feature.");
    println!("Run with: cargo run --example marketplace --features marketplace");
}
