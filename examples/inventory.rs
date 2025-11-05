/// Example demonstrating Inventory functionality
/// Manage items and virtual currencies
///
/// Run with: cargo run --example inventory --features inventory

#[cfg(feature = "inventory")]
use idos_game_sdk::inventory::handler::InventoryHandler;

#[cfg(feature = "inventory")]
use idos_game_sdk::{IdosClient, IdosConfig, IdosResult};

#[cfg(feature = "inventory")]
#[tokio::main]
async fn main() -> IdosResult<()> {
    println!("📦 Inventory Example\n");

    // Initialize client
    let config = IdosConfig {
        api_key: "your_api_key".to_string(),
        game_id: "your_game_id".to_string(),
        ..Default::default()
    };

    let client = IdosClient::new(config);
    let mut inventory = InventoryHandler::new(client);

    // Set authentication (normally from login response)
    inventory.set_auth("user123".to_string(), "session_ticket_xyz".to_string());

    println!("✅ Inventory handler initialized\n");

    // Example 1: Get user inventory
    println!("📊 Example 1: Get User Inventory");
    println!("─────────────────────────────────");

    match inventory.get_inventory().await {
        Ok(result) => {
            println!("✅ Inventory retrieved");
            println!("   Total items: {}", result.inventory.len());
            println!("   Virtual currencies: {}", result.virtual_currency.len());

            // Show items
            println!("\n   Items:");
            for (i, item) in result.inventory.iter().take(5).enumerate() {
                println!(
                    "   {}. {} (x{})",
                    i + 1,
                    item.display_name.as_deref().unwrap_or(&item.item_id),
                    item.remaining_uses.unwrap_or(1)
                );
            }

            // Show currencies
            println!("\n   Virtual Currencies:");
            for (currency_id, amount) in &result.virtual_currency {
                println!("   • {}: {}", currency_id, amount);
            }
            println!();
        }
        Err(e) => println!("❌ Failed: {}\n", e),
    }

    // Example 2: Check item amounts (from cache)
    println!("🔍 Example 2: Check Item Amounts");
    println!("────────────────────────────────");

    let item_id = "sword_legendary_001";
    let amount = inventory.get_item_amount(item_id);
    println!("Item '{}': {} in inventory", item_id, amount);

    if inventory.has_item(item_id) {
        println!("✅ Player owns this item!\n");
    } else {
        println!("❌ Player doesn't have this item\n");
    }

    // Example 3: Check virtual currency
    println!("💰 Example 3: Check Virtual Currency");
    println!("────────────────────────────────────");

    let currency_id = "GOLD";
    let amount = inventory.get_virtual_currency_amount(currency_id);
    println!("{}: {} available", currency_id, amount);

    let required = 100;
    if inventory.has_currency(currency_id, required) {
        println!(
            "✅ Player has enough {} (need: {})\n",
            currency_id, required
        );
    } else {
        println!(
            "❌ Not enough {} (have: {}, need: {})\n",
            currency_id, amount, required
        );
    }

    // Example 4: Subtract virtual currency
    println!("💸 Example 4: Subtract Virtual Currency");
    println!("───────────────────────────────────────");

    match inventory.subtract_virtual_currency("GOLD", 50).await {
        Ok(response) => {
            println!("✅ Successfully subtracted 50 GOLD");
            println!(
                "   New amount: {}",
                inventory.get_virtual_currency_amount("GOLD")
            );
            println!("   Response: {}\n", response);
        }
        Err(e) => println!("❌ Failed: {}\n", e),
    }

    // Example 5: Grant items to user
    println!("🎁 Example 5: Grant Items to User");
    println!("──────────────────────────────────");

    match inventory
        .grant_items(
            vec![
                "potion_health_001".to_string(),
                "potion_health_001".to_string(),
                "sword_iron_001".to_string(),
            ],
            None,
        )
        .await
    {
        Ok(items) => {
            println!("✅ Items granted successfully");
            println!("   Granted {} items", items.len());

            for item in items {
                println!(
                    "   • {} ({})",
                    item.display_name.as_deref().unwrap_or(&item.item_id),
                    item.item_id
                );
            }
            println!();
        }
        Err(e) => println!("❌ Failed: {}\n", e),
    }

    // Example 6: Consume an item
    println!("🔄 Example 6: Consume Item");
    println!("──────────────────────────");

    let item_instance_id = "instance_xyz123";
    match inventory.consume_item(item_instance_id, 1).await {
        Ok(response) => {
            println!("✅ Item consumed successfully");
            println!("   Instance ID: {}", response.item_instance_id);
            println!("   Remaining uses: {}\n", response.remaining_uses);
        }
        Err(e) => println!("❌ Failed: {}\n", e),
    }

    // Example 7: Get all items and currencies
    println!("📋 Example 7: List All Inventory");
    println!("────────────────────────────────");

    let all_items = inventory.get_all_items();
    let all_currencies = inventory.get_all_currencies();

    println!("Items in cache: {} types", all_items.len());
    for (item_id, quantity) in all_items.iter().take(3) {
        println!("   • {}: x{}", item_id, quantity);
    }

    println!("\nCurrencies in cache: {} types", all_currencies.len());
    for (currency_id, amount) in all_currencies {
        println!("   • {}: {}", currency_id, amount);
    }

    println!("\n✨ Inventory examples complete!");
    println!("\n📚 Available Operations:");
    println!("   • get_inventory() - Fetch complete inventory from server");
    println!("   • get_item_amount() - Check cached item quantity");
    println!("   • get_virtual_currency_amount() - Check cached currency");
    println!("   • has_item() - Quick ownership check");
    println!("   • has_currency() - Quick currency check");
    println!("   • subtract_virtual_currency() - Spend currency");
    println!("   • grant_items() - Add items to inventory");
    println!("   • consume_item() - Use/reduce item");
    println!("   • get_all_items() - Get all cached items");
    println!("   • get_all_currencies() - Get all cached currencies");

    Ok(())
}

// Fallback main for non-inventory builds
#[cfg(not(feature = "inventory"))]
fn main() {
    println!("❌ This example requires the 'inventory' feature.");
    println!("Run with: cargo run --example inventory --features inventory");
}
