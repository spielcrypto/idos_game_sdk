//! iDos Games SDK for Bevy
//!
//! A comprehensive game development SDK providing authentication, IAP, analytics,
//! crypto wallets, marketplace, and more for Bevy games.
//!
//! # Features
//! - **Authentication**: User login, registration, and session management
//! - **IAP**: In-app purchases and payment processing
//! - **Analytics**: Event tracking and user behavior analytics
//! - **Leaderboards**: Global and regional leaderboards
//! - **Inventory**: Item management and virtual currency
//! - **Marketplace**: Player-to-player trading
//! - **Crypto Wallets**: Ethereum and Solana wallet integration
//!
//! # Quick Start
//!
//! ```no_run
//! use bevy::prelude::*;
//! use idos_game_sdk::{IdosGamesPlugin, IdosConfig};
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(IdosGamesPlugin::new(IdosConfig {
//!             api_key: "your_api_key".to_string(),
//!             game_id: "your_game_id".to_string(),
//!             ..default()
//!         }))
//!         .run();
//! }
//! ```

pub mod client;
pub mod config;
pub mod error;
pub mod storage;

// Feature-gated modules
#[cfg(feature = "auth")]
pub mod auth;

#[cfg(feature = "analytics")]
pub mod analytics;

#[cfg(feature = "iap")]
pub mod iap;

#[cfg(feature = "leaderboard")]
pub mod leaderboard;

#[cfg(feature = "inventory")]
pub mod inventory;

#[cfg(feature = "marketplace")]
pub mod marketplace;

#[cfg(feature = "crypto_ethereum")]
pub mod crypto_ethereum;

#[cfg(feature = "crypto_solana")]
pub mod crypto_solana;

#[cfg(feature = "wallet")]
pub mod wallet;

// Re-exports
pub use analytics::AnalyticsPlugin;
pub use auth::auth_plugin::AuthPlugin;
pub use client::IdosClient;
pub use config::IdosConfig;
pub use error::{IdosError, IdosResult};
pub use iap::iap_plugin::IapPlugin;

use bevy::prelude::*;

/// Main plugin for iDos Games SDK
pub struct IdosGamesPlugin {
    config: IdosConfig,
}

impl IdosGamesPlugin {
    pub fn new(config: IdosConfig) -> Self {
        Self { config }
    }
}

impl Plugin for IdosGamesPlugin {
    fn build(&self, app: &mut App) {
        // Insert config as a resource
        app.insert_resource(self.config.clone());

        // Initialize client
        let client = IdosClient::new(self.config.clone());
        app.insert_resource(client);

        // Add feature-specific plugins
        #[cfg(feature = "auth")]
        app.add_plugins(AuthPlugin);

        #[cfg(feature = "analytics")]
        app.add_plugins(AnalyticsPlugin);

        #[cfg(feature = "iap")]
        app.add_plugins(IapPlugin);

        #[cfg(feature = "leaderboard")]
        app.add_plugins(leaderboard::LeaderboardPlugin);

        #[cfg(feature = "inventory")]
        app.add_plugins(inventory::InventoryPlugin);

        #[cfg(feature = "marketplace")]
        app.add_plugins(marketplace::MarketplacePlugin);

        // Note: Crypto wallet plugins (Ethereum, Solana) must be added manually
        // with their respective blockchain settings. They are not auto-added here.
    }
}
