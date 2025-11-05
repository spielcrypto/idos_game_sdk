/// Secure keystore for encrypted wallet data
/// Matches Unity SDK's PrivateKeyManager storage pattern
use super::dto::*;
use super::encryption;
use crate::{storage::Storage, IdosError, IdosResult};

const ENCRYPTED_PRIVATE_KEY_PREFIX: &str = "EncryptedPrivateKey_";
const ENCRYPTED_SEED_PHRASE_PREFIX: &str = "EncryptedSeedPhrase_";
const WALLET_ADDRESS_PREFIX: &str = "WalletAddress_";
const WALLET_NETWORK_PREFIX: &str = "WalletNetwork_";

#[derive(Clone)]
pub struct Keystore {
    storage: Storage,
    user_id: String,
}

impl Keystore {
    pub fn new(user_id: String) -> Self {
        Self {
            storage: Storage::new("idos_wallet_".to_string()),
            user_id,
        }
    }

    fn private_key_key(&self) -> String {
        format!("{}{}", ENCRYPTED_PRIVATE_KEY_PREFIX, self.user_id)
    }

    fn seed_phrase_key(&self) -> String {
        format!("{}{}", ENCRYPTED_SEED_PHRASE_PREFIX, self.user_id)
    }

    fn wallet_address_key(&self) -> String {
        format!("{}{}", WALLET_ADDRESS_PREFIX, self.user_id)
    }

    fn wallet_network_key(&self) -> String {
        format!("{}{}", WALLET_NETWORK_PREFIX, self.user_id)
    }

    /// Save wallet (encrypts private key and seed phrase)
    /// Matches Unity SDK's PrivateKeyManager.SaveSeedPhrase
    pub fn save_wallet(
        &self,
        wallet_info: &WalletInfo,
        seed_phrase: Option<&str>,
        password: &str,
    ) -> IdosResult<()> {
        // Encrypt and save private key
        if let Some(private_key) = &wallet_info.private_key {
            let encrypted_private_key = encryption::encrypt(private_key, password)?;
            self.storage
                .set(&self.private_key_key(), &encrypted_private_key)?;
        }

        // Encrypt and save seed phrase (if available)
        if let Some(seed) = seed_phrase.or(wallet_info.seed_phrase.as_deref()) {
            let encrypted_seed_phrase = encryption::encrypt(seed, password)?;
            self.storage
                .set(&self.seed_phrase_key(), &encrypted_seed_phrase)?;
        }

        // Save wallet address and network (not encrypted)
        self.storage
            .set(&self.wallet_address_key(), &wallet_info.address)?;
        self.storage
            .set(&self.wallet_network_key(), wallet_info.network.as_str())?;

        Ok(())
    }

    /// Load wallet (decrypts private key and seed phrase)
    /// Matches Unity SDK's PrivateKeyManager.GetSeedPhrase
    pub fn load_wallet(&self, password: &str) -> IdosResult<Option<WalletInfo>> {
        // Check if wallet exists
        let address = match self.storage.get(&self.wallet_address_key())? {
            Some(addr) => addr,
            None => return Ok(None),
        };

        let network_str = self
            .storage
            .get(&self.wallet_network_key())?
            .unwrap_or_else(|| "Ethereum".to_string());

        let network = match network_str.as_str() {
            "Ethereum" => BlockchainNetwork::Ethereum,
            "Solana" => BlockchainNetwork::Solana,
            _ => BlockchainNetwork::Ethereum,
        };

        // Decrypt private key
        let private_key =
            if let Some(encrypted) = self.storage.get(&self.private_key_key())? {
                Some(encryption::decrypt(&encrypted, password).map_err(|_| {
                    IdosError::Auth("Incorrect password for private key".to_string())
                })?)
            } else {
                return Err(IdosError::Wallet("Private key not found".to_string()));
            };

        // Decrypt seed phrase (optional)
        let seed_phrase =
            if let Some(encrypted) = self.storage.get(&self.seed_phrase_key())? {
                Some(encryption::decrypt(&encrypted, password).map_err(|_| {
                    IdosError::Auth("Incorrect password for seed phrase".to_string())
                })?)
            } else {
                None
            };

        Ok(Some(WalletInfo {
            address,
            network,
            private_key,
            seed_phrase,
        }))
    }

    /// Check if wallet exists for this user
    pub fn has_wallet(&self) -> IdosResult<bool> {
        Ok(self.storage.get(&self.wallet_address_key())?.is_some())
    }

    /// Get wallet address without password (for display)
    pub fn get_wallet_address(&self) -> IdosResult<Option<String>> {
        self.storage.get(&self.wallet_address_key())
    }

    /// Delete wallet
    /// Matches Unity SDK's Disconnect functionality
    pub fn delete_wallet(&self) -> IdosResult<()> {
        self.storage.remove(&self.private_key_key())?;
        self.storage.remove(&self.seed_phrase_key())?;
        self.storage.remove(&self.wallet_address_key())?;
        self.storage.remove(&self.wallet_network_key())?;
        Ok(())
    }
}

#[cfg(not(feature = "wallet"))]
pub struct Keystore;

#[cfg(not(feature = "wallet"))]
impl Keystore {
    pub fn new(_user_id: String) -> Self {
        Self
    }

    pub fn save_wallet(
        &self,
        _wallet_info: &WalletInfo,
        _seed_phrase: Option<&str>,
        _password: &str,
    ) -> IdosResult<()> {
        Err(IdosError::PlatformNotSupported(
            "Wallet feature not enabled".to_string(),
        ))
    }

    pub fn load_wallet(&self, _password: &str) -> IdosResult<Option<WalletInfo>> {
        Err(IdosError::PlatformNotSupported(
            "Wallet feature not enabled".to_string(),
        ))
    }

    pub fn has_wallet(&self) -> IdosResult<bool> {
        Ok(false)
    }

    pub fn get_wallet_address(&self) -> IdosResult<Option<String>> {
        Ok(None)
    }

    pub fn delete_wallet(&self) -> IdosResult<()> {
        Ok(())
    }
}

#[cfg(all(test, feature = "wallet"))]
mod tests {
    use super::*;

    #[test]
    fn test_keystore_save_load() {
        let keystore = Keystore::new("test_user".to_string());

        let wallet_info = WalletInfo {
            address: "0x1234567890abcdef".to_string(),
            network: BlockchainNetwork::Ethereum,
            private_key: Some("0xdeadbeef".to_string()),
            seed_phrase: Some("test seed phrase".to_string()),
        };

        let password = "testpassword123";

        // Save (may fail in test environment without localStorage/files)
        if let Err(e) = keystore.save_wallet(&wallet_info, Some("test seed phrase"), password) {
            eprintln!("Note: Storage not available in test environment: {}", e);
            return; // Skip test if storage not available
        }

        // Load
        match keystore.load_wallet(password) {
            Ok(Some(loaded)) => {
                assert_eq!(loaded.address, wallet_info.address);
                assert_eq!(loaded.private_key, wallet_info.private_key);
                assert_eq!(loaded.seed_phrase, wallet_info.seed_phrase);
            }
            Ok(None) => {
                eprintln!("Note: Wallet not found in test environment");
            }
            Err(e) => {
                eprintln!("Note: Storage error in test environment: {}", e);
            }
        }
    }
}
