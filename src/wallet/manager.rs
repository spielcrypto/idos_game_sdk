use super::dto::*;
/// Wallet Manager - Main interface for wallet operations
/// Matches Unity SDK's WalletManager behavior
use super::{creation, import, keystore::Keystore};
use crate::{IdosError, IdosResult};
use bevy::prelude::*;

/// Wallet Manager Resource
/// Manages wallet state and operations for both Ethereum and Solana
/// Matches Unity SDK's WalletManager.cs functionality
#[derive(Resource, Clone)]
pub struct WalletManager {
    keystore: Keystore,
    current_wallet: Option<WalletInfo>,
    current_network: BlockchainNetwork,
}

impl WalletManager {
    pub fn new(user_id: String, default_network: BlockchainNetwork) -> Self {
        Self {
            keystore: Keystore::new(user_id),
            current_wallet: None,
            current_network: default_network,
        }
    }

    /// Get current wallet address
    /// Matches Unity SDK's WalletManager.WalletAddress
    pub fn wallet_address(&self) -> Option<String> {
        self.current_wallet.as_ref().map(|w| w.address.clone())
    }

    /// Get current private key (only when unlocked)
    /// Matches Unity SDK's WalletManager.PrivateKey
    pub fn private_key(&self) -> Option<String> {
        self.current_wallet
            .as_ref()
            .and_then(|w| w.private_key.clone())
    }

    /// Get current seed phrase (only when unlocked)
    /// Matches Unity SDK's WalletManager.SeedPhrase
    pub fn seed_phrase(&self) -> Option<String> {
        self.current_wallet
            .as_ref()
            .and_then(|w| w.seed_phrase.clone())
    }

    /// Check if wallet is connected/unlocked
    pub fn is_connected(&self) -> bool {
        self.current_wallet.is_some()
    }

    /// Get current network
    pub fn current_network(&self) -> BlockchainNetwork {
        self.current_network
    }

    /// Set current network
    pub fn set_network(&mut self, network: BlockchainNetwork) {
        self.current_network = network;
    }

    /// Create a new wallet with random mnemonic
    /// Matches Unity SDK's WalletCreationManager.CreateWallet
    pub fn create_wallet(
        &mut self,
        password: &str,
        word_count: usize,
    ) -> IdosResult<WalletCreationResult> {
        if password.len() < 6 {
            return Err(IdosError::InvalidInput(
                "Password must be at least 6 characters".to_string(),
            ));
        }

        // Generate new wallet
        let result = creation::generate_wallet(self.current_network, word_count)?;

        // Save encrypted wallet
        self.keystore
            .save_wallet(&result.wallet_info, Some(&result.seed_phrase), password)?;

        // Set as current wallet
        self.current_wallet = Some(result.wallet_info.clone());

        info!(
            "Created new {} wallet: {}",
            self.current_network.as_str(),
            result.wallet_info.address
        );

        Ok(result)
    }

    /// Import wallet from seed phrase or private key
    /// Matches Unity SDK's WalletImportManager.OnImportButtonClick
    pub fn import_wallet(
        &mut self,
        source: ImportSource,
        password: &str,
    ) -> IdosResult<WalletInfo> {
        if password.len() < 6 {
            return Err(IdosError::InvalidInput(
                "Password must be at least 6 characters".to_string(),
            ));
        }

        // Import wallet
        let wallet_info = import::import_wallet(source.clone(), self.current_network)?;

        // Extract seed phrase if it was from seed phrase import
        let seed_phrase = match source {
            ImportSource::SeedPhrase(ref phrase) => Some(phrase.as_str()),
            ImportSource::PrivateKey(_) => None,
        };

        // Save encrypted wallet
        self.keystore
            .save_wallet(&wallet_info, seed_phrase, password)?;

        // Set as current wallet
        self.current_wallet = Some(wallet_info.clone());

        info!(
            "Imported {} wallet: {}",
            self.current_network.as_str(),
            wallet_info.address
        );

        Ok(wallet_info)
    }

    /// Login to existing wallet with password
    /// Matches Unity SDK's InGameWallet.Login
    pub fn login(&mut self, password: &str) -> IdosResult<WalletInfo> {
        let wallet_info = self
            .keystore
            .load_wallet(password)?
            .ok_or_else(|| IdosError::Wallet("No wallet found".to_string()))?;

        self.current_wallet = Some(wallet_info.clone());
        self.current_network = wallet_info.network;

        info!("Logged into wallet: {}", wallet_info.address);

        Ok(wallet_info)
    }

    /// Logout (clear in-memory wallet data but keep encrypted storage)
    /// Matches Unity SDK's WalletManager.NulledPrivateKey
    pub fn logout(&mut self) {
        self.current_wallet = None;
        info!("Logged out from wallet");
    }

    /// Disconnect (delete wallet completely)
    /// Matches Unity SDK's WalletManager.Disconnect
    pub fn disconnect(&mut self) -> IdosResult<()> {
        self.keystore.delete_wallet()?;
        self.current_wallet = None;
        info!("Wallet disconnected and deleted");
        Ok(())
    }

    /// Check if a wallet exists in storage
    pub fn has_stored_wallet(&self) -> IdosResult<bool> {
        self.keystore.has_wallet()
    }

    /// Get stored wallet address without unlocking
    /// Matches Unity SDK's loading WalletAddress from PlayerPrefs
    pub fn get_stored_wallet_address(&self) -> IdosResult<Option<String>> {
        self.keystore.get_wallet_address()
    }

    /// Get truncated address for display (0x1234...5678)
    /// Matches Unity SDK's WalletManager.UpdateWalletAddress
    pub fn get_display_address(&self) -> Option<String> {
        self.wallet_address().map(|addr| {
            if addr.len() > 10 {
                format!("{}...{}", &addr[..6], &addr[addr.len() - 4..])
            } else {
                addr
            }
        })
    }

    /// Verify password is correct without loading full wallet
    pub fn verify_password(&self, password: &str) -> IdosResult<bool> {
        match self.keystore.load_wallet(password) {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(IdosError::Auth(_)) => Ok(false), // Wrong password
            Err(e) => Err(e),
        }
    }
}

impl Default for WalletManager {
    fn default() -> Self {
        Self::new("default_user".to_string(), BlockchainNetwork::Ethereum)
    }
}
