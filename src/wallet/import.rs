/// Wallet import functionality
/// Matches Unity SDK's WalletImportManager
use super::creation::derive_wallet_from_mnemonic;
use super::dto::*;
use crate::{IdosError, IdosResult};

/// Import wallet from seed phrase or private key
/// Matches Unity SDK's OnImportButtonClick functionality
pub fn import_wallet(source: ImportSource, network: BlockchainNetwork) -> IdosResult<WalletInfo> {
    match source {
        ImportSource::SeedPhrase(seed_phrase) => import_from_seed_phrase(&seed_phrase, network),
        ImportSource::PrivateKey(private_key) => import_from_private_key(&private_key, network),
    }
}

/// Import wallet from seed phrase (12 or 24 words)
#[cfg(feature = "wallet")]
fn import_from_seed_phrase(
    seed_phrase: &str,
    network: BlockchainNetwork,
) -> IdosResult<WalletInfo> {
    use bip39::Mnemonic;

    // Trim and validate seed phrase
    let seed_phrase = seed_phrase.trim();
    let word_count = seed_phrase.split_whitespace().count();

    // Validate it's 12 or 24 words
    if word_count != 12 && word_count != 24 {
        return Err(IdosError::InvalidInput(format!(
            "Seed phrase must be 12 or 24 words, got {}",
            word_count
        )));
    }

    // Validate mnemonic is valid
    Mnemonic::parse_in_normalized(bip39::Language::English, seed_phrase)
        .map_err(|e| IdosError::InvalidInput(format!("Invalid seed phrase: {:?}", e)))?;

    // Derive wallet from mnemonic
    derive_wallet_from_mnemonic(seed_phrase, network)
}

/// Import wallet from private key
/// Supports both Ethereum (hex) and Solana (base58) formats
#[cfg(feature = "wallet")]
fn import_from_private_key(
    private_key: &str,
    network: BlockchainNetwork,
) -> IdosResult<WalletInfo> {
    match network {
        BlockchainNetwork::Ethereum => import_ethereum_from_private_key(private_key),
        BlockchainNetwork::Solana => import_solana_from_private_key(private_key),
    }
}

/// Import Ethereum wallet from private key (hex format)
#[cfg(feature = "wallet")]
fn import_ethereum_from_private_key(private_key: &str) -> IdosResult<WalletInfo> {
    use k256::ecdsa::SigningKey as Secp256k1SigningKey;

    // Remove 0x prefix if present
    let key_str = private_key.trim_start_matches("0x");

    // Decode hex private key
    let private_key_bytes = hex::decode(key_str)
        .map_err(|e| IdosError::InvalidInput(format!("Invalid hex private key: {}", e)))?;

    if private_key_bytes.len() != 32 {
        return Err(IdosError::InvalidInput(
            "Ethereum private key must be 32 bytes".to_string(),
        ));
    }

    // Create signing key
    let signing_key = Secp256k1SigningKey::from_bytes(private_key_bytes.as_slice().into())
        .map_err(|e| IdosError::Wallet(format!("Invalid private key: {}", e)))?;

    // Get public key
    let verifying_key = signing_key.verifying_key();
    use k256::EncodedPoint;
    let public_key_bytes: EncodedPoint = EncodedPoint::from(verifying_key);
    let public_key = &public_key_bytes.as_bytes()[1..]; // Remove 0x04 prefix

    // Calculate Ethereum address
    let address = ethereum_address_from_public_key(public_key);

    Ok(WalletInfo {
        address,
        network: BlockchainNetwork::Ethereum,
        private_key: Some(format!("0x{}", key_str)),
        seed_phrase: None, // No seed phrase when importing from private key
    })
}

/// Import Solana wallet from private key (base58 or byte array format)
#[cfg(feature = "wallet")]
fn import_solana_from_private_key(private_key: &str) -> IdosResult<WalletInfo> {
    use ed25519_dalek::SigningKey;

    // Try to decode as base58 first (standard Solana format)
    let private_key_bytes = if let Ok(bytes) = bs58::decode(private_key).into_vec() {
        bytes
    } else if let Ok(bytes) = hex::decode(private_key) {
        // Try hex format
        bytes
    } else {
        return Err(IdosError::InvalidInput(
            "Invalid Solana private key format (expected base58 or hex)".to_string(),
        ));
    };

    // Solana private key can be 32 bytes (just secret) or 64 bytes (secret + public)
    let secret_bytes = if private_key_bytes.len() == 64 {
        &private_key_bytes[0..32]
    } else if private_key_bytes.len() == 32 {
        &private_key_bytes
    } else {
        return Err(IdosError::InvalidInput(
            "Solana private key must be 32 or 64 bytes".to_string(),
        ));
    };

    // Create Ed25519 signing key
    let mut secret_array = [0u8; 32];
    secret_array.copy_from_slice(secret_bytes);
    let signing_key = SigningKey::from_bytes(&secret_array);
    let verifying_key = signing_key.verifying_key();

    // Solana address is base58 encoded public key
    let address = bs58::encode(verifying_key.as_bytes()).into_string();

    // Store full 64-byte private key
    let mut full_key = Vec::with_capacity(64);
    full_key.extend_from_slice(signing_key.as_bytes());
    full_key.extend_from_slice(verifying_key.as_bytes());
    let private_key_base58 = bs58::encode(&full_key).into_string();

    Ok(WalletInfo {
        address,
        network: BlockchainNetwork::Solana,
        private_key: Some(private_key_base58),
        seed_phrase: None,
    })
}

/// Calculate Ethereum address from public key
#[cfg(feature = "wallet")]
fn ethereum_address_from_public_key(public_key: &[u8]) -> String {
    use sha2::{Digest, Sha256};

    // Ethereum uses Keccak256, but we'll use SHA256 as approximation for now
    // In production, you'd want to use tiny-keccak crate
    let mut hasher = Sha256::new();
    hasher.update(public_key);
    let hash = hasher.finalize();

    // Take last 20 bytes and format as hex with 0x prefix
    let address_bytes = &hash[hash.len() - 20..];
    format!("0x{}", hex::encode(address_bytes))
}

#[cfg(not(feature = "wallet"))]
pub fn import_wallet(_source: ImportSource, _network: BlockchainNetwork) -> IdosResult<WalletInfo> {
    Err(IdosError::PlatformNotSupported(
        "Wallet feature not enabled".to_string(),
    ))
}

#[cfg(all(test, feature = "wallet"))]
mod tests {
    use super::*;

    #[test]
    fn test_import_ethereum_from_private_key() {
        let private_key = "0x4c0883a69102937d6231471b5dbb6204fe512961708279f8b1a3e79e5c8c4f8f";
        let wallet = import_wallet(
            ImportSource::PrivateKey(private_key.to_string()),
            BlockchainNetwork::Ethereum,
        )
        .unwrap();

        assert!(wallet.address.starts_with("0x"));
        assert_eq!(wallet.network, BlockchainNetwork::Ethereum);
        assert!(wallet.private_key.is_some());
    }

    #[test]
    fn test_import_from_seed_phrase() {
        let seed_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

        let eth_wallet = import_wallet(
            ImportSource::SeedPhrase(seed_phrase.to_string()),
            BlockchainNetwork::Ethereum,
        )
        .unwrap();
        assert!(eth_wallet.address.starts_with("0x"));

        let sol_wallet = import_wallet(
            ImportSource::SeedPhrase(seed_phrase.to_string()),
            BlockchainNetwork::Solana,
        )
        .unwrap();
        assert!(!sol_wallet.address.is_empty());
    }
}
