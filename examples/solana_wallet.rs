/// Solana wallet integration example for iDos Games SDK
///
/// This example demonstrates how to:
/// - Configure and add the SolanaPlugin
/// - Connect to Phantom/Solflare wallet
/// - Check SOL and SPL token balances
/// - Request withdrawal signatures from backend
use bevy::prelude::*;
use idos_game_sdk::{IdosConfig, IdosGamesPlugin};

#[cfg(feature = "crypto_solana")]
use idos_game_sdk::crypto_solana::{SolanaCluster, SolanaHandler, SolanaPlugin, SolanaSettings};

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "iDos Games SDK - Solana Wallet Example".to_string(),
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

    // Add Solana plugin with blockchain settings
    #[cfg(feature = "crypto_solana")]
    {
        let solana_settings = SolanaSettings {
            cluster: SolanaCluster::Devnet,
            rpc_url: "https://api.devnet.solana.com".to_string(),
            ws_url: Some("wss://api.devnet.solana.com".to_string()),
            program_id: "YourProgramIdHere".to_string(), // Replace with your program ID
        };

        app.add_plugins(SolanaPlugin::new(solana_settings));
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
            "iDos Games SDK - Solana Wallet Example\n\n\
            Press 'C' to connect wallet (Phantom/Solflare)\n\
            Press 'B' to check SOL balance\n\
            Press 'T' to check SPL token balance\n\
            Press 'W' to check wallet availability\n\
            Press 'S' to request withdrawal signature\n\n\
            Note: WASM build required for wallet interaction",
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

    info!("Solana Wallet Example started!");
}

#[cfg(feature = "crypto_solana")]
fn handle_input(keyboard: Res<ButtonInput<KeyCode>>, solana: Option<Res<SolanaHandler>>) {
    if let Some(sol) = solana {
        // Example wallet address - will be replaced by connected wallet
        let wallet_address = "11111111111111111111111111111111"; // Example address

        // Connect wallet (WASM only)
        #[cfg(target_arch = "wasm32")]
        if keyboard.just_pressed(KeyCode::KeyC) {
            info!("Connecting to Solana wallet...");

            let sol_clone = sol.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match sol_clone.connect_wallet().await {
                    Ok(public_key) => {
                        info!("✓ Connected to wallet: {}", public_key);
                    }
                    Err(e) => error!("Failed to connect wallet: {}", e),
                }
            });
        }

        // Check wallet availability
        if keyboard.just_pressed(KeyCode::KeyW) {
            if sol.is_wallet_available() {
                info!("✓ Solana wallet (Phantom/Solflare) is available!");
            } else {
                warn!(
                    "✗ Solana wallet not detected. Please install Phantom or Solflare extension."
                );
            }
        }

        // Check SOL balance
        if keyboard.just_pressed(KeyCode::KeyB) {
            info!("Checking SOL balance...");

            #[cfg(target_arch = "wasm32")]
            {
                let sol_clone = sol.clone();
                let addr = wallet_address.to_string();
                wasm_bindgen_futures::spawn_local(async move {
                    match sol_clone.get_balance(&addr).await {
                        Ok(lamports) => {
                            let sol_amount = SolanaHandler::lamports_to_sol(lamports);
                            info!("SOL balance: {} ({} lamports)", sol_amount, lamports);
                        }
                        Err(e) => error!("Failed to get balance: {}", e),
                    }
                });
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                warn!("Balance check only available in WASM build");
            }
        }

        // Check SPL token balance
        if keyboard.just_pressed(KeyCode::KeyT) {
            info!("Checking SPL token balance...");

            // Example: USDC on devnet
            let token_mint = "Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr"; // USDC devnet

            #[cfg(target_arch = "wasm32")]
            {
                let sol_clone = sol.clone();
                let addr = wallet_address.to_string();
                let mint = token_mint.to_string();
                wasm_bindgen_futures::spawn_local(async move {
                    match sol_clone.get_token_balance(&addr, &mint).await {
                        Ok(token_amount) => {
                            info!(
                                "Token balance: {} (decimals: {})",
                                token_amount.ui_amount_string.unwrap_or_default(),
                                token_amount.decimals
                            );
                        }
                        Err(e) => error!("Failed to get token balance: {}", e),
                    }
                });
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                warn!("Token balance check only available in WASM build");
            }
        }

        // Request withdrawal signature
        if keyboard.just_pressed(KeyCode::KeyS) {
            info!("Requesting withdrawal signature from backend...");

            let token_mint = "Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr";
            let amount = 1000000u64; // 1 USDC (6 decimals)

            #[cfg(target_arch = "wasm32")]
            {
                let sol_clone = sol.clone();
                let mint = token_mint.to_string();
                let addr = wallet_address.to_string();
                wasm_bindgen_futures::spawn_local(async move {
                    match sol_clone
                        .get_withdrawal_signature(&mint, amount, &addr)
                        .await
                    {
                        Ok(signature_payload) => {
                            info!("✓ Got withdrawal signature!");
                            info!("  Nonce: {}", signature_payload.nonce);
                            info!("  Amount: {}", signature_payload.amount);
                            // Now you can use this to withdraw tokens
                        }
                        Err(e) => error!("Failed to get withdrawal signature: {}", e),
                    }
                });
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                warn!("Withdrawal signature request only available in WASM build");
            }
        }
    } else {
        if keyboard.just_pressed(KeyCode::KeyC)
            || keyboard.just_pressed(KeyCode::KeyB)
            || keyboard.just_pressed(KeyCode::KeyT)
            || keyboard.just_pressed(KeyCode::KeyW)
            || keyboard.just_pressed(KeyCode::KeyS)
        {
            warn!("Solana plugin not loaded! Enable 'crypto_solana' feature and configure plugin.");
        }
    }
}

#[cfg(not(feature = "crypto_solana"))]
fn handle_input(keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::KeyC)
        || keyboard.just_pressed(KeyCode::KeyB)
        || keyboard.just_pressed(KeyCode::KeyT)
        || keyboard.just_pressed(KeyCode::KeyW)
        || keyboard.just_pressed(KeyCode::KeyS)
    {
        warn!("Solana feature not enabled! Run with: cargo run --example solana_wallet --features crypto_solana");
    }
}
