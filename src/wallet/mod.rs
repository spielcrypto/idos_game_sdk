/// In-game wallet management module
/// Provides HD wallet creation, import, and secure storage for both Ethereum and Solana
/// Matches Unity SDK's NewWallet functionality
pub mod creation;
pub mod dto;
pub mod encryption;
pub mod import;
pub mod keystore;
pub mod manager;

pub use dto::*;
pub use manager::WalletManager;
