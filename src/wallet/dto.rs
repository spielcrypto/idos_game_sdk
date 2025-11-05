/// Data Transfer Objects for Wallet Management
use serde::{Deserialize, Serialize};

/// Blockchain network type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockchainNetwork {
    Ethereum,
    Solana,
}

impl BlockchainNetwork {
    pub fn as_str(&self) -> &str {
        match self {
            BlockchainNetwork::Ethereum => "Ethereum",
            BlockchainNetwork::Solana => "Solana",
        }
    }
}

/// Wallet information (matches Unity SDK WalletManager)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub address: String,
    pub network: BlockchainNetwork,
    #[serde(skip_serializing)]
    pub private_key: Option<String>, // Never serialize
    #[serde(skip_serializing)]
    pub seed_phrase: Option<String>, // Never serialize
}

/// Encrypted wallet data stored in PlayerPrefs/localStorage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EncryptedWalletData {
    pub encrypted_private_key: String,
    pub encrypted_seed_phrase: Option<String>,
    pub address: String,
    pub network: String,
}

/// Wallet creation result
#[derive(Debug, Clone)]
pub struct WalletCreationResult {
    pub wallet_info: WalletInfo,
    pub seed_phrase: String, // Return to show user once
}

/// Wallet import source
#[derive(Debug, Clone)]
pub enum ImportSource {
    SeedPhrase(String),
    PrivateKey(String),
}
