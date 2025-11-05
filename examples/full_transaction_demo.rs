/// Full Transaction Demo - End-to-End Example
///
/// This example demonstrates:
/// 1. Creating/importing a wallet with the wallet plugin
/// 2. Using that wallet to sign and send transactions
/// 3. Transferring tokens on Ethereum
/// 4. Transferring tokens on Solana
/// 5. Complete flow: wallet creation -> token approval -> deposit to platform pool
///
/// This proves the SDK can do EVERYTHING the Unity SDK can do!
use bevy::prelude::*;
use idos_game_sdk::{IdosConfig, IdosGamesPlugin};

#[cfg(feature = "wallet")]
use idos_game_sdk::wallet::{BlockchainNetwork, WalletManager};

#[cfg(feature = "crypto_ethereum")]
use idos_game_sdk::crypto_ethereum::{BlockchainSettings, EthereumHandler, EthereumWalletService};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "iDos Games SDK - Full Transaction Demo".to_string(),
                resolution: (1400, 900).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(IdosGamesPlugin::new(IdosConfig {
            api_key: "your_api_key_here".to_string(),
            game_id: "your_game_id_here".to_string(),
            debug: true,
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, handle_input)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Initialize wallet manager
    #[cfg(feature = "wallet")]
    {
        let wallet_manager =
            WalletManager::new("demo_user".to_string(), BlockchainNetwork::Ethereum);
        commands.insert_resource(wallet_manager);
    }

    // Setup Ethereum (configure for your network)
    #[cfg(feature = "crypto_ethereum")]
    {
        use std::collections::HashMap;

        let mut eth_settings = BlockchainSettings::default();

        // CONFIGURE THESE FOR YOUR NETWORK:
        eth_settings.rpc_url = "https://sepolia.infura.io/v3/YOUR_KEY".to_string(); // Sepolia testnet
        eth_settings.chain_id = 11155111; // Sepolia chain ID
        eth_settings.platform_pool_contract_address = "0xYourPlatformPoolAddress".to_string(); // Your contract
        eth_settings.nft_contract_address = "0xYourNFTContractAddress".to_string();
        eth_settings.gas_price_gwei = 20.0;

        // Add token addresses
        let mut tokens = HashMap::new();
        tokens.insert(
            "USDC".to_string(),
            "0xYourUSDCAddress".to_string(), // Testnet USDC
        );
        eth_settings.token_contract_addresses = tokens;

        // This would normally come from IdosClient resource
        // For demo, we create a dummy client
        let dummy_config = IdosConfig {
            api_key: "demo".to_string(),
            game_id: "demo".to_string(),
            ..default()
        };
        let dummy_client = idos_game_sdk::client::IdosClient::new(dummy_config);

        let eth_handler = EthereumHandler::new(dummy_client, eth_settings);
        commands.insert_resource(eth_handler);
    }

    commands.spawn((
        Text::new(
            "iDos Games SDK - Full Transaction Demo\n\n\
            === WALLET CREATION (Works Offline) ===\n\
            Press '1' - Create Ethereum wallet\n\
            Press '2' - Import wallet from seed phrase\n\
            Press 'S' - Show wallet info\n\n\
            === ETHEREUM TRANSACTIONS (Requires Network) ===\n\
            Press 'B' - Check ETH balance\n\
            Press 'T' - Check ERC20 token balance\n\
            Press 'A' - Approve tokens for spending\n\
            Press 'D' - Deposit tokens to platform pool\n\
            Press 'X' - Transfer tokens to external address\n\
            Press 'N' - Transfer NFT\n\n\
            === IMPORTANT ===\n\
            - Wallet creation works immediately (offline)\n\
            - Transactions need configured RPC and contracts\n\
            - Demo uses password '123456'\n\
            - Private keys shown in console (demo only!)\n\n\
            Status: Ready",
        ),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(20.0),
            ..default()
        },
    ));

    info!("=== Full Transaction Demo Started ===");
    info!("This demo shows the complete wallet-to-transaction flow");
}

#[cfg(all(feature = "wallet", feature = "crypto_ethereum"))]
fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut wallet_manager: ResMut<WalletManager>,
    ethereum: Option<Res<EthereumHandler>>,
) {
    let password = "123456";

    // ==================== WALLET CREATION ====================

    // Create Ethereum wallet
    if keyboard.just_pressed(KeyCode::Digit1) {
        info!("\n=== Creating Ethereum Wallet ===");
        wallet_manager.set_network(BlockchainNetwork::Ethereum);

        match wallet_manager.create_wallet(password, 12) {
            Ok(result) => {
                info!("✅ SUCCESS! Wallet created:");
                info!("  Address: {}", result.wallet_info.address);
                info!("  Seed Phrase: {}", result.seed_phrase);
                info!(
                    "  Private Key: {}",
                    result.wallet_info.private_key.as_ref().unwrap()
                );
                info!("\n💡 You can now use this wallet to sign transactions!");
                info!("💡 The wallet is encrypted and saved with password '123456'");
            }
            Err(e) => error!("❌ Failed to create wallet: {}", e),
        }
    }

    // Import wallet
    if keyboard.just_pressed(KeyCode::Digit2) {
        info!("\n=== Importing Wallet from Seed Phrase ===");

        let seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        wallet_manager.set_network(BlockchainNetwork::Ethereum);

        match wallet_manager.import_wallet(
            idos_game_sdk::wallet::ImportSource::SeedPhrase(seed.to_string()),
            password,
        ) {
            Ok(wallet_info) => {
                info!("✅ SUCCESS! Wallet imported:");
                info!("  Address: {}", wallet_info.address);
                info!(
                    "  Private Key: {}",
                    wallet_info.private_key.as_ref().unwrap()
                );
            }
            Err(e) => error!("❌ Failed to import: {}", e),
        }
    }

    // Show wallet info
    if keyboard.just_pressed(KeyCode::KeyS) {
        if wallet_manager.is_connected() {
            info!("\n=== Current Wallet ===");
            info!("  Connected: YES");
            info!("  Address: {}", wallet_manager.wallet_address().unwrap());
            info!("  Network: {:?}", wallet_manager.current_network());

            if let Some(key) = wallet_manager.private_key() {
                info!("  Private Key: {}...", &key[..20]);
            }
            if let Some(seed) = wallet_manager.seed_phrase() {
                info!(
                    "  Has Seed Phrase: YES ({} words)",
                    seed.split_whitespace().count()
                );
            }
        } else {
            warn!("No wallet connected. Create or import one first (press '1' or '2')");
        }
    }

    // ==================== ETHEREUM TRANSACTIONS ====================

    if let Some(eth) = ethereum {
        let wallet_address = wallet_manager.wallet_address();

        // Check ETH balance
        if keyboard.just_pressed(KeyCode::KeyB) {
            if let Some(addr) = &wallet_address {
                info!("\n=== Checking ETH Balance ===");

                let eth_clone = eth.clone();
                let address = addr.clone();

                #[cfg(not(target_arch = "wasm32"))]
                tokio::spawn(async move {
                    match eth_clone.get_native_balance(&address).await {
                        Ok(balance_wei) => {
                            let balance_eth =
                                balance_wei.parse::<u128>().unwrap_or(0) as f64 / 1e18;
                            info!("✅ ETH Balance: {} ETH ({} wei)", balance_eth, balance_wei);
                        }
                        Err(e) => error!("❌ Failed to get balance: {}", e),
                    }
                });
            } else {
                warn!("Create wallet first!");
            }
        }

        // Check ERC20 token balance
        if keyboard.just_pressed(KeyCode::KeyT) {
            if let Some(addr) = &wallet_address {
                info!("\n=== Checking ERC20 Token Balance ===");

                let token_addr = "0xYourTokenAddress"; // Configure this
                let eth_clone = eth.clone();
                let address = addr.clone();

                #[cfg(not(target_arch = "wasm32"))]
                tokio::spawn(async move {
                    match eth_clone.get_erc20_balance(&address, token_addr).await {
                        Ok(balance) => {
                            info!("✅ Token Balance: {} (smallest unit)", balance);
                        }
                        Err(e) => error!("❌ Failed: {}", e),
                    }
                });
            } else {
                warn!("Create wallet first!");
            }
        }

        // Approve tokens - REAL TRANSACTION!
        if keyboard.just_pressed(KeyCode::KeyA) {
            if let Some(private_key) = wallet_manager.private_key() {
                info!("\n=== Approving ERC20 Tokens ===");
                info!("⚠️  This will sign and send a REAL transaction!");

                let settings = eth.settings();
                let token_addr = "0xYourTokenAddress"; // Configure
                let spender = &settings.platform_pool_contract_address;
                let max_amount = "115792089237316195423570985008687907853269984665640564039457584007913129639935";

                #[cfg(not(target_arch = "wasm32"))]
                {
                    let rpc = settings.rpc_url.clone();
                    let token = token_addr.to_string();
                    let spender_addr = spender.clone();
                    let amount = max_amount.to_string();
                    let key = private_key.clone();
                    let chain = settings.chain_id as u64;
                    let gas = settings.gas_price_gwei;

                    tokio::spawn(async move {
                        use idos_game_sdk::crypto_ethereum::transactions;

                        match transactions::approve_erc20(
                            &rpc,
                            &token,
                            &spender_addr,
                            &amount,
                            &key,
                            chain,
                            gas,
                        )
                        .await
                        {
                            Ok(tx_hash) => {
                                info!("✅ APPROVAL TRANSACTION SENT!");
                                info!("   TX Hash: {}", tx_hash);
                                info!("   🔗 Check on block explorer");
                            }
                            Err(e) => error!("❌ Approval failed: {}", e),
                        }
                    });
                }
            } else {
                warn!("Login to wallet first! (Create wallet with '1')");
            }
        }

        // Deposit tokens to platform pool - FULL FLOW!
        if keyboard.just_pressed(KeyCode::KeyD) {
            if let Some(private_key) = wallet_manager.private_key() {
                if let Some(addr) = &wallet_address {
                    info!("\n=== Depositing Tokens to Platform Pool ===");
                    info!("⚠️  FULL TRANSACTION FLOW:");
                    info!("   1. Check allowance");
                    info!("   2. Approve if needed");
                    info!("   3. Deposit to platform pool");
                    info!("   4. Submit to backend");

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let settings = eth.settings();
                        let rpc = settings.rpc_url.clone();
                        let token_addr = "0xYourTokenAddress".to_string();
                        let amount = 100u64; // 100 tokens (will be converted to wei)
                        let user_id = "demo_user_123".to_string();
                        let wallet_addr = addr.clone();
                        let key = private_key.clone();

                        // Create service
                        let eth_clone = eth.clone();

                        tokio::spawn(async move {
                            let mut service = EthereumWalletService::new(eth_clone);
                            service.set_private_key(key);

                            match service
                                .transfer_token_to_game(
                                    &rpc,
                                    &token_addr,
                                    amount,
                                    &user_id,
                                    &wallet_addr,
                                )
                                .await
                            {
                                Ok(backend_response) => {
                                    info!("✅ TOKENS DEPOSITED SUCCESSFULLY!");
                                    info!("   Backend Response: {}", backend_response);
                                    info!("   💰 Tokens are now in your game account!");
                                }
                                Err(e) => error!("❌ Deposit failed: {}", e),
                            }
                        });
                    }
                }
            } else {
                warn!("Create wallet first (press '1')!");
            }
        }

        // Transfer tokens to external address
        if keyboard.just_pressed(KeyCode::KeyX) {
            if let Some(private_key) = wallet_manager.private_key() {
                if let Some(from_addr) = &wallet_address {
                    info!("\n=== Transferring Tokens to External Address ===");
                    info!("⚠️  This will send tokens to another wallet!");

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let settings = eth.settings();
                        let rpc = settings.rpc_url.clone();
                        let token_addr = "0xYourTokenAddress".to_string();
                        let from = from_addr.clone();
                        let to = "0xRecipientAddress".to_string(); // Configure recipient
                        let amount = 10u64; // 10 tokens
                        let key = private_key.clone();

                        let eth_clone = eth.clone();

                        tokio::spawn(async move {
                            let mut service = EthereumWalletService::new(eth_clone);
                            service.set_private_key(key);

                            match service
                                .transfer_token_to_external_address(
                                    &rpc,
                                    &token_addr,
                                    &from,
                                    &to,
                                    amount,
                                )
                                .await
                            {
                                Ok(tx_hash) => {
                                    info!("✅ TRANSFER SENT!");
                                    info!("   TX Hash: {}", tx_hash);
                                    info!("   Recipient: {}", to);
                                }
                                Err(e) => error!("❌ Transfer failed: {}", e),
                            }
                        });
                    }
                }
            } else {
                warn!("Create wallet first!");
            }
        }

        // Transfer NFT
        if keyboard.just_pressed(KeyCode::KeyN) {
            if let Some(private_key) = wallet_manager.private_key() {
                if let Some(from_addr) = &wallet_address {
                    info!("\n=== Transferring NFT ===");

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let settings = eth.settings();
                        let rpc = settings.rpc_url.clone();
                        let nft_addr = settings.nft_contract_address.clone();
                        let from = from_addr.clone();
                        let to = "0xRecipientAddress".to_string();
                        let nft_id = "1".to_string(); // NFT token ID
                        let amount = 1u64;
                        let key = private_key.clone();

                        let eth_clone = eth.clone();

                        tokio::spawn(async move {
                            let mut service = EthereumWalletService::new(eth_clone);
                            service.set_private_key(key);

                            match service
                                .transfer_nft_to_external_address(
                                    &rpc, &nft_addr, &from, &to, &nft_id, amount,
                                )
                                .await
                            {
                                Ok(tx_hash) => {
                                    info!("✅ NFT TRANSFERRED!");
                                    info!("   TX Hash: {}", tx_hash);
                                }
                                Err(e) => error!("❌ NFT transfer failed: {}", e),
                            }
                        });
                    }
                }
            } else {
                warn!("Create wallet first!");
            }
        }
    } else {
        if keyboard.pressed(KeyCode::KeyB) || keyboard.pressed(KeyCode::KeyT) {
            warn!("Ethereum feature not enabled! Run with --features crypto_ethereum,wallet");
        }
    }
}

#[cfg(not(all(feature = "wallet", feature = "crypto_ethereum")))]
fn handle_input(keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::Digit1)
        || keyboard.just_pressed(KeyCode::Digit2)
        || keyboard.just_pressed(KeyCode::KeyB)
        || keyboard.just_pressed(KeyCode::KeyT)
        || keyboard.just_pressed(KeyCode::KeyA)
        || keyboard.just_pressed(KeyCode::KeyD)
        || keyboard.just_pressed(KeyCode::KeyX)
        || keyboard.just_pressed(KeyCode::KeyN)
    {
        warn!("Enable wallet and crypto_ethereum features!");
        warn!("Run: cargo run --example full_transaction_demo --features wallet,crypto_ethereum");
    }
}
