/// In-Game Wallet Management Example
/// Demonstrates wallet creation, import, and management - matches Unity SDK functionality
///
/// This example shows how to:
/// - Create new HD wallets with BIP39 mnemonics (12/24 words)
/// - Import wallets from seed phrases or private keys
/// - Securely store wallets with password encryption
/// - Support both Ethereum and Solana networks
/// - Match Unity SDK's WalletCreationManager and WalletImportManager behavior
use bevy::prelude::*;
use idos_game_sdk::{IdosConfig, IdosGamesPlugin};

#[cfg(feature = "wallet")]
use idos_game_sdk::wallet::{BlockchainNetwork, ImportSource, WalletManager};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "iDos Games SDK - Wallet Management Example".to_string(),
                resolution: (1280, 800).into(),
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
    // Spawn a camera
    commands.spawn(Camera2d);

    // Initialize wallet manager with default network (Ethereum)
    #[cfg(feature = "wallet")]
    {
        let wallet_manager = WalletManager::new(
            "user_123".to_string(), // User ID (from auth)
            BlockchainNetwork::Ethereum,
        );
        commands.insert_resource(wallet_manager);
    }

    // Spawn UI instructions
    commands.spawn((
        Text::new(
            "iDos Games SDK - Wallet Management Example\n\n\
            WALLET CREATION:\n\
            Press '1' - Create Ethereum wallet (12 words)\n\
            Press '2' - Create Ethereum wallet (24 words)\n\
            Press '3' - Create Solana wallet (12 words)\n\n\
            WALLET IMPORT:\n\
            Press 'I' - Import from seed phrase\n\
            Press 'P' - Import from private key\n\n\
            WALLET MANAGEMENT:\n\
            Press 'L' - Login to stored wallet\n\
            Press 'O' - Logout (clear keys from memory)\n\
            Press 'D' - Disconnect (delete wallet completely)\n\
            Press 'S' - Show wallet info\n\n\
            Note: Password hardcoded to '123456' for demo",
        ),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(30.0),
            left: Val::Px(30.0),
            ..default()
        },
    ));

    info!("Wallet Management Example started!");
    info!("Password: 123456");
}

#[cfg(feature = "wallet")]
fn handle_input(keyboard: Res<ButtonInput<KeyCode>>, mut wallet_manager: ResMut<WalletManager>) {
    let password = "123456"; // In real app, get from UI input

    // Create Ethereum wallet (12 words)
    if keyboard.just_pressed(KeyCode::Digit1) {
        info!("Creating new Ethereum wallet (12 words)...");
        wallet_manager.set_network(BlockchainNetwork::Ethereum);

        match wallet_manager.create_wallet(password, 12) {
            Ok(result) => {
                info!("✓ Wallet created successfully!");
                info!("  Address: {}", result.wallet_info.address);
                info!("  Network: Ethereum");
                info!("  Seed phrase (SAVE THIS): {}", result.seed_phrase);
                info!(
                    "  Private key: {}",
                    result.wallet_info.private_key.as_ref().unwrap()
                );
            }
            Err(e) => error!("Failed to create wallet: {}", e),
        }
    }

    // Create Ethereum wallet (24 words)
    if keyboard.just_pressed(KeyCode::Digit2) {
        info!("Creating new Ethereum wallet (24 words)...");
        wallet_manager.set_network(BlockchainNetwork::Ethereum);

        match wallet_manager.create_wallet(password, 24) {
            Ok(result) => {
                info!("✓ Wallet created successfully!");
                info!("  Address: {}", result.wallet_info.address);
                info!("  Seed phrase (24 words): {}", result.seed_phrase);
            }
            Err(e) => error!("Failed to create wallet: {}", e),
        }
    }

    // Create Solana wallet (12 words)
    if keyboard.just_pressed(KeyCode::Digit3) {
        info!("Creating new Solana wallet (12 words)...");
        wallet_manager.set_network(BlockchainNetwork::Solana);

        match wallet_manager.create_wallet(password, 12) {
            Ok(result) => {
                info!("✓ Wallet created successfully!");
                info!("  Address: {}", result.wallet_info.address);
                info!("  Network: Solana");
                info!("  Seed phrase: {}", result.seed_phrase);
                info!(
                    "  Private key (base58): {}",
                    result.wallet_info.private_key.as_ref().unwrap()
                );
            }
            Err(e) => error!("Failed to create wallet: {}", e),
        }
    }

    // Import from seed phrase
    if keyboard.just_pressed(KeyCode::KeyI) {
        info!("Importing wallet from seed phrase...");

        // Example seed phrase (replace with actual user input in real app)
        let seed_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

        wallet_manager.set_network(BlockchainNetwork::Ethereum);

        match wallet_manager
            .import_wallet(ImportSource::SeedPhrase(seed_phrase.to_string()), password)
        {
            Ok(wallet_info) => {
                info!("✓ Wallet imported successfully!");
                info!("  Address: {}", wallet_info.address);
                info!("  Network: {:?}", wallet_info.network);
            }
            Err(e) => error!("Failed to import wallet: {}", e),
        }
    }

    // Import from private key
    if keyboard.just_pressed(KeyCode::KeyP) {
        info!("Importing Ethereum wallet from private key...");

        // Example private key (replace with actual user input)
        let private_key = "0x4c0883a69102937d6231471b5dbb6204fe512961708279f8b1a3e79e5c8c4f8f";

        wallet_manager.set_network(BlockchainNetwork::Ethereum);

        match wallet_manager
            .import_wallet(ImportSource::PrivateKey(private_key.to_string()), password)
        {
            Ok(wallet_info) => {
                info!("✓ Wallet imported from private key!");
                info!("  Address: {}", wallet_info.address);
                warn!("  Note: No seed phrase when importing from private key");
            }
            Err(e) => error!("Failed to import wallet: {}", e),
        }
    }

    // Login to stored wallet
    if keyboard.just_pressed(KeyCode::KeyL) {
        info!("Logging into stored wallet...");

        match wallet_manager.login(password) {
            Ok(wallet_info) => {
                info!("✓ Logged in successfully!");
                info!("  Address: {}", wallet_info.address);
                info!("  Network: {:?}", wallet_info.network);
                info!("  Has private key: {}", wallet_info.private_key.is_some());
                info!("  Has seed phrase: {}", wallet_info.seed_phrase.is_some());
            }
            Err(e) => error!(
                "Failed to login: {} (No wallet stored or wrong password)",
                e
            ),
        }
    }

    // Logout (clear keys from memory)
    if keyboard.just_pressed(KeyCode::KeyO) {
        wallet_manager.logout();
        info!("✓ Logged out (wallet still stored securely)");
    }

    // Disconnect (delete wallet completely)
    if keyboard.just_pressed(KeyCode::KeyD) {
        match wallet_manager.disconnect() {
            Ok(_) => info!("✓ Wallet disconnected and deleted from storage"),
            Err(e) => error!("Failed to disconnect: {}", e),
        }
    }

    // Show wallet info
    if keyboard.just_pressed(KeyCode::KeyS) {
        if wallet_manager.is_connected() {
            info!("Current Wallet Info:");
            info!("  Connected: {}", wallet_manager.is_connected());
            info!("  Network: {:?}", wallet_manager.current_network());
            if let Some(address) = wallet_manager.wallet_address() {
                info!("  Address: {}", address);
                info!(
                    "  Display: {}",
                    wallet_manager.get_display_address().unwrap()
                );
            }
            if let Some(seed) = wallet_manager.seed_phrase() {
                warn!("  Seed phrase: {} (KEEP SECRET)", seed);
            }
            if let Some(key) = wallet_manager.private_key() {
                warn!("  Private key: {}... (KEEP SECRET)", &key[..10]);
            }
        } else {
            info!("No wallet connected. Create or login first.");
        }
    }
}

#[cfg(not(feature = "wallet"))]
fn handle_input(keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::Digit1)
        || keyboard.just_pressed(KeyCode::Digit2)
        || keyboard.just_pressed(KeyCode::Digit3)
        || keyboard.just_pressed(KeyCode::KeyI)
        || keyboard.just_pressed(KeyCode::KeyP)
        || keyboard.just_pressed(KeyCode::KeyL)
        || keyboard.just_pressed(KeyCode::KeyO)
        || keyboard.just_pressed(KeyCode::KeyD)
        || keyboard.just_pressed(KeyCode::KeyS)
    {
        warn!("Wallet feature not enabled! Run with: cargo run --example wallet_management --features wallet");
    }
}
