# Inventory Module Implementation

## ✅ Complete Implementation - 100% Unity Parity

The Inventory module manages player items and virtual currencies with local caching for performance.

---

## 📦 Module Structure

Following the SDK's modular architecture:

```
src/inventory/
├── dto.rs                  # Data structures
├── handler.rs              # Core API handler with caching
├── inventory_plugin.rs     # Bevy plugin
└── mod.rs                  # Module exports
```

---

## 📊 Data Structures (`dto.rs`)

### Core Models

**`GetUserInventoryResult`** - Complete inventory from server:
```rust
pub struct GetUserInventoryResult {
    pub inventory: Vec<ItemInstance>,
    pub virtual_currency: HashMap<String, i32>,
    pub virtual_currency_recharge_times: Option<HashMap<String, VirtualCurrencyRechargeTime>>,
}
```

**`ItemInstance`** - Individual item in inventory:
```rust
pub struct ItemInstance {
    pub item_id: String,
    pub item_instance_id: Option<String>,
    pub display_name: Option<String>,
    pub item_class: Option<String>,
    pub remaining_uses: Option<i32>,
    pub uses_incremented_by: Option<i32>,
    pub expiration: Option<String>,
    pub purchase_date: Option<String>,
    // ... more fields
}
```

**`VirtualCurrencyRechargeTime`** - For rechargeable currencies:
```rust
pub struct VirtualCurrencyRechargeTime {
    pub recharge_max: i32,
    pub recharge_time: String,
    pub seconds_to_recharge: i32,
}
```

---

## 🔧 Handler API (`handler.rs`)

### Initialization

```rust
use idos_game_sdk::inventory::InventoryHandler;

let client = IdosClient::new(config);
let mut inventory = InventoryHandler::new(client);

// After login
inventory.set_auth(user_id, session_ticket);
```

### Get Inventory

```rust
// Fetch complete inventory from server and cache locally
let result = inventory.get_inventory().await?;

println!("Items: {}", result.inventory.len());
println!("Currencies: {}", result.virtual_currency.len());

// Access items
for item in result.inventory {
    println!("{}: x{}", 
        item.display_name.unwrap_or(item.item_id), 
        item.remaining_uses.unwrap_or(1)
    );
}

// Access currencies
for (currency_id, amount) in result.virtual_currency {
    println!("{}: {}", currency_id, amount);
}
```

### Check Item Amounts (Cached)

```rust
// Get from cache (call get_inventory first to refresh)
let sword_count = inventory.get_item_amount("sword_legendary_001");
println!("Legendary swords: {}", sword_count);

// Quick ownership check
if inventory.has_item("sword_legendary_001") {
    println!("Player owns this item!");
}
```

### Check Virtual Currency (Cached)

```rust
// Get from cache
let gold = inventory.get_virtual_currency_amount("GOLD");
println!("Gold: {}", gold);

// Check if player has enough
if inventory.has_currency("GOLD", 100) {
    println!("Player has at least 100 GOLD!");
}
```

### Subtract Virtual Currency

```rust
// Server-side operation with local validation
match inventory.subtract_virtual_currency("GOLD", 50).await {
    Ok(response) => {
        println!("Subtracted 50 GOLD");
        // Cache is automatically updated
        println!("New balance: {}", inventory.get_virtual_currency_amount("GOLD"));
    }
    Err(e) => println!("Failed: {}", e),
}
```

### Grant Items

```rust
// Add items to player's inventory (server-side)
let items = inventory.grant_items(
    vec![
        "potion_health".to_string(),
        "potion_mana".to_string(),
    ],
    None,  // Catalog version (optional)
).await?;

println!("Granted {} items", items.len());
```

### Consume Item

```rust
// Use/reduce item (server-side)
let response = inventory.consume_item(
    "item_instance_xyz123",  // Item instance ID
    1,                        // Consume count
).await?;

println!("Remaining uses: {}", response.remaining_uses);
```

### Get All Items and Currencies

```rust
// Get all cached items
let all_items = inventory.get_all_items();
for (item_id, quantity) in all_items {
    println!("{}: x{}", item_id, quantity);
}

// Get all cached currencies
let all_currencies = inventory.get_all_currencies();
for (currency_id, amount) in all_currencies {
    println!("{}: {}", currency_id, amount);
}
```

---

## 🎮 Bevy Integration

### Add Plugin

```rust
use idos_game_sdk::inventory::InventoryPlugin;

App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(InventoryPlugin)
    .run();
```

### Use in Systems

```rust
fn inventory_system(
    mut inventory: ResMut<InventoryHandler>,
) {
    // Check items
    if inventory.has_item("sword") {
        // Do something
    }
    
    // Check currency
    if inventory.has_currency("GOLD", 100) {
        // Can afford action
    }
}
```

---

## 🌐 WASM Compatibility

✅ **Full WASM Support** - The inventory module works identically on:
- **Native builds** (Desktop, Mobile)
- **WASM builds** (Web browsers)

All operations use WASM-compatible HTTP requests via `reqwest`.

---

## 💾 Local Caching

The inventory handler maintains local caches for performance:

```rust
// Internal caches (HashMap):
items: HashMap<String, i32>              // item_id -> quantity
virtual_currency: HashMap<String, i32>   // currency_id -> amount
```

**Benefits:**
- ✅ Fast lookups without server roundtrips
- ✅ Automatic cache updates after operations
- ✅ Reduced server load

**Usage Pattern:**
```rust
// 1. Fetch from server (caches locally)
inventory.get_inventory().await?;

// 2. Use cached data (instant, no server call)
let gold = inventory.get_virtual_currency_amount("GOLD");
let has_sword = inventory.has_item("sword_001");

// 3. Server operations auto-update cache
inventory.subtract_virtual_currency("GOLD", 50).await?;
// Cache is now updated with new balance
```

---

## 📝 Unity SDK Parity

| Unity Feature | Rust SDK | Status |
|---------------|----------|--------|
| `GetUserInventory` | `get_inventory()` | ✅ |
| `UserInventory.GetItemAmount` | `get_item_amount()` | ✅ |
| `UserInventory.GetVirtualCurrencyAmount` | `get_virtual_currency_amount()` | ✅ |
| `SubtractVirtualCurrency` | `subtract_virtual_currency()` | ✅ |
| `GrantItemsToUser` | `grant_items()` | ✅ |
| `ConsumeItem` | `consume_item()` | ✅ |
| Check if has item | `has_item()` | ✅ |
| Check if has currency | `has_currency()` | ✅ |
| Local caching | `HashMap` cache | ✅ |
| Auto cache updates | Automatic | ✅ |

**Result: 100% Parity** ✅

---

## 🚀 Complete Usage Example

```rust
use idos_game_sdk::{IdosClient, IdosConfig, IdosResult};
use idos_game_sdk::inventory::InventoryHandler;

#[tokio::main]
async fn main() -> IdosResult<()> {
    // Setup
    let config = IdosConfig {
        api_key: "your_api_key".to_string(),
        game_id: "your_game_id".to_string(),
        ..Default::default()
    };

    let client = IdosClient::new(config);
    let mut inventory = InventoryHandler::new(client);
    
    // Authenticate
    inventory.set_auth(user_id, session_ticket);

    // Fetch inventory (caches locally)
    let inv = inventory.get_inventory().await?;
    
    // Check if player can afford something
    if inventory.has_currency("GOLD", 100) {
        // Deduct currency
        inventory.subtract_virtual_currency("GOLD", 100).await?;
        
        // Grant item
        inventory.grant_items(
            vec!["legendary_sword".to_string()],
            None,
        ).await?;
        
        println!("Purchased legendary sword!");
    }

    // Check owned items
    let sword_count = inventory.get_item_amount("legendary_sword");
    println!("You have {} legendary swords", sword_count);

    Ok(())
}
```

---

## 🎯 Best Practices

### 1. Cache Management
```rust
// ✅ Good: Refresh cache periodically
inventory.get_inventory().await?;  // Syncs with server

// ✅ Good: Use cached data for frequent checks
if inventory.has_currency("GOLD", price) {
    // Fast lookup, no server call
}
```

### 2. Error Handling
```rust
// ✅ Good: Handle insufficient currency gracefully
match inventory.subtract_virtual_currency("GOLD", 100).await {
    Ok(_) => println!("Purchase successful"),
    Err(IdosError::InvalidInput(msg)) => {
        println!("Not enough currency: {}", msg);
    }
    Err(e) => println!("Server error: {}", e),
}
```

### 3. Item Consumption
```rust
// Consumable items (potions, tickets, etc.)
let response = inventory.consume_item(instance_id, 1).await?;

if response.remaining_uses == 0 {
    println!("Item fully consumed");
} else {
    println!("{} uses remaining", response.remaining_uses);
}
```

---

## ✅ Implementation Complete!

The Inventory module is production-ready with:
- ✅ Full WASM compatibility
- ✅ 100% Unity SDK parity
- ✅ Local caching for performance
- ✅ Automatic cache updates
- ✅ Virtual currency management
- ✅ Item grants and consumption
- ✅ Ownership and balance checks

**Ready for item management in your Bevy games!** 📦

