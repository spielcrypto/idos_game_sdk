/// Ethereum wallet integration example for iDos Games SDK
///
/// This example demonstrates how to:
/// - Configure and add the EthereumPlugin
/// - Check wallet balances (native tokens and ERC20)
/// - Interact with smart contracts
/// - Handle MetaMask on WASM
use bevy::prelude::*;
use idos_game_sdk::{IdosConfig, IdosGamesPlugin};

#[cfg(feature = "crypto_ethereum")]
use idos_game_sdk::crypto_ethereum::{BlockchainSettings, EthereumHandler, EthereumPlugin};

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "iDos Games SDK - Ethereum Wallet Example".to_string(),
            resolution: (1024, 768).into(),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(IdosGamesPlugin::new(IdosConfig {
        api_key: "your_api_key_here".to_string(),
        game_id: "your_game_id_here".to_string(),
        debug: true,
        ..default()
    }));

    // Add Ethereum plugin with blockchain settings
    #[cfg(feature = "crypto_ethereum")]
    {
        let mut eth_settings = BlockchainSettings::default();

        // Configure for Ethereum mainnet (change for testnet/other chains)
        eth_settings.rpc_url = "https://mainnet.infura.io/v3/YOUR_INFURA_KEY".to_string();
        eth_settings.chain_id = 1; // 1 = Ethereum mainnet, 5 = Goerli testnet
        eth_settings.platform_pool_contract_address =
            "0x0000000000000000000000000000000000000000".to_string(); // Replace with actual address
        eth_settings.nft_contract_address =
            "0x0000000000000000000000000000000000000000".to_string(); // Replace with actual address
        eth_settings.gas_price_gwei = 20.0;

        // Add token contract addresses
        eth_settings.token_contract_addresses.insert(
            "USDC".to_string(),
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(), // USDC on mainnet
        );
        eth_settings.token_contract_addresses.insert(
            "USDT".to_string(),
            "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(), // USDT on mainnet
        );

        app.add_plugins(EthereumPlugin::new(eth_settings));
    }

    app.add_systems(Startup, setup)
        .add_systems(Update, handle_input)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn a camera
    commands.spawn(Camera2d);

    // Spawn UI instructions
    commands.spawn((
        Text::new(
            "iDos Games SDK - Ethereum Wallet Example\n\n\
            Press 'B' to check native balance (ETH)\n\
            Press 'T' to check token balance (USDC)\n\
            Press 'A' to check allowance\n\
            Press 'M' to check MetaMask availability (WASM only)\n\
            Press 'G' to check gas sufficiency\n\n\
            Wallet Address: Connect your wallet first",
        ),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(50.0),
            left: Val::Px(50.0),
            ..default()
        },
    ));

    info!("Ethereum Wallet Example started!");
    info!("Note: Set your wallet address and configure RPC URL in the code");
}

#[cfg(feature = "crypto_ethereum")]
fn handle_input(keyboard: Res<ButtonInput<KeyCode>>, ethereum: Option<Res<EthereumHandler>>) {
    if let Some(eth) = ethereum {
        // Example wallet address - replace with actual wallet address
        let wallet_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"; // Example address

        // Check native balance (ETH, BNB, MATIC, etc.)
        if keyboard.just_pressed(KeyCode::KeyB) {
            info!("Checking native token balance...");

            #[cfg(target_arch = "wasm32")]
            {
                let eth_clone = eth.clone();
                let addr = wallet_address.to_string();
                wasm_bindgen_futures::spawn_local(async move {
                    match eth_clone.get_native_balance(&addr).await {
                        Ok(balance) => {
                            info!("Native balance (wei): {}", balance);
                            // Convert from wei to ETH: divide by 10^18
                        }
                        Err(e) => error!("Failed to get balance: {}", e),
                    }
                });
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                let eth_clone = eth.clone();
                let addr = wallet_address.to_string();
                tokio::spawn(async move {
                    match eth_clone.get_native_balance(&addr).await {
                        Ok(balance) => {
                            info!("Native balance (wei): {}", balance);
                        }
                        Err(e) => error!("Failed to get balance: {}", e),
                    }
                });
            }
        }

        // Check ERC20 token balance
        if keyboard.just_pressed(KeyCode::KeyT) {
            info!("Checking USDC token balance...");

            let token_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"; // USDC mainnet

            #[cfg(target_arch = "wasm32")]
            {
                let eth_clone = eth.clone();
                let addr = wallet_address.to_string();
                let token = token_address.to_string();
                wasm_bindgen_futures::spawn_local(async move {
                    match eth_clone.get_erc20_balance(&addr, &token).await {
                        Ok(balance) => {
                            info!("USDC balance (smallest unit): {}", balance);
                            // USDC has 6 decimals, divide by 10^6
                        }
                        Err(e) => error!("Failed to get token balance: {}", e),
                    }
                });
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                let eth_clone = eth.clone();
                let addr = wallet_address.to_string();
                let token = token_address.to_string();
                tokio::spawn(async move {
                    match eth_clone.get_erc20_balance(&addr, &token).await {
                        Ok(balance) => {
                            info!("USDC balance (smallest unit): {}", balance);
                        }
                        Err(e) => error!("Failed to get token balance: {}", e),
                    }
                });
            }
        }

        // Check ERC20 allowance
        if keyboard.just_pressed(KeyCode::KeyA) {
            info!("Checking token allowance...");

            let token_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"; // USDC
            let spender_address = "0x0000000000000000000000000000000000000000"; // Replace with contract

            #[cfg(target_arch = "wasm32")]
            {
                let eth_clone = eth.clone();
                let owner = wallet_address.to_string();
                let token = token_address.to_string();
                let spender = spender_address.to_string();
                wasm_bindgen_futures::spawn_local(async move {
                    match eth_clone
                        .get_erc20_allowance(&token, &owner, &spender)
                        .await
                    {
                        Ok(allowance) => {
                            info!("Token allowance: {}", allowance);
                        }
                        Err(e) => error!("Failed to get allowance: {}", e),
                    }
                });
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                let eth_clone = eth.clone();
                let owner = wallet_address.to_string();
                let token = token_address.to_string();
                let spender = spender_address.to_string();
                tokio::spawn(async move {
                    match eth_clone
                        .get_erc20_allowance(&token, &owner, &spender)
                        .await
                    {
                        Ok(allowance) => {
                            info!("Token allowance: {}", allowance);
                        }
                        Err(e) => error!("Failed to get allowance: {}", e),
                    }
                });
            }
        }

        // Check MetaMask availability (WASM only)
        #[cfg(target_arch = "wasm32")]
        if keyboard.just_pressed(KeyCode::KeyM) {
            if eth.is_metamask_available() {
                info!("✓ MetaMask is available!");
            } else {
                warn!("✗ MetaMask not detected. Please install MetaMask extension.");
            }
        }

        // Check gas sufficiency
        if keyboard.just_pressed(KeyCode::KeyG) {
            info!("Checking if wallet has sufficient gas...");

            #[cfg(target_arch = "wasm32")]
            {
                let eth_clone = eth.clone();
                let addr = wallet_address.to_string();
                wasm_bindgen_futures::spawn_local(async move {
                    match eth_clone.has_sufficient_gas(&addr, 100000).await {
                        Ok(has_gas) => {
                            if has_gas {
                                info!("✓ Wallet has sufficient gas for transaction");
                            } else {
                                warn!("✗ Insufficient gas! Please add more native tokens");
                            }
                        }
                        Err(e) => error!("Failed to check gas: {}", e),
                    }
                });
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                let eth_clone = eth.clone();
                let addr = wallet_address.to_string();
                tokio::spawn(async move {
                    match eth_clone.has_sufficient_gas(&addr, 100000).await {
                        Ok(has_gas) => {
                            if has_gas {
                                info!("✓ Wallet has sufficient gas for transaction");
                            } else {
                                warn!("✗ Insufficient gas! Please add more native tokens");
                            }
                        }
                        Err(e) => error!("Failed to check gas: {}", e),
                    }
                });
            }
        }
    } else {
        if keyboard.just_pressed(KeyCode::KeyB)
            || keyboard.just_pressed(KeyCode::KeyT)
            || keyboard.just_pressed(KeyCode::KeyA)
            || keyboard.just_pressed(KeyCode::KeyM)
            || keyboard.just_pressed(KeyCode::KeyG)
        {
            warn!("Ethereum plugin not loaded! Enable 'crypto_ethereum' feature and configure plugin.");
        }
    }
}

#[cfg(not(feature = "crypto_ethereum"))]
fn handle_input(keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::KeyB)
        || keyboard.just_pressed(KeyCode::KeyT)
        || keyboard.just_pressed(KeyCode::KeyA)
        || keyboard.just_pressed(KeyCode::KeyM)
        || keyboard.just_pressed(KeyCode::KeyG)
    {
        warn!("Ethereum feature not enabled! Run with: cargo run --example ethereum_wallet --features crypto_ethereum");
    }
}
