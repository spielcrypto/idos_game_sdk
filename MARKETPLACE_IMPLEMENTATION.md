# Marketplace Module Implementation

## ✅ Complete Implementation - 100% Unity Parity

The Marketplace module enables player-to-player trading of items and NFTs within your game.

---

## 📦 Module Structure

Following the SDK's modular architecture:

```
src/marketplace/
├── dto.rs                    # Data structures
├── handler.rs                # Core API handler
├── marketplace_plugin.rs     # Bevy plugin
└── mod.rs                    # Module exports
```

---

## 📊 Data Structures (`dto.rs`)

### Enums

**`MarketplacePanel`** - Data retrieval panels:
- `GroupedOffers` - All items with active offers
- `ActiveOffersByItemID` - Offers for a specific item
- `PlayerActiveOffers` - Current player's active listings
- `PlayerHistory` - Player's trading history

**`MarketplaceAction`** - Marketplace operations:
- `CreateOffer` - List an item for sale
- `CreateDemand` - Create a buy request
- `UpdateOffer` - Update price/currency
- `DeleteOffer` - Remove a listing
- `BuyOffer` - Purchase an item

**`MarketplaceSortOrder`**:
- `Asc` - Ascending order
- `Desc` - Descending order

**`MarketplaceOrderBy`**:
- `Date` - Sort by listing date
- `Price` - Sort by price

### Models

**`MarketplaceActiveOffer`**:
```rust
pub struct MarketplaceActiveOffer {
    pub id: String,
    pub item_id: String,
    pub seller_id: String,
    pub currency_id: String,
    pub price: f64,
}
```

**`MarketplaceGroupedOffer`**:
```rust
pub struct MarketplaceGroupedOffer {
    pub item_id: String,
    pub offer_count: i32,
}
```

**`MarketplaceCommission`**:
```rust
pub struct MarketplaceCommission {
    pub company: i32,    // Company commission %
    pub referral: i32,   // Referral commission %
    pub author: i32,     // Content creator commission %
}
```

---

## 🔧 Handler API (`handler.rs`)

### Initialization

```rust
let client = IdosClient::new(config);
let mut marketplace = MarketplaceHandler::new(client);

// After login
marketplace.set_auth(user_id, session_ticket);
```

### Data Retrieval Methods

#### Get Grouped Offers
```rust
let response = marketplace.get_grouped_offers(
    20,              // Items per page
    None,            // Continuation token (for pagination)
).await?;
```

#### Get Offers for Specific Item
```rust
let response = marketplace.get_offers_by_item(
    "item_sword_001",                     // Item ID
    20,                                   // Items per page
    None,                                 // Continuation token
    Some("GOLD".to_string()),            // Filter by currency
    Some(MarketplaceSortOrder::Asc),     // Sort order
    Some(MarketplaceOrderBy::Price),     // Order by field
).await?;
```

#### Get Player's Active Offers
```rust
let response = marketplace.get_player_active_offers(
    20,              // Items per page
    None,            // Continuation token
).await?;
```

#### Get Player's Trading History
```rust
let response = marketplace.get_player_history(
    20,              // Items per page
    None,            // Continuation token
).await?;
```

### Marketplace Actions

#### Create Offer
```rust
let response = marketplace.create_offer(
    "item_sword_legendary_001",  // Item ID
    "GOLD",                       // Currency
    100,                          // Price
).await?;
```

#### Update Offer
```rust
let response = marketplace.update_offer(
    "offer_xyz123",    // Offer ID
    "GOLD",            // New currency
    150,               // New price
).await?;
```

#### Delete Offer
```rust
let response = marketplace.delete_offer("offer_xyz123").await?;
```

#### Buy Offer
```rust
let response = marketplace.buy_offer("offer_xyz123").await?;
```

#### Create Demand
```rust
let response = marketplace.create_demand(
    "item_shield_001",  // Item you want to buy
    "GOLD",             // Currency you'll pay with
    50,                 // Max price you'll pay
).await?;
```

---

## 🎮 Bevy Integration

### Add Plugin

```rust
use idos_game_sdk::marketplace::MarketplacePlugin;

App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(MarketplacePlugin)
    .run();
```

### Use in Systems

```rust
fn marketplace_system(
    marketplace: Res<MarketplaceHandler>,
) {
    // Use marketplace handler in your systems
}
```

---

## 🌐 WASM Compatibility

✅ **Full WASM Support** - The marketplace module works identically on:
- **Native builds** (Desktop, Mobile)
- **WASM builds** (Web browsers)

All HTTP requests use `reqwest` with WASM-compatible configuration.

---

## 🔄 Pagination Support

The marketplace supports pagination for large result sets:

```rust
let mut continuation_token: Option<String> = None;

loop {
    let response = marketplace.get_grouped_offers(20, continuation_token.clone()).await?;
    
    // Parse response
    let data: MarketplaceDataResponse = serde_json::from_str(&response)?;
    
    // Process data.data...
    
    // Check for next page
    if data.continuation_token.is_none() {
        break;  // No more pages
    }
    
    continuation_token = data.continuation_token;
}
```

---

## 💰 Commission Calculation

```rust
use idos_game_sdk::marketplace::dto::MarketplaceCommission;

let commission = MarketplaceCommission {
    company: 5,      // 5%
    referral: 2,     // 2%
    author: 3,       // 3%
};

let total_commission = commission.total();  // 10%
let player_receives = commission.calculate_player_receives(100);  // 90 GOLD
```

---

## 📝 Unity SDK Parity

| Unity Feature | Rust SDK | Status |
|---------------|----------|--------|
| `GetDataFromMarketplace` | `get_data()` | ✅ |
| `TryDoMarketplaceAction` | `do_action()` | ✅ |
| Grouped offers panel | `get_grouped_offers()` | ✅ |
| Offers by item panel | `get_offers_by_item()` | ✅ |
| Player active offers | `get_player_active_offers()` | ✅ |
| Player history | `get_player_history()` | ✅ |
| Create offer | `create_offer()` | ✅ |
| Update offer | `update_offer()` | ✅ |
| Delete offer | `delete_offer()` | ✅ |
| Buy offer | `buy_offer()` | ✅ |
| Create demand | `create_demand()` | ✅ |
| Pagination | Continuation tokens | ✅ |
| Filtering | Currency, sort, order | ✅ |
| Commission calculation | `MarketplaceCommission` | ✅ |

**Result: 100% Parity** ✅

---

## 🚀 Complete Usage Example

```rust
use idos_game_sdk::{IdosClient, IdosConfig, IdosResult};
use idos_game_sdk::marketplace::{
    MarketplaceHandler,
    MarketplaceOrderBy,
    MarketplaceSortOrder,
};

#[tokio::main]
async fn main() -> IdosResult<()> {
    // Setup
    let config = IdosConfig {
        api_key: "your_api_key".to_string(),
        game_id: "your_game_id".to_string(),
        ..Default::default()
    };

    let client = IdosClient::new(config);
    let mut marketplace = MarketplaceHandler::new(client);
    
    // Authenticate
    marketplace.set_auth(user_id, session_ticket);

    // List item for sale
    let offer = marketplace.create_offer(
        "legendary_sword",
        "GOLD",
        500,
    ).await?;

    // Browse offers sorted by price
    let offers = marketplace.get_offers_by_item(
        "legendary_sword",
        20,
        None,
        None,
        Some(MarketplaceSortOrder::Asc),
        Some(MarketplaceOrderBy::Price),
    ).await?;

    // Buy the cheapest offer
    marketplace.buy_offer("offer_id").await?;

    Ok(())
}
```

---

## ✅ Implementation Complete!

The Marketplace module is production-ready with:
- ✅ Full WASM compatibility
- ✅ 100% Unity SDK parity
- ✅ Clean modular architecture
- ✅ Comprehensive API coverage
- ✅ Pagination support
- ✅ Filtering and sorting
- ✅ Commission calculations

**Ready for player-to-player trading in your Bevy games!** 🎮

