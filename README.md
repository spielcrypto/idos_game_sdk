# iDos Games SDK for Bevy

A comprehensive game development SDK for Bevy games, supporting both **native** and **WebAssembly (WASM)** targets.
[See original SDK in Unity](https://github.com/iDos-Games/iDos-Games-Engine-Unity-SDK)

## DOCS

📖 **Read the guides:**
- **[INVENTORY_IMPLEMENTATION.md](INVENTORY_IMPLEMENTATION.md)** - Items & currency
- **[MARKETPLACE_IMPLEMENTATION.md](MARKETPLACE_IMPLEMENTATION.md)** - Player trading
- **[LEADERBOARD_IMPLEMENTATION.md](LEADERBOARD_IMPLEMENTATION.md)** - Rankings & rewards
- **[METAPLEX_INTEGRATION.md](METAPLEX_INTEGRATION.md)** - NFT loading ✨ NEW!

🎮 **Run the demo:**
```bash
cargo run --example full_transaction_demo --features wallet,crypto_ethereum
```

---

## Features

- ✅ **Cross-platform**: Native (Windows, Linux, macOS) and WebAssembly
- 🔐 **Authentication**: Email/password, guest, social (Web3), and crypto wallet login
- 💰 **In-App Purchases**: Flexible payment processing (credit card, crypto, Telegram payments)
- 📊 **Analytics**: Event tracking and user behavior analytics
- 🏆 **Leaderboards**: Competitive rankings, tournaments, and reward systems ✨ MIGRATED!
- 📦 **Inventory**: Item management and virtual currency ✨ MIGRATED!
- 🛒 **Marketplace**: Player-to-player trading system ✨ MIGRATED!
- 🔗 **Crypto Wallets**: Ethereum and Solana integration (MetaMask, Phantom)
- 💼 **Wallet Management**: HD wallet creation, import, and secure storage
- 🎨 **NFT Support**: Metaplex NFT loading with full metadata ✨ NEW!

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
bevy = "0.17.2"
idos_game_sdk = { path = "../idos_game_sdk", features = ["all"] }

# For WASM builds
[target.'cfg(target_arch = "wasm32")'.dependencies]
idos_game_sdk = { path = "../idos_game_sdk", features = ["all"] }
```

### Feature Flags

```toml
[dependencies.idos_game_sdk]
path = "../idos_game_sdk"
features = [
    "auth",              # Authentication
    "analytics",         # Analytics
    "iap",              # In-App Purchases
    "leaderboard",      # Leaderboards
    "inventory",        # Inventory system
    "marketplace",      # Marketplace
    "crypto_ethereum",  # Ethereum wallet support
    "crypto_solana",    # Solana wallet support
    "wallet",           # In-game wallet management (HD wallets, BIP39/BIP44)
]
```

## Quick Start

```rust
use bevy::prelude::*;
use idos_game_sdk::{IdosGamesPlugin, IdosConfig};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(IdosGamesPlugin::new(IdosConfig {
            api_key: "your_api_key".to_string(),
            game_id: "your_game_id".to_string(),
            ..default()
        }))
        .add_systems(Startup, setup)
        .run();
}

fn setup(/* your systems */) {
    // Your game setup
}
```

## WASM Support

### Building for WebAssembly

1. **Install WASM target**:
```bash
rustup target add wasm32-unknown-unknown
```

2. **Install wasm-bindgen-cli**:
```bash
cargo install wasm-bindgen-cli
```

3. **Build your game**:
```bash
cargo build --release --target wasm32-unknown-unknown
```

4. **Generate WASM bindings**:
```bash
wasm-bindgen --out-dir ./out --target web \
    ./target/wasm32-unknown-unknown/release/your_game.wasm
```

5. **Create an HTML file** (`index.html`):
```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Your Game</title>
    <style>
        body { margin: 0; overflow: hidden; }
        canvas { width: 100%; height: 100vh; display: block; }
    </style>
</head>
<body>
    <script type="module">
        import init from './out/your_game.js';
        init();
    </script>
</body>
</html>
```

6. **Serve with a local server**:
```bash
# Using Python
python3 -m http.server 8000

# Or using `basic-http-server`
cargo install basic-http-server
basic-http-server .
```

### Optimizing WASM Build

Add to your `Cargo.toml`:

```toml
[profile.release]
opt-level = 'z'     # Optimize for size
lto = true          # Enable Link Time Optimization
codegen-units = 1   # Better optimization
panic = 'abort'     # Smaller binary
strip = true        # Strip symbols
```

Then optimize the WASM file:

```bash
# Install wasm-opt (from binaryen)
cargo install wasm-opt

# Optimize
wasm-opt -Oz -o optimized.wasm your_game_bg.wasm
```

## Usage Examples

### Authentication

```rust
use bevy::prelude::*;
use idos_game_sdk::auth::{AuthHandler, AuthEvent};

fn login_system(
    auth: Res<AuthHandler>,
    mut auth_events: EventWriter<AuthEvent>,
) {
    // Login with email/password
    #[cfg(target_arch = "wasm32")]
    {
        let auth = auth.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match auth.login(
                "user@example.com".to_string(),
                "password".to_string()
            ).await {
                Ok(response) => {
                    info!("Logged in as: {}", response.user.username);
                }
                Err(e) => error!("Login failed: {}", e),
            }
        });
    }
}
```

### Analytics

```rust
use idos_game_sdk::analytics::AnalyticsHandler;
use std::collections::HashMap;

fn track_level_complete(analytics: Res<AnalyticsHandler>) {
    let mut props = HashMap::new();
    props.insert("level".to_string(), serde_json::json!(5));
    props.insert("score".to_string(), serde_json::json!(1000));
    
    #[cfg(target_arch = "wasm32")]
    {
        let analytics = analytics.clone();
        wasm_bindgen_futures::spawn_local(async move {
            analytics.track_event("level_complete", props).await.ok();
        });
    }
}
```

### In-App Purchases

```rust
use idos_game_sdk::iap::{IapHandler, PaymentMethod};

fn purchase_item(iap: Res<IapHandler>) {
    #[cfg(target_arch = "wasm32")]
    {
        let iap = iap.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match iap.purchase(
                "premium_currency_100".to_string(),
                PaymentMethod::CreditCard,
            ).await {
                Ok(response) => {
                    info!("Purchase successful: {:?}", response);
                }
                Err(e) => error!("Purchase failed: {}", e),
            }
        });
    }
}
```

### Ethereum Wallet Integration

**Setup (in main.rs):**
```rust
use idos_game_sdk::crypto_ethereum::{EthereumPlugin, BlockchainSettings};

fn main() {
    // Configure blockchain settings
    let mut eth_settings = BlockchainSettings::default();
    eth_settings.rpc_url = "https://mainnet.infura.io/v3/YOUR_KEY".to_string();
    eth_settings.chain_id = 1; // Ethereum mainnet
    eth_settings.platform_pool_contract_address = "0x...".to_string();
    eth_settings.gas_price_gwei = 20.0;
    
    // Add token contracts
    eth_settings.token_contract_addresses.insert(
        "USDC".to_string(),
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
    );

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(IdosGamesPlugin::new(IdosConfig { ... }))
        .add_plugins(EthereumPlugin::new(eth_settings)) // Add Ethereum plugin
        .run();
}
```

**Usage:**
```rust
use idos_game_sdk::crypto_ethereum::EthereumHandler;

fn check_balance(ethereum: Res<EthereumHandler>) {
    let wallet_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb";
    
    #[cfg(target_arch = "wasm32")]
    {
        let eth = ethereum.clone();
        let addr = wallet_address.to_string();
        wasm_bindgen_futures::spawn_local(async move {
            match eth.get_native_balance(&addr).await {
                Ok(balance) => info!("ETH balance (wei): {}", balance),
                Err(e) => error!("Failed to get balance: {}", e),
            }
        });
    }
}
```

**Features:**
- ✅ Works on both Native and WASM (WebGL)
- ✅ MetaMask integration for WASM builds
- ✅ Native token balances (ETH, BNB, MATIC, etc.)
- ✅ ERC20 token balances and allowances
- ✅ Transaction submission and monitoring
- ✅ Gas estimation and sufficiency checks

### Solana Wallet Integration

**Setup (in main.rs):**
```rust
use idos_game_sdk::crypto_solana::{SolanaPlugin, SolanaSettings, SolanaCluster};

fn main() {
    // Configure Solana settings
    let solana_settings = SolanaSettings {
        cluster: SolanaCluster::Devnet,
        rpc_url: "https://api.devnet.solana.com".to_string(),
        ws_url: Some("wss://api.devnet.solana.com".to_string()),
        program_id: "YourProgramIdHere".to_string(),
    };

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(IdosGamesPlugin::new(IdosConfig { ... }))
        .add_plugins(SolanaPlugin::new(solana_settings)) // Add Solana plugin
        .run();
}
```

**Usage:**
```rust
use idos_game_sdk::crypto_solana::SolanaHandler;

fn check_balance(solana: Res<SolanaHandler>) {
    let wallet_address = "11111111111111111111111111111111";
    
    #[cfg(target_arch = "wasm32")]
    {
        let sol = solana.clone();
        let addr = wallet_address.to_string();
        wasm_bindgen_futures::spawn_local(async move {
            match sol.get_balance(&addr).await {
                Ok(lamports) => {
                    let sol_amount = SolanaHandler::lamports_to_sol(lamports);
                    info!("SOL balance: {} ({} lamports)", sol_amount, lamports);
                }
                Err(e) => error!("Failed to get balance: {}", e),
            }
        });
    }
}
```

**Features:**
- ✅ Works on both Native and WASM (WebGL)
- ✅ Phantom/Solflare wallet integration for WASM builds
- ✅ SOL balance checking
- ✅ SPL token balances
- ✅ Platform pool deposit/withdrawal
- ✅ Transaction status monitoring
- ✅ Backend signature requests
- ✅ **Metaplex NFT support** (load and display NFTs) ✨ NEW!

**NFT Loading (Metaplex):**
```rust
use idos_game_sdk::crypto_solana::SolanaHandler;

fn load_nfts(solana: Res<SolanaHandler>) {
    let wallet_address = "YourWalletAddress";
    
    #[cfg(target_arch = "wasm32")]
    {
        let sol = solana.clone();
        let addr = wallet_address.to_string();
        wasm_bindgen_futures::spawn_local(async move {
            match sol.load_nfts(&addr).await {
                Ok(result) => {
                    info!("Found {} NFTs", result.count);
                    for nft in result.nfts {
                        info!("NFT: {} - {}", nft.metadata.name, nft.metadata.mint);
                        
                        // Off-chain metadata (IPFS/Arweave)
                        if let Some(json) = nft.json_metadata {
                            info!("Image: {:?}", json.image);
                            if let Some(attributes) = json.attributes {
                                for attr in attributes {
                                    info!("  {}: {}", attr.trait_type, attr.value);
                                }
                            }
                        }
                    }
                }
                Err(e) => error!("Failed to load NFTs: {}", e),
            }
        });
    }
}
```

**NFT Features:**
- ✅ Load all NFTs owned by wallet
- ✅ Parse Metaplex metadata (on-chain)
- ✅ Fetch JSON metadata (IPFS/Arweave)
- ✅ Creator verification & royalties
- ✅ Collection support
- ✅ Attribute/trait parsing
- ✅ WASM compatible

### In-Game Wallet Management

**Setup:**
```rust
use idos_game_sdk::wallet::{WalletManager, BlockchainNetwork};

fn setup(mut commands: Commands) {
    // Initialize wallet manager
    let wallet_manager = WalletManager::new(
        "user_id_from_auth".to_string(),
        BlockchainNetwork::Ethereum, // or Solana
    );
    commands.insert_resource(wallet_manager);
}
```

**Create New Wallet:**
```rust
fn create_wallet(mut wallet_manager: ResMut<WalletManager>) {
    let password = "user_password"; // From UI input
    
    match wallet_manager.create_wallet(password, 12) {
        Ok(result) => {
            info!("Wallet created: {}", result.wallet_info.address);
            info!("Seed phrase: {}", result.seed_phrase); // Show to user ONCE
        }
        Err(e) => error!("Failed: {}", e),
    }
}
```

**Import Wallet:**
```rust
use idos_game_sdk::wallet::ImportSource;

fn import_wallet(mut wallet_manager: ResMut<WalletManager>) {
    let password = "user_password";
    let seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    
    match wallet_manager.import_wallet(
        ImportSource::SeedPhrase(seed.to_string()),
        password
    ) {
        Ok(wallet_info) => info!("Imported: {}", wallet_info.address),
        Err(e) => error!("Failed: {}", e),
    }
}
```

**Features:**
- ✅ **Full Unity SDK parity** - Same API, same behavior
- ✅ BIP39 mnemonic generation (12/24 words)
- ✅ BIP44 key derivation (Ethereum: m/44'/60'/0'/0/0, Solana: m/44'/501'/0'/0')
- ✅ Password-protected encryption (matches Unity's PrivateKeyManager)
- ✅ Persistent storage (localStorage on WASM, files on native)
- ✅ Import from seed phrase or private key
- ✅ Works on both Ethereum and Solana
- ✅ WASM compatible
- ✅ No browser extension required

## Platform-Specific Features

### Web (WASM) Only
- Social login (OAuth)
- Web3 wallet integration (MetaMask, Phantom)
- Browser local storage
- Web-based payment gateways

### Native Only
- Native wallet apps
- File-based storage
- System notifications

## Development

### Running Tests
```bash
cargo test
```

### Running Examples

**Basic Usage:**
```bash
cargo run --example basic_usage --features auth,analytics
```

**Ethereum Wallet Integration:**
```bash
# Native build
cargo run --example ethereum_wallet --features crypto_ethereum

# WASM build
cargo build --example ethereum_wallet --target wasm32-unknown-unknown --features crypto_ethereum
```

The Ethereum example demonstrates:
- Configuring blockchain settings (RPC URL, chain ID, contract addresses)
- Adding the EthereumPlugin to your Bevy app
- Checking native token balances (ETH, MATIC, BNB, etc.)
- Checking ERC20 token balances
- Checking token allowances
- MetaMask integration for WASM builds
- Gas sufficiency checks

**Solana Wallet Integration:**
```bash
# Native build
cargo run --example solana_wallet --features crypto_solana

# WASM build
cargo build --example solana_wallet --target wasm32-unknown-unknown --features crypto_solana
```

The Solana example demonstrates:
- Configuring Solana cluster and RPC settings
- Adding the SolanaPlugin to your Bevy app
- Connecting to Phantom/Solflare wallet
- Checking SOL balances
- Checking SPL token balances
- Requesting withdrawal signatures from backend
- Transaction status monitoring

**Solana NFT Loading (Metaplex):**
```bash
# Native build
cargo run --example solana_nft_loading --features crypto_solana

# WASM build
cargo build --example solana_nft_loading --target wasm32-unknown-unknown --features crypto_solana
```

The NFT example demonstrates:
- Loading all NFTs owned by a wallet
- Parsing Metaplex Token Metadata
- Fetching off-chain JSON metadata (IPFS/Arweave)
- Displaying NFT attributes and properties
- Creator verification and royalty info
- Collection support
- **Full Unity SDK parity**

**In-Game Wallet Management:**
```bash
# Native build
cargo run --example wallet_management --features wallet

# WASM build  
cargo build --example wallet_management --target wasm32-unknown-unknown --features wallet
```

The Wallet Management example demonstrates:
- Creating HD wallets with BIP39 mnemonics (12/24 words)
- Importing wallets from seed phrases or private keys
- Password-protected wallet storage
- Support for both Ethereum and Solana
- Login/logout/disconnect functionality
- **Full feature parity with Unity SDK's WalletManager**

### Building Documentation
```bash
cargo doc --open --features all
```

## License

MIT License - see LICENSE file for details.

